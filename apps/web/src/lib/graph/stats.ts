import type {
  SocialGraph,
  SocialGraphEdge,
  SocialGraphNode,
  SocialGraphRelationship,
  SocialGraphStats
} from '$lib/types/social-graph';

export const getSocialGraphStats = (
  nodes: SocialGraphNode[],
  edges: SocialGraphEdge[],
  graph?: SocialGraph
): SocialGraphStats => {
  return {
    edges: graph?.totalEdges ?? edges.length,
    followers: graph?.totalFollowers ?? count(nodes, 'follower'),
    following: graph?.totalFollowing ?? count(nodes, 'following'),
    mutuals: graph?.totalMutuals ?? count(nodes, 'mutuals'),
    nodes: graph?.totalNodes ?? nodes.length
  };
};

const count = (nodes: SocialGraphNode[], relationship: SocialGraphRelationship) => {
  return nodes.filter((node) => node.data.relationship === relationship).length;
};
