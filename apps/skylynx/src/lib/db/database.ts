import Dexie, { type EntityTable } from 'dexie';

import {
  DB_NAME,
  DB_VERSION,
  accountFromRelationshipEdge,
  actorFromProfile,
  authorActivityForStorage,
  digestProgressId,
  digestRunId,
  externalLinkFromPost,
  followEdgeFromFollow,
  linkShareFromPost,
  relationshipEdgeFromAccount,
  relationshipEdgeId,
  type CachedAuthorActivity,
  type CachedActor,
  type DigestRun,
  type DigestProgressSnapshot,
  type DigestProgressStatus,
  type ExternalLink,
  type FollowEdge,
  type LinkShare,
  type RelationshipEdge
} from './schema';

import type {
  ActorProfile,
  AuthorActivity,
  Did,
  ExternalLinkPost,
  Follow,
  GraphRelationship,
  LinkDigestOptions,
  RelationshipAccount
} from '../types';

export class SkylynxDb extends Dexie {
  authorActivity!: EntityTable<CachedAuthorActivity, 'authorDid'>;
  actors!: EntityTable<CachedActor, 'did'>;
  digestProgress!: EntityTable<DigestProgressSnapshot, 'id'>;
  digestRuns!: EntityTable<DigestRun, 'id'>;
  externalLinks!: EntityTable<ExternalLink, 'uri'>;
  followEdges!: EntityTable<FollowEdge, 'id'>;
  linkShares!: EntityTable<LinkShare, 'id'>;
  relationshipEdges!: EntityTable<RelationshipEdge, 'id'>;

  constructor() {
    super(DB_NAME);

    this.version(1).stores({
      actors: '&did, handle, updatedAt',
      externalLinks: '&uri, updatedAt',
      followEdges: '&id, actorDid, followedDid, [actorDid+followedDid], updatedAt',
      linkShares: '&id, externalUri, postUri, sharedByDid, sharedAt, [externalUri+sharedByDid], updatedAt'
    });

    this.version(DB_VERSION).stores({
      authorActivity: '&authorDid, handle, lastPostAt, updatedAt',
      actors: '&did, handle, updatedAt',
      digestProgress: '&id, actor, actorDid, status, phase, updatedAt, createdAt',
      digestRuns: '&id, actorDid, completedAt',
      externalLinks: '&uri, updatedAt',
      followEdges: '&id, actorDid, followedDid, [actorDid+followedDid], updatedAt',
      linkShares: '&id, externalUri, postUri, sharedByDid, sharedAt, [externalUri+sharedByDid], updatedAt',
      relationshipEdges: '&id, actorDid, relationship, subjectDid, [actorDid+relationship], updatedAt, lastPostAt'
    });
  }
}

export const db = new SkylynxDb();

export const cacheActor = async (profile: ActorProfile) => {
  await db.actors.put(actorFromProfile(profile));
};

export const cacheFollows = async (actorDid: Did, follows: Follow[]) => {
  const updatedAt = new Date().toISOString();

  await db.transaction('rw', db.followEdges, db.relationshipEdges, async () => {
    await db.followEdges.bulkPut(follows.map((follow) => followEdgeFromFollow(actorDid, follow, updatedAt)));
    await cacheRelationshipRows(
      actorDid,
      follows.map((follow) => ({ ...follow, relationship: 'following' })),
      updatedAt
    );
  });
};

export const getCachedFollows = async (actorDid: Did) => {
  const relationshipRows = await getCachedRelationships(actorDid, 'following');
  if (relationshipRows.length > 0) {
    return relationshipRows.map((row) => {
      return { did: row.did, handle: row.handle, profileUrl: row.profileUrl } satisfies Follow;
    });
  }

  const rows = await db.followEdges.where('actorDid').equals(actorDid).toArray();
  return rows.map((row): Follow => {
    return { did: row.followedDid, handle: row.followedHandle, profileUrl: row.profileUrl } satisfies Follow;
  });
};

export const cacheRelationships = async (
  actorDid: Did,
  relationship: GraphRelationship,
  accounts: Omit<RelationshipAccount, 'relationship'>[]
) => {
  const updatedAt = new Date().toISOString();
  await db.transaction('rw', db.relationshipEdges, async () => {
    await cacheRelationshipRows(
      actorDid,
      accounts.map((account) => ({ ...account, relationship })),
      updatedAt
    );
  });
};

export const getCachedRelationships = async (actorDid: Did, relationship: GraphRelationship) => {
  const rows = await db.relationshipEdges.where('[actorDid+relationship]').equals([actorDid, relationship]).toArray();
  return rows.map(accountFromRelationshipEdge).toSorted(compareRelationshipAccounts);
};

export const cacheAuthorActivity = async (activities: AuthorActivity[]) => {
  if (activities.length === 0) return;

  const latest = latestActivities(activities);
  const updatedAt = new Date().toISOString();

  await db.transaction('rw', db.authorActivity, db.relationshipEdges, async () => {
    await db.authorActivity.bulkPut(latest.map((activity) => authorActivityForStorage(activity, updatedAt)));

    for (const activity of latest) {
      const rows = await db.relationshipEdges.where('subjectDid').equals(activity.authorDid).toArray();
      await db.relationshipEdges.bulkPut(
        rows.map((row) => ({ ...row, lastPostAt: activity.lastPostAt, lastPostUri: activity.lastPostUri, updatedAt }))
      );
    }
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

export const createDigestProgress = async (options: LinkDigestOptions) => {
  const now = new Date().toISOString();
  const run: DigestProgressSnapshot = {
    actor: options.actor,
    completed: 0,
    createdAt: now,
    follows: [],
    id: digestProgressId(),
    options: digestProgressOptions(options),
    phase: 'resolving',
    postCount: 0,
    posts: [],
    processedFollowDids: [],
    status: 'running',
    total: 0,
    updatedAt: now
  };

  await db.digestProgress.put(run);
  return run;
};

export const getDigestProgress = async (id: string) => {
  return db.digestProgress.get(id);
};

export const getLatestPausedDigestProgress = async (actor?: string) => {
  const rows = await db.digestProgress.where('status').equals('paused').toArray();
  const normalizedActor = actor?.trim().toLowerCase();
  return rows
    .filter((row) => !normalizedActor || row.actor.toLowerCase() === normalizedActor)
    .toSorted((left, right) => right.updatedAt.localeCompare(left.updatedAt))[0];
};

export const getCompletedDigestProgress = async (actor?: string) => {
  const rows = await db.digestProgress.where('status').equals('completed').toArray();
  const normalizedActor = actor?.trim().toLowerCase();
  return rows
    .filter((row) => !normalizedActor || row.actor.toLowerCase() === normalizedActor)
    .toSorted((left, right) => right.updatedAt.localeCompare(left.updatedAt));
};

export const updateDigestProgress = async (
  id: string,
  patch: Partial<Omit<DigestProgressSnapshot, 'createdAt' | 'id'>>
) => {
  await db.digestProgress.update(id, { ...patch, updatedAt: new Date().toISOString() });
};

export const markDigestProgressStatus = async (id: string, status: DigestProgressStatus, error?: string) => {
  await updateDigestProgress(id, { error, status });
};

const cacheRelationshipRows = async (actorDid: Did, accounts: RelationshipAccount[], updatedAt: string) => {
  if (accounts.length === 0) return;

  const existing = await db.relationshipEdges
    .where('id')
    .anyOf(accounts.map((account) => relationshipEdgeId(actorDid, account.relationship, account.did)))
    .toArray();
  const existingById = new Map(existing.map((row) => [row.id, row]));

  await db.relationshipEdges.bulkPut(
    accounts.map((account) => {
      const previous = existingById.get(relationshipEdgeId(actorDid, account.relationship, account.did));
      return relationshipEdgeFromAccount(
        actorDid,
        {
          ...account,
          avatar: account.avatar ?? previous?.avatar,
          displayName: account.displayName ?? previous?.displayName,
          lastPostAt: account.lastPostAt ?? previous?.lastPostAt,
          lastPostUri: account.lastPostUri ?? previous?.lastPostUri
        },
        updatedAt
      );
    })
  );
};

const latestActivities = (activities: AuthorActivity[]) => {
  const latestByDid = new Map<Did, AuthorActivity>();

  for (const activity of activities) {
    const current = latestByDid.get(activity.authorDid);
    if (!current || activity.lastPostAt > current.lastPostAt) {
      latestByDid.set(activity.authorDid, activity);
    }
  }

  return [...latestByDid.values()];
};

const compareRelationshipAccounts = (left: RelationshipAccount, right: RelationshipAccount) => {
  if (left.lastPostAt && right.lastPostAt && left.lastPostAt !== right.lastPostAt) {
    return right.lastPostAt.localeCompare(left.lastPostAt);
  }

  if (left.lastPostAt && !right.lastPostAt) return -1;
  if (!left.lastPostAt && right.lastPostAt) return 1;

  return left.handle.localeCompare(right.handle);
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

const digestProgressOptions = (options: LinkDigestOptions): DigestProgressSnapshot['options'] => {
  return {
    actor: options.actor,
    feedLimit: options.feedLimit,
    limit: options.limit,
    maxPages: options.maxPages,
    minScore: options.minScore,
    minShares: options.minShares,
    refreshFollows: options.refreshFollows,
    since: options.since,
    until: options.until
  };
};
