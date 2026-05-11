import type { Edge, Node } from '@xyflow/svelte';

import type { Did } from './api';
import type { GraphFetchLimit } from './db';

export type SocialGraphRelationship = 'origin' | 'following' | 'follower' | 'mutuals' | 'second-hop';
export type SocialGraphEdgeRelationship = 'following' | 'follower' | 'mutuals';
export type SocialGraphFilter = 'all' | 'following' | 'followers' | 'mutuals';
export type SocialGraphAvatarMode = 'avatars' | 'rings';

export type SocialGraphNodeData = {
  [key: string]: unknown;
  avatarMode?: SocialGraphAvatarMode;
  avatarUrl?: string;
  description?: string;
  did: Did;
  displayName: string;
  handle: string;
  name: string;
  relationship: SocialGraphRelationship;
};

export type SocialGraphEdgeData = { [key: string]: unknown; relationship: SocialGraphEdgeRelationship };

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
