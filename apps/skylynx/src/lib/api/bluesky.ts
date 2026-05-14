import { Client, ok, simpleFetchHandler } from '@atcute/client';
import type { AppBskyActorDefs, AppBskyFeedDefs } from '@atcute/bluesky';
import type { ActorIdentifier } from '@atcute/lexicons';
import type { ExternalLinkPost, Follow } from '../types';

const BSKY_PUBLIC_API = 'https://public.api.bsky.app';
const MAX_PAGE_SIZE = 100;

type JsonRecord = Record<string, unknown>;

export const fetchActorProfile = async (
  actor: string,
  fetcher = fetch
): Promise<AppBskyActorDefs.ProfileViewDetailed> => {
  return ok(
    createBlueskyClient(fetcher).get('app.bsky.actor.getProfile', { params: { actor: actorIdentifier(actor) } })
  );
};

export const fetchAllFollows = async (
  actor: string,
  options: { fetcher?: typeof fetch; limit?: number } = {}
): Promise<Follow[]> => {
  const follows: Follow[] = [];
  let cursor: string | undefined;

  do {
    const page = await ok(
      createBlueskyClient(options.fetcher ?? fetch).get('app.bsky.graph.getFollows', {
        params: { actor: actorIdentifier(actor), cursor, limit: clampLimit(options.limit ?? MAX_PAGE_SIZE) }
      })
    );

    follows.push(
      ...page.follows.map((profile) => {
        return followFromProfile(profile);
      })
    );
    cursor = page.cursor;
  } while (cursor);

  return follows;
};

export const fetchFollowExternalLinkPosts = async ({
  feedLimit,
  fetcher = fetch,
  follow,
  maxPages,
  since,
  until
}: {
  feedLimit: number;
  fetcher?: typeof fetch;
  follow: Follow;
  maxPages: number;
  since?: string;
  until?: string;
}): Promise<ExternalLinkPost[]> => {
  const links: ExternalLinkPost[] = [];
  let cursor: string | undefined;

  for (let pageNumber = 0; pageNumber < maxPages; pageNumber += 1) {
    const page = await ok(
      createBlueskyClient(fetcher).get('app.bsky.feed.getAuthorFeed', {
        params: {
          actor: actorIdentifier(follow.did),
          cursor,
          filter: 'posts_with_replies',
          includePins: false,
          limit: clampLimit(feedLimit)
        }
      })
    );

    for (const item of page.feed) {
      if (!postMatchesWindow(item, since, until)) continue;

      const post = extractExternalLinkPost(item, follow);
      if (post) links.push(post);
    }

    if (!page.cursor) break;
    cursor = page.cursor;
  }

  return links;
};

const extractExternalLinkPost = (item: AppBskyFeedDefs.FeedViewPost, follow: Follow): ExternalLinkPost | undefined => {
  const embed = item.post.embed;
  if (embed?.$type !== 'app.bsky.embed.external#view') return;

  return {
    author: item.post.author.handle,
    authorDid: item.post.author.did,
    bookmarkCount: item.post.bookmarkCount ?? 0,
    createdAt: createdAt(item.post.record),
    description: embed.external.description,
    externalUri: embed.external.uri,
    indexedAt: item.post.indexedAt,
    likeCount: item.post.likeCount ?? 0,
    postUri: item.post.uri,
    repostCount: item.post.repostCount ?? 0,
    sharedAt: sharedAt(item),
    sharedBy: follow.handle,
    sharedByDid: follow.did,
    title: embed.external.title
  };
};

const postMatchesWindow = (item: AppBskyFeedDefs.FeedViewPost, since?: string, until?: string) => {
  const sortAt = createdAt(item.post.record) ?? item.post.indexedAt;

  if (since && sortAt < since) return false;
  if (until && sortAt >= until) return false;

  return true;
};

const sharedAt = (item: AppBskyFeedDefs.FeedViewPost) => {
  if (item.reason && 'indexedAt' in item.reason) {
    return item.reason.indexedAt;
  }

  return createdAt(item.post.record) ?? item.post.indexedAt;
};

const createBlueskyClient = (fetcher: typeof fetch) => {
  return new Client({ handler: simpleFetchHandler({ service: BSKY_PUBLIC_API, fetch: fetcher }) });
};

const actorIdentifier = (actor: string) => actor as ActorIdentifier;

const followFromProfile = (profile: AppBskyActorDefs.ProfileView): Follow => {
  return { did: profile.did, handle: profile.handle, profileUrl: `https://bsky.app/profile/${profile.handle}` };
};

const createdAt = (record: unknown) => {
  if (!isRecord(record)) return;
  return stringValue(record.createdAt);
};

const clampLimit = (limit: number) => Math.max(1, Math.min(MAX_PAGE_SIZE, limit));
const isRecord = (value: unknown): value is JsonRecord => typeof value === 'object' && value !== null;
const stringValue = (value: unknown) => (typeof value === 'string' ? value : undefined);
