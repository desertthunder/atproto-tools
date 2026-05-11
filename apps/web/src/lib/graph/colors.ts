import type { SocialGraphRelationship } from '$lib/types/social-graph';

export const SOCIAL_GRAPH_COLORS = {
  follower: 'rgb(30 64 175)',
  following: 'rgb(244 63 94)',
  mutual: 'rgb(16 185 129)',
  origin: 'rgb(14 165 233)'
} satisfies Record<SocialGraphRelationship, string>;

export const SOCIAL_GRAPH_EDGE_COLORS = {
  follower: 'rgb(37 99 235 / 0.86)',
  following: SOCIAL_GRAPH_COLORS.following,
  mutual: SOCIAL_GRAPH_COLORS.mutual
} satisfies Record<Exclude<SocialGraphRelationship, 'origin'>, string>;
