import type { Did, ProfileView } from './api';
import type { CachedActor, GraphFetchLimit } from './db';
import type { SocialGraph, SocialGraphEdgeRelationship } from './social-graph';

export type GraphActorRecord = CachedActor | ProfileView;

export type GraphRelationship = SocialGraphEdgeRelationship;

export type RenderedGraphRelationship = { sourceDid: Did; targetDid: Did };

export type GraphLoadProgress = {
  count?: number;
  message: string;
  phase: 'cache' | 'followers' | 'following' | 'profile' | 'rendered-relationships' | 'second-hop';
};

export type GraphLoadOptions = {
  actor: string;
  fetch?: typeof globalThis.fetch;
  forceRefresh?: boolean;
  limit?: GraphFetchLimit;
  onProgress?: (progress: GraphLoadProgress) => void;
};

export type GraphExpandOptions = {
  actor: string;
  fetch?: typeof globalThis.fetch;
  graph: SocialGraph;
  limit?: GraphFetchLimit;
  onProgress?: (progress: GraphLoadProgress) => void;
};

export type GraphRelationshipFetchResult = { complete: boolean; dids: Did[] };

export type GraphArcConfig = { end: number; radius: number; start: number };
