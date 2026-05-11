import type { Position, XYPosition } from '@xyflow/svelte';
import type { SimulationLinkDatum, SimulationNodeDatum } from 'd3-force';
import type { Did, ProfileView } from './api';
import type { CachedActor, GraphFetchLimit } from './db';
import type { SocialGraphRelationship } from './social-graph';

export type GraphActorRecord = CachedActor | ProfileView;

export type GraphRelationship = Exclude<SocialGraphRelationship, 'origin'>;

export type GraphLoadProgress = {
  count?: number;
  message: string;
  phase: 'cache' | 'followers' | 'following' | 'profile';
};

export type GraphLoadOptions = {
  actor: string;
  fetch?: typeof globalThis.fetch;
  forceRefresh?: boolean;
  limit?: GraphFetchLimit;
  onProgress?: (progress: GraphLoadProgress) => void;
};

export type GraphRelationshipFetchResult = { complete: boolean; dids: Did[] };

export type FloatingEdgeParams = {
  sourcePosition: Position;
  sourceX: number;
  sourceY: number;
  targetPosition: Position;
  targetX: number;
  targetY: number;
};

export type GraphNodeBox = XYPosition & { height: number; width: number };

export type GraphForceNode = SimulationNodeDatum & { id: string; relationship: SocialGraphRelationship };

export type GraphForceLink = SimulationLinkDatum<GraphForceNode> & {
  relationship: Exclude<SocialGraphRelationship, 'origin'>;
};

export type GraphArcConfig = { end: number; radius: number; start: number };
