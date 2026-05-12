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

export type SocialGraphPosition = { x: number; y: number };

export type SocialGraphNode = { data: SocialGraphNodeData; id: Did; position: SocialGraphPosition };

export type SocialGraphEdge = { data: SocialGraphEdgeData; id: string; source: Did; target: Did };

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
