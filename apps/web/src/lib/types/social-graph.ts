import type { Edge, Node } from '@xyflow/svelte';

import type { GraphFetchLimit } from './db';

export type SocialGraphRelationship = 'origin' | 'following' | 'follower' | 'mutual';
export type SocialGraphFilter = 'all' | 'following' | 'followers' | 'mutual';

export type SocialGraphNodeData = {
  [key: string]: unknown;
  avatarUrl?: string;
  description?: string;
  displayName: string;
  handle: string;
  name: string;
  relationship: SocialGraphRelationship;
};

export type SocialGraphEdgeData = { [key: string]: unknown; relationship: Exclude<SocialGraphRelationship, 'origin'> };

export type SocialGraphNode = Node<SocialGraphNodeData, 'user'>;
export type SocialGraphEdge = Edge<SocialGraphEdgeData, 'floating'>;

export type SocialGraphStats = { edges: number; followers: number; following: number; mutuals: number; nodes: number };

export type SocialGraphSource = 'cache' | 'network' | 'sample';

export type SocialGraph = {
  actor: SocialGraphNodeData;
  edges: SocialGraphEdge[];
  fetchedAt?: string;
  limit: GraphFetchLimit;
  nodes: SocialGraphNode[];
  source: SocialGraphSource;
  totalEdges: number;
  totalFollowers: number;
  totalFollowing: number;
  totalMutuals: number;
  totalNodes: number;
  truncated: boolean;
};
