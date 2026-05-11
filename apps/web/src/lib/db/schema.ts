import type { Did, ProfileView } from '$lib/types/api';
import type { CachedActor, GraphSnapshotKind } from '$lib/types/db';

export const DB_NAME = 'atproto-tools-web';
export const DB_VERSION = 1;

export const actorFromProfile = (profile: ProfileView, updatedAt = new Date().toISOString()) => {
  return {
    avatar: profile.avatar,
    description: profile.description,
    did: profile.did,
    displayName: profile.displayName,
    handle: profile.handle,
    indexedAt: profile.indexedAt,
    updatedAt
  } satisfies CachedActor;
};

export const graphSnapshotId = (actor: string, kind: GraphSnapshotKind) => `${actor}:${kind}`;
export const relationshipId = (sourceDid: Did, targetDid: Did) => `${sourceDid}->${targetDid}`;
