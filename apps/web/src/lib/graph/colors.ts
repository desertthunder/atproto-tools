import type { SocialGraphEdgeRelationship, SocialGraphRelationship } from '$lib/types/social-graph';

export const SOCIAL_GRAPH_COLORS = {
  follower: 'rgb(30 64 175)',
  following: 'rgb(244 63 94)',
  mutuals: 'rgb(16 185 129)',
  origin: 'rgb(14 165 233)',
  'second-hop': 'rgb(99 102 241)'
} satisfies Record<SocialGraphRelationship, string>;

export const SOCIAL_GRAPH_EDGE_COLORS = {
  follower: 'rgb(37 99 235 / 0.86)',
  following: SOCIAL_GRAPH_COLORS.following,
  mutuals: SOCIAL_GRAPH_COLORS.mutuals
} satisfies Record<SocialGraphEdgeRelationship, string>;
