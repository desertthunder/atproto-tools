import Dexie, { type EntityTable } from 'dexie';

import {
  DB_NAME,
  DB_VERSION,
  actorFromProfile,
  digestRunId,
  externalLinkFromPost,
  followEdgeFromFollow,
  linkShareFromPost,
  type CachedActor,
  type DigestRun,
  type ExternalLink,
  type FollowEdge,
  type LinkShare
} from './schema';

import type { ActorProfile, Did, ExternalLinkPost, Follow, LinkDigestOptions } from '../types';

export class SkylynxDb extends Dexie {
  actors!: EntityTable<CachedActor, 'did'>;
  digestRuns!: EntityTable<DigestRun, 'id'>;
  externalLinks!: EntityTable<ExternalLink, 'uri'>;
  followEdges!: EntityTable<FollowEdge, 'id'>;
  linkShares!: EntityTable<LinkShare, 'id'>;

  constructor() {
    super(DB_NAME);

    this.version(1).stores({
      actors: '&did, handle, updatedAt',
      externalLinks: '&uri, updatedAt',
      followEdges: '&id, actorDid, followedDid, [actorDid+followedDid], updatedAt',
      linkShares: '&id, externalUri, postUri, sharedByDid, sharedAt, [externalUri+sharedByDid], updatedAt'
    });

    this.version(DB_VERSION).stores({
      actors: '&did, handle, updatedAt',
      digestRuns: '&id, actorDid, completedAt',
      externalLinks: '&uri, updatedAt',
      followEdges: '&id, actorDid, followedDid, [actorDid+followedDid], updatedAt',
      linkShares: '&id, externalUri, postUri, sharedByDid, sharedAt, [externalUri+sharedByDid], updatedAt'
    });
  }
}

export const db = new SkylynxDb();

export const cacheActor = async (profile: ActorProfile) => {
  await db.actors.put(actorFromProfile(profile));
};

export const cacheFollows = async (actorDid: Did, follows: Follow[]) => {
  const updatedAt = new Date().toISOString();

  await db.transaction('rw', db.followEdges, async () => {
    await db.followEdges.bulkPut(follows.map((follow) => followEdgeFromFollow(actorDid, follow, updatedAt)));
  });
};

export const getCachedFollows = async (actorDid: Did) => {
  const rows = await db.followEdges.where('actorDid').equals(actorDid).toArray();
  return rows.map((row) => {
    return { did: row.followedDid, handle: row.followedHandle, profileUrl: row.profileUrl } satisfies Follow;
  });
};

export const cacheExternalLinkPosts = async (posts: ExternalLinkPost[]) => {
  if (posts.length === 0) return;

  const updatedAt = new Date().toISOString();

  await db.transaction('rw', db.externalLinks, db.linkShares, async () => {
    await db.externalLinks.bulkPut(posts.map((post) => externalLinkFromPost(post, updatedAt)));
    await db.linkShares.bulkPut(posts.map((post) => linkShareFromPost(post, updatedAt)));
  });
};

export const cacheDigestRun = async ({
  actor,
  actorDid,
  linkCount,
  options
}: {
  actor: string;
  actorDid: Did;
  linkCount: number;
  options: LinkDigestOptions;
}) => {
  await db.digestRuns.put({
    actor,
    actorDid,
    completedAt: new Date().toISOString(),
    id: digestRunId(actorDid),
    linkCount,
    options: {
      feedLimit: options.feedLimit,
      maxPages: options.maxPages,
      minScore: options.minScore,
      minShares: options.minShares,
      since: options.since,
      until: options.until
    }
  });
};
