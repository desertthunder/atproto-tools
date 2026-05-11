import { ok } from '@atcute/client';

import { createBlueskyClient, createConstellationClient } from './clients';

import type { Did, GraphFetchOptions, GraphPage, MutualsOptions, ProfileView } from '$lib/types/api';

const FOLLOW_SOURCE = 'app.bsky.graph.follow:subject' as const;
const MAX_PAGE_SIZE = 100;

export const fetchActorProfile = async ({
  actor,
  fetch
}: Pick<GraphFetchOptions, 'actor' | 'fetch'>): Promise<ProfileView> => {
  const client = createBlueskyClient({ fetch });
  return ok(client.get('app.bsky.actor.getProfile', { params: { actor } }));
};

export const fetchFollowingPage = async ({
  actor,
  cursor,
  fetch,
  limit = MAX_PAGE_SIZE
}: GraphFetchOptions): Promise<GraphPage<ProfileView>> => {
  const client = createBlueskyClient({ fetch });
  const data = await ok(
    client.get('app.bsky.graph.getFollows', { params: { actor, cursor, limit: clampLimit(limit) } })
  );

  return { cursor: data.cursor, items: data.follows };
};

export const fetchFollowersPage = async ({
  actor,
  cursor,
  fetch,
  limit = MAX_PAGE_SIZE
}: GraphFetchOptions): Promise<GraphPage<ProfileView>> => {
  const client = createBlueskyClient({ fetch });
  const data = await ok(
    client.get('app.bsky.graph.getFollowers', { params: { actor, cursor, limit: clampLimit(limit) } })
  );

  return { cursor: data.cursor, items: data.followers };
};

export const fetchAllFollowing = async (options: GraphFetchOptions): Promise<ProfileView[]> => {
  return collectPageItems(fetchFollowingPages(options));
};

export const fetchAllFollowers = async (options: GraphFetchOptions): Promise<ProfileView[]> => {
  return collectPageItems(fetchFollowersPages(options));
};

export async function* fetchFollowingPages(options: GraphFetchOptions): AsyncGenerator<GraphPage<ProfileView>> {
  yield* fetchPages((cursor) => fetchFollowingPage({ ...options, cursor }));
}

export async function* fetchFollowersPages(options: GraphFetchOptions): AsyncGenerator<GraphPage<ProfileView>> {
  yield* fetchPages((cursor) => fetchFollowersPage({ ...options, cursor }));
}

export async function* fetchFollowingItems(options: GraphFetchOptions): AsyncGenerator<ProfileView> {
  yield* flattenPages(fetchFollowingPages(options));
}

export async function* fetchFollowersItems(options: GraphFetchOptions): AsyncGenerator<ProfileView> {
  yield* flattenPages(fetchFollowersPages(options));
}

export const findMutualDidsWithConstellation = async ({
  actorDid,
  fetch,
  followingDids,
  limit = MAX_PAGE_SIZE
}: MutualsOptions): Promise<Did[]> => {
  if (followingDids.length === 0) {
    return [];
  }

  const client = createConstellationClient({ fetch });
  const mutuals = new Set<Did>();

  for (const dids of chunk(followingDids, MAX_PAGE_SIZE)) {
    const data = await ok(
      client.get('blue.microcosm.links.getBacklinks', {
        params: { did: dids, limit: clampLimit(limit), source: FOLLOW_SOURCE, subject: actorDid }
      })
    );

    for (const record of data.records) {
      mutuals.add(record.did);
    }
  }

  return [...mutuals];
};

export const fetchMutualProfiles = async (options: GraphFetchOptions & { actorDid: Did }): Promise<ProfileView[]> => {
  const following = await fetchAllFollowing(options);
  const mutualDids = await findMutualDidsWithConstellation({
    actorDid: options.actorDid,
    fetch: options.fetch,
    followingDids: following.map((profile) => profile.did),
    limit: options.limit
  });
  const mutualDidSet = new Set(mutualDids);

  return following.filter((profile) => mutualDidSet.has(profile.did));
};

async function* fetchPages<T>(fetchPage: (cursor?: string) => Promise<GraphPage<T>>): AsyncGenerator<GraphPage<T>> {
  let cursor: string | undefined;

  do {
    const page = await fetchPage(cursor);
    yield page;
    cursor = page.cursor;
  } while (cursor);
}

async function* flattenPages<T>(pages: AsyncIterable<GraphPage<T>>): AsyncGenerator<T> {
  for await (const page of pages) {
    yield* page.items;
  }
}

const collectPageItems = async <T>(pages: AsyncIterable<GraphPage<T>>) => {
  const items: T[] = [];

  for await (const page of pages) {
    items.push(...page.items);
  }

  return items;
};

const clampLimit = (limit: number) => Math.max(1, Math.min(MAX_PAGE_SIZE, limit));

const chunk = <T>(items: T[], size: number): T[][] => {
  const chunks: T[][] = [];

  for (let index = 0; index < items.length; index += size) {
    chunks.push(items.slice(index, index + size));
  }

  return chunks;
};
