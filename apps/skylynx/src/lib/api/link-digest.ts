import type { AppBskyActorDefs } from '@atcute/bluesky';
import { cacheActor, cacheDigestRun, cacheExternalLinkPosts, cacheFollows, getCachedFollows } from '../db/database';
import { fetchActorProfile, fetchAllFollows, fetchFollowExternalLinkPosts } from './bluesky';

import type { DigestLink, ExternalLinkPost, Follow, LinkDigestOptions, LinkDigestProgress } from '../types';

const AUTHOR_FEED_MAX_PARALLEL = 4;
const AUTHOR_FEED_START_DELAY_MS = 100;

export const buildLinkDigest = async (
  options: LinkDigestOptions,
  progress: (progress: LinkDigestProgress) => void = () => {}
): Promise<LinkDigestResult> => {
  progress({ completed: 0, phase: 'resolving', total: 0 });
  const actor = await fetchActorProfile(options.actor);
  await cacheActor(actor);

  progress({ completed: 0, phase: 'fetching-follows', total: 0 });
  const cachedFollows = options.refreshFollows ? [] : await getCachedFollows(actor.did);
  const follows = cachedFollows.length > 0 ? cachedFollows : await fetchAllFollows(actor.did);
  await cacheFollows(actor.did, follows);

  let completed = 0;
  progress({ completed, phase: 'fetching-feeds', total: follows.length });
  const posts = await mapWithConcurrency(follows, AUTHOR_FEED_MAX_PARALLEL, async (follow, index) => {
    await delay(index * AUTHOR_FEED_START_DELAY_MS);
    const links = await fetchFollowExternalLinkPosts({
      feedLimit: options.feedLimit,
      follow,
      maxPages: options.maxPages,
      since: options.since,
      until: options.until
    });
    completed += 1;
    progress({ completed, phase: 'fetching-feeds', total: follows.length });
    return links;
  });

  const flatPosts = posts.flat();
  await cacheExternalLinkPosts(flatPosts);

  const links = aggregateDigestLinks(flatPosts)
    .filter((link) => link.score >= options.minScore)
    .filter((link) => link.sharers.length >= options.minShares)
    .toSorted(compareDigestLinks)
    .slice(0, options.limit);

  await cacheDigestRun({ actor: options.actor, actorDid: actor.did, linkCount: links.length, options });
  progress({ completed: follows.length, phase: 'done', total: follows.length });

  return { actor, follows, links, posts: flatPosts };
};

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

const mapWithConcurrency = async <Input, Output>(
  items: Input[],
  concurrency: number,
  mapper: (item: Input, index: number) => Promise<Output>
) => {
  const results: Output[] = [];
  let nextIndex = 0;

  const workers = Array.from({ length: Math.min(concurrency, items.length) }, async () => {
    while (nextIndex < items.length) {
      const index = nextIndex;
      nextIndex += 1;
      results[index] = await mapper(items[index], index);
    }
  });

  await Promise.all(workers);
  return results;
};

const distinctSorted = (items: string[]) => [...new Set(items)].toSorted();
const delay = (milliseconds: number) => new Promise((resolve) => setTimeout(resolve, milliseconds));

type LinkDigestResult = {
  actor: AppBskyActorDefs.ProfileViewDetailed;
  follows: Follow[];
  links: DigestLink[];
  posts: ExternalLinkPost[];
};
