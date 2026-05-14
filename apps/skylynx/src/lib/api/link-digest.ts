import { cacheActor, cacheDigestRun, cacheExternalLinkPosts, cacheFollows, getCachedFollows } from '../db/database';
import { fetchActorProfile, fetchAllFollows, fetchFollowExternalLinkPosts } from './bluesky';

import type {
  DigestLink,
  ExternalLinkPost,
  LinkDigestOptions,
  LinkDigestProgress,
  LinkDigestResult,
  LinkDigestStatusEvent
} from '../types';

const AUTHOR_FEED_MAX_PARALLEL = 4;
const AUTHOR_FEED_START_DELAY_MS = 100;

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

export async function* generateLinkDigest(options: LinkDigestOptions): AsyncGenerator<LinkDigestStatusEvent, LinkDigestResult> {
  yield { actor: options.actor, type: 'resolving-actor' };
  const actor = await fetchActorProfile(options.actor);
  await cacheActor(actor);
  yield { actor, type: 'actor-resolved' };

  yield { actorDid: actor.did, refresh: options.refreshFollows, type: 'loading-follows' };
  const cachedFollows = options.refreshFollows ? [] : await getCachedFollows(actor.did);
  const follows = cachedFollows.length > 0 ? cachedFollows : await fetchAllFollows(actor.did);
  await cacheFollows(actor.did, follows);
  yield { count: follows.length, source: cachedFollows.length > 0 ? 'cache' : 'network', type: 'follows-loaded' };

  let completed = 0;
  yield { completed, total: follows.length, type: 'fetching-feeds' };

  const flatPosts: ExternalLinkPost[] = [];
  for (let index = 0; index < follows.length; index += AUTHOR_FEED_MAX_PARALLEL) {
    const batch = follows.slice(index, index + AUTHOR_FEED_MAX_PARALLEL);
    const batchPosts = await Promise.all(
      batch.map(async (follow, batchIndex) => {
        await delay(batchIndex * AUTHOR_FEED_START_DELAY_MS);
        const links = await fetchFollowExternalLinkPosts({
          feedLimit: options.feedLimit,
          follow,
          maxPages: options.maxPages,
          since: options.since,
          until: options.until
        });

        return { follow, links };
      })
    );

    for (const { follow, links } of batchPosts) {
      flatPosts.push(...links);
      completed += 1;
      yield { completed, follow, linkCount: links.length, total: follows.length, type: 'follow-feed-fetched' };
    }
  }

  yield { count: flatPosts.length, type: 'caching-posts' };
  await cacheExternalLinkPosts(flatPosts);

  const links = aggregateDigestLinks(flatPosts)
    .filter((link) => link.score >= options.minScore)
    .filter((link) => link.sharers.length >= options.minShares)
    .toSorted(compareDigestLinks)
    .slice(0, options.limit);

  await cacheDigestRun({ actor: options.actor, actorDid: actor.did, linkCount: links.length, options });
  yield { linkCount: links.length, postCount: flatPosts.length, type: 'digest-ready' };

  const result = { actor, follows, links, posts: flatPosts };
  yield { result, type: 'done' };

  return result;
}

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
const delay = (milliseconds: number) => new Promise((resolve) => setTimeout(resolve, milliseconds));

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

  if (event.type === 'done') {
    return { completed: event.result.follows.length, phase: 'done', total: event.result.follows.length };
  }

  return { completed: 0, phase: 'fetching-feeds', total: 0 };
};
