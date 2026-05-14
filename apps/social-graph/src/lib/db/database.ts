import Dexie, { type EntityTable } from 'dexie';

import { DB_NAME, DB_VERSION, actorFromProfile, graphSnapshotId, relationshipId } from './schema';

import type { Did, ProfileView } from '$lib/types/api';
import type {
  CachedActor,
  GraphFetchLimit,
  GraphRelationship,
  GraphSnapshot,
  GraphSnapshotKind,
  GraphSnapshotSource
} from '$lib/types/db';

export class AtprotoToolsDb extends Dexie {
  actors!: EntityTable<CachedActor, 'did'>;
  graphSnapshots!: EntityTable<GraphSnapshot, 'id'>;
  relationships!: EntityTable<GraphRelationship, 'id'>;

  constructor() {
    super(DB_NAME);

    this.version(1).stores({
      actors: '&did, handle, updatedAt',
      graphSnapshots: '&id, [actor+kind], actor, kind, fetchedAt',
      relationships: '&id, [sourceDid+targetDid], sourceDid, targetDid, updatedAt'
    });

    this.version(DB_VERSION).stores({
      actors: '&did, handle, updatedAt',
      graphSnapshots: '&id, [actor+kind+limit], actor, kind, limit, fetchedAt',
      relationships: '&id, [sourceDid+targetDid], sourceDid, targetDid, updatedAt'
    });
  }
}

export const db = new AtprotoToolsDb();

export const cacheActors = async (profiles: ProfileView[]) => {
  const updatedAt = new Date().toISOString();
  await db.actors.bulkPut(profiles.map((profile) => actorFromProfile(profile, updatedAt)));
};

export const cacheFollowing = async (sourceDid: Did, profiles: ProfileView[]) => {
  const updatedAt = new Date().toISOString();
  const relationships = profiles.map((profile) => {
    return {
      id: relationshipId(sourceDid, profile.did),
      indexedAt: profile.indexedAt,
      sourceDid,
      targetDid: profile.did,
      updatedAt
    } satisfies GraphRelationship;
  });

  await db.transaction('rw', db.actors, db.relationships, async () => {
    await cacheActors(profiles);
    await db.relationships.bulkPut(relationships);
  });
};

export const cacheFollowers = async (targetDid: Did, profiles: ProfileView[]) => {
  const updatedAt = new Date().toISOString();
  const relationships = profiles.map((profile) => {
    return {
      id: relationshipId(profile.did, targetDid),
      indexedAt: profile.indexedAt,
      sourceDid: profile.did,
      targetDid,
      updatedAt
    } satisfies GraphRelationship;
  });

  await db.transaction('rw', db.actors, db.relationships, async () => {
    await cacheActors(profiles);
    await db.relationships.bulkPut(relationships);
  });
};

export const cacheRelationships = async (relationships: Array<Pick<GraphRelationship, 'sourceDid' | 'targetDid'>>) => {
  if (relationships.length === 0) return;

  const updatedAt = new Date().toISOString();

  await db.relationships.bulkPut(
    relationships.map((relationship) => {
      return {
        id: relationshipId(relationship.sourceDid, relationship.targetDid),
        sourceDid: relationship.sourceDid,
        targetDid: relationship.targetDid,
        updatedAt
      } satisfies GraphRelationship;
    })
  );
};

export const cacheGraphSnapshot = async ({
  actor,
  complete,
  dids,
  kind,
  limit,
  source
}: {
  actor: Did | string;
  complete: boolean;
  dids: Did[];
  kind: GraphSnapshotKind;
  limit: GraphFetchLimit;
  source: GraphSnapshotSource;
}) => {
  const snapshot: GraphSnapshot = {
    actor,
    complete,
    dids,
    fetchedAt: new Date().toISOString(),
    id: graphSnapshotId(actor, kind, limit),
    kind,
    limit,
    source
  };

  await db.graphSnapshots.put(snapshot);
  return snapshot;
};

export const getGraphSnapshot = (actor: Did | string, kind: GraphSnapshotKind, limit: GraphFetchLimit) => {
  return db.graphSnapshots.get(graphSnapshotId(actor, kind, limit));
};

export const getCachedActorByHandle = (handle: string) => {
  return db.actors.where('handle').equals(handle).first();
};

export const getCachedActors = async (dids: Did[]) => {
  const actors = await db.actors.bulkGet(dids);
  return new Map(actors.filter((actor): actor is CachedActor => Boolean(actor)).map((actor) => [actor.did, actor]));
};

export const getCachedRelationshipsBetween = async (sourceDids: Did[], targetDids: Did[]) => {
  if (sourceDids.length === 0 || targetDids.length === 0) return [];

  const targetSet = new Set(targetDids);

  return db.relationships
    .where('sourceDid')
    .anyOf(sourceDids)
    .filter((relationship) => targetSet.has(relationship.targetDid))
    .toArray();
};
