import type { Did } from './api';

export type GraphSnapshotKind = 'followers' | 'following' | 'mutuals';
export type GraphSnapshotSource = 'bluesky' | 'constellation' | 'derived';

export type CachedActor = {
  avatar?: string;
  description?: string;
  did: Did;
  displayName?: string;
  handle?: string;
  indexedAt?: string;
  updatedAt: string;
};

export type GraphRelationship = { id: string; indexedAt?: string; sourceDid: Did; targetDid: Did; updatedAt: string };

export type GraphSnapshot = {
  actor: Did | string;
  dids: Did[];
  fetchedAt: string;
  id: string;
  kind: GraphSnapshotKind;
  source: GraphSnapshotSource;
};
