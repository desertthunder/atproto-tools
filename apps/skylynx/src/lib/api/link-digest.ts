import PQueue from 'p-queue';

import {
  cacheActor,
  cacheAuthorActivity,
  cacheDigestRun,
  cacheExternalLinkPosts,
  cacheFollows,
  createDigestProgress,
  getDigestProgress,
  getCachedFollows,
  markDigestProgressStatus,
  updateDigestProgress
} from '../db/database';
import { fetchActorProfile, fetchAllFollows, fetchFollowExternalLinkPosts } from './bluesky';

import type {
  DigestLink,
  AuthorActivity,
  Did,
  ExternalLinkPost,
  LinkDigestOptions,
  LinkDigestProgress,
  LinkDigestResult,
  LinkDigestStatusEvent
} from '../types';

const AUTHOR_SCAN_CONCURRENCY = 32;
const FEED_REQUEST_CONCURRENCY = 12;
const FEED_REQUEST_INTERVAL_CAP = 24;
const FEED_REQUEST_INTERVAL_MS = 1000;

type GenerateLinkDigestOptions = { resumeRunId?: string; shouldPause?: () => boolean };

export const buildLinkDigest = async (
  options: LinkDigestOptions,
  progress: (progress: LinkDigestProgress) => void = () => {}
): Promise<LinkDigestResult> => {
  let result: LinkDigestResult | undefined;

  for await (const event of generateLinkDigest(options)) {
    progress(progressFromEvent(event));
    if (event.type === 'done') {
      result = event.result;
    }
  }

  if (!result) {
    throw new Error('Digest did not complete');
  }

  return result;
};

export async function* generateLinkDigest(
  options: LinkDigestOptions,
  runOptions: GenerateLinkDigestOptions = {}
): AsyncGenerator<LinkDigestStatusEvent, LinkDigestResult> {
  const initialState = runOptions.resumeRunId ? await getDigestProgress(runOptions.resumeRunId) : undefined;
  const normalizedOptions = initialState?.options ?? options;
  const progress = initialState ?? (await createDigestProgress(options));

  yield { actor: normalizedOptions.actor, type: 'resolving-actor' };
  const actor = await fetchActorProfile(normalizedOptions.actor);
  await cacheActor(actor);
  await updateDigestProgress(progress.id, { actorDid: actor.did, phase: 'fetching-follows', status: 'running' });
  yield { actor, type: 'actor-resolved' };

  yield { actorDid: actor.did, refresh: normalizedOptions.refreshFollows, type: 'loading-follows' };
  const followResult =
    initialState && initialState.follows.length > 0
      ? { follows: initialState.follows, source: 'cache' as const }
      : await loadFollows(actor.did, normalizedOptions);
  const follows = followResult.follows;
  await updateDigestProgress(progress.id, {
    actorDid: actor.did,
    follows,
    phase: 'fetching-feeds',
    total: follows.length
  });
  yield { count: follows.length, source: followResult.source, type: 'follows-loaded' };

  const processedFollowDids = new Set(initialState?.processedFollowDids);
  let completed = processedFollowDids.size;
  yield { completed, total: follows.length, type: 'fetching-feeds' };

  const flatPosts: ExternalLinkPost[] = [...(initialState?.posts ?? [])];
  const activities: AuthorActivity[] = [];
  const remainingFollows = follows.filter((follow) => !processedFollowDids.has(follow.did));

  for await (const { activity, follow, links } of scanFollowFeeds(remainingFollows, normalizedOptions, runOptions)) {
    if (activity) activities.push(activity);
    flatPosts.push(...links);
    processedFollowDids.add(follow.did);
    completed += 1;
    await updateDigestProgress(progress.id, {
      completed,
      phase: 'fetching-feeds',
      postCount: flatPosts.length,
      posts: flatPosts,
      processedFollowDids: [...processedFollowDids],
      status: 'running',
      total: follows.length
    });
    yield { completed, follow, linkCount: links.length, total: follows.length, type: 'follow-feed-fetched' };

    if (runOptions.shouldPause?.()) {
      await markDigestProgressStatus(progress.id, 'paused');
      yield { completed, runId: progress.id, total: follows.length, type: 'paused' };
      return incompleteDigestResult(actor, follows, flatPosts, normalizedOptions);
    }
  }

  await updateDigestProgress(progress.id, { phase: 'caching-posts', postCount: flatPosts.length, posts: flatPosts });
  yield { count: flatPosts.length, type: 'caching-posts' };
  await cacheAuthorActivity(activities);
  await cacheExternalLinkPosts(flatPosts);

  const links = aggregateDigestLinks(flatPosts)
    .filter((link) => link.score >= normalizedOptions.minScore)
    .filter((link) => link.sharers.length >= normalizedOptions.minShares)
    .toSorted(compareDigestLinks)
    .slice(0, normalizedOptions.limit);

  await cacheDigestRun({
    actor: normalizedOptions.actor,
    actorDid: actor.did,
    linkCount: links.length,
    options: normalizedOptions
  });
  await updateDigestProgress(progress.id, {
    completed: follows.length,
    phase: 'done',
    postCount: flatPosts.length,
    status: 'completed',
    total: follows.length
  });
  yield { linkCount: links.length, postCount: flatPosts.length, type: 'digest-ready' };

  const result = { actor, follows, links, posts: flatPosts };
  yield { result, type: 'done' };

  return result;
}

const loadFollows = async (actorDid: Did, options: LinkDigestOptions) => {
  const cachedFollows = options.refreshFollows ? [] : await getCachedFollows(actorDid);
  const follows = cachedFollows.length > 0 ? cachedFollows : await fetchAllFollows(actorDid);
  await cacheFollows(actorDid, follows);
  return { follows, source: cachedFollows.length > 0 ? ('cache' as const) : ('network' as const) };
};

const incompleteDigestResult = (
  actor: LinkDigestResult['actor'],
  follows: LinkDigestResult['follows'],
  posts: ExternalLinkPost[],
  options: LinkDigestOptions
): LinkDigestResult => {
  const links = aggregateDigestLinks(posts)
    .filter((link) => link.score >= options.minScore)
    .filter((link) => link.sharers.length >= options.minShares)
    .toSorted(compareDigestLinks)
    .slice(0, options.limit);

  return { actor, follows, links, posts };
};

type FollowFeedScanResult = {
  activity?: AuthorActivity;
  follow: LinkDigestResult['follows'][number];
  links: ExternalLinkPost[];
};

async function* scanFollowFeeds(
  follows: LinkDigestResult['follows'],
  options: LinkDigestOptions,
  runOptions: GenerateLinkDigestOptions
): AsyncGenerator<FollowFeedScanResult> {
  const requestQueue = new PQueue({
    carryoverConcurrencyCount: true,
    concurrency: FEED_REQUEST_CONCURRENCY,
    interval: FEED_REQUEST_INTERVAL_MS,
    intervalCap: FEED_REQUEST_INTERVAL_CAP
  });
  const pending = new Map<number, Promise<QueuedFollowFeedScanResult>>();
  let nextIndex = 0;
  let taskId = 0;

  const scheduleNext = () => {
    if (nextIndex >= follows.length) return;
    if (runOptions.shouldPause?.()) return;

    const id = taskId;
    taskId += 1;
    const follow = follows[nextIndex];
    nextIndex += 1;

    pending.set(
      id,
      Promise.resolve().then(async () => {
        const { activity, links } = await fetchFollowExternalLinkPosts({
          feedLimit: options.feedLimit,
          follow,
          maxPages: options.maxPages,
          requestQueue,
          since: options.since,
          until: options.until
        });

        return { activity, follow, id, links };
      })
    );
  };

  while (pending.size < AUTHOR_SCAN_CONCURRENCY && nextIndex < follows.length) {
    scheduleNext();
  }

  while (pending.size > 0) {
    const result = await Promise.race(pending.values());
    pending.delete(result.id);
    yield result;

    while (pending.size < AUTHOR_SCAN_CONCURRENCY && nextIndex < follows.length) {
      scheduleNext();
    }
  }

  await requestQueue.onIdle();
}

type QueuedFollowFeedScanResult = FollowFeedScanResult & { id: number };

export const aggregateDigestLinks = (posts: ExternalLinkPost[]): DigestLink[] => {
  const byUri = new Map<string, DigestLink>();

  for (const post of posts) {
    const current = byUri.get(post.externalUri);
    const link =
      current ??
      ({
        bookmarkCount: 0,
        description: post.description,
        firstSeen: post.sharedAt,
        lastSeen: post.sharedAt,
        likeCount: 0,
        repostCount: 0,
        score: 0,
        shares: [],
        sharers: [],
        title: post.title,
        uri: post.externalUri
      } satisfies DigestLink);

    const alreadyCountedPost = link.shares.some((share) => share.postUri === post.postUri);
    const alreadyRecordedShare = link.shares.some(
      (share) => share.postUri === post.postUri && share.sharedByDid === post.sharedByDid
    );
    if (alreadyRecordedShare) continue;

    if (!link.title && post.title) link.title = post.title;
    if (!link.description && post.description) link.description = post.description;
    if (post.sharedAt < link.firstSeen) link.firstSeen = post.sharedAt;
    if (post.sharedAt > link.lastSeen) link.lastSeen = post.sharedAt;

    if (!alreadyCountedPost) {
      link.bookmarkCount += post.bookmarkCount;
      link.repostCount += post.repostCount;
      link.likeCount += post.likeCount;
      link.score = link.bookmarkCount + link.repostCount + link.likeCount;
    }

    link.shares.push(post);
    link.sharers = distinctSorted(link.shares.map((share) => share.sharedBy));
    byUri.set(post.externalUri, link);
  }

  return [...byUri.values()];
};

const compareDigestLinks = (left: DigestLink, right: DigestLink) => {
  const shareOrder = right.sharers.length - left.sharers.length;
  if (shareOrder !== 0) return shareOrder;

  const scoreOrder = right.score - left.score;
  if (scoreOrder !== 0) return scoreOrder;

  const titleOrder = left.title.localeCompare(right.title);
  if (titleOrder !== 0) return titleOrder;

  return left.uri.localeCompare(right.uri);
};

const distinctSorted = (items: string[]) => [...new Set(items)].toSorted();

const progressFromEvent = (event: LinkDigestStatusEvent): LinkDigestProgress => {
  if (event.type === 'resolving-actor' || event.type === 'actor-resolved') {
    return { completed: 0, phase: 'resolving', total: 0 };
  }

  if (event.type === 'loading-follows' || event.type === 'follows-loaded') {
    return { completed: 0, phase: 'fetching-follows', total: event.type === 'follows-loaded' ? event.count : 0 };
  }

  if (event.type === 'fetching-feeds' || event.type === 'follow-feed-fetched') {
    return { completed: event.completed, phase: 'fetching-feeds', total: event.total };
  }

  if (event.type === 'paused') {
    return { completed: event.completed, phase: 'paused', total: event.total };
  }

  if (event.type === 'done') {
    return { completed: event.result.follows.length, phase: 'done', total: event.result.follows.length };
  }

  return { completed: 0, phase: 'fetching-feeds', total: 0 };
};
