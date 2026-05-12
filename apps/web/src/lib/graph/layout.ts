import { DirectedGraph } from 'graphology';
import forceAtlas2 from 'graphology-layout-forceatlas2';

import type { Did } from '$lib/types/api';
import type { GraphArcConfig } from '$lib/types/graph';
import type { SocialGraphEdge, SocialGraphNode, SocialGraphRelationship } from '$lib/types/social-graph';

export const SOCIAL_NODE_SIZE = 12;
export const SOCIAL_ORIGIN_NODE_SIZE = 18;

const relationshipArcs = {
  follower: { end: 210, radius: 4.3, start: 150 },
  following: { end: 30, radius: 4.3, start: -30 },
  mutuals: { end: -52, radius: 3.45, start: -128 },
  'second-hop': { end: 330, radius: 5.7, start: 210 }
} satisfies Record<Exclude<SocialGraphRelationship, 'origin'>, GraphArcConfig>;

type LayoutNodeAttributes = {
  fixed?: boolean;
  relationship: SocialGraphRelationship;
  size: number;
  x: number;
  y: number;
};

type LayoutEdgeAttributes = { relationship: SocialGraphEdge['data']['relationship']; weight: number };

export const layoutSocialGraph = (nodes: SocialGraphNode[], edges: SocialGraphEdge[]) => {
  const origin = nodes.find((node) => node.data.relationship === 'origin') ?? nodes[0];
  if (!origin) return [];

  const layoutGraph = buildLayoutGraph(nodes, edges, origin.id);

  forceAtlas2.assign<LayoutNodeAttributes, LayoutEdgeAttributes>(layoutGraph, {
    iterations: iterationsForNodeCount(nodes.length),
    getEdgeWeight: 'weight',
    settings: {
      adjustSizes: true,
      barnesHutOptimize: nodes.length > 60,
      edgeWeightInfluence: 0.35,
      gravity: 0.8,
      scalingRatio: 1.8,
      slowDown: 1.2,
      strongGravityMode: true
    }
  });

  const originX = layoutGraph.getNodeAttribute(origin.id, 'x') ?? 0;
  const originY = layoutGraph.getNodeAttribute(origin.id, 'y') ?? 0;

  return nodes.map((node) => {
    const x = layoutGraph.getNodeAttribute(node.id, 'x') ?? node.position.x;
    const y = layoutGraph.getNodeAttribute(node.id, 'y') ?? node.position.y;

    return {
      ...node,
      position: { x: node.id === origin.id ? 0 : x - originX, y: node.id === origin.id ? 0 : y - originY }
    };
  });
};

const iterationsForNodeCount = (nodeCount: number) => {
  if (nodeCount > 300) return 40;
  if (nodeCount > 150) return 65;
  if (nodeCount > 80) return 95;
  return 140;
};

const buildLayoutGraph = (nodes: SocialGraphNode[], edges: SocialGraphEdge[], originId: Did) => {
  const graph = new DirectedGraph<LayoutNodeAttributes, LayoutEdgeAttributes>({ allowSelfLoops: false });
  const initialPositions = getInitialPositions(nodes, originId);

  for (const node of nodes) {
    const position = initialPositions.get(node.id) ?? node.position;
    graph.addNode(node.id, {
      fixed: node.id === originId,
      relationship: node.data.relationship,
      size: node.data.relationship === 'origin' ? SOCIAL_ORIGIN_NODE_SIZE : SOCIAL_NODE_SIZE,
      x: node.id === originId ? 0 : position.x,
      y: node.id === originId ? 0 : position.y
    });
  }

  for (const edge of edges) {
    if (!graph.hasNode(edge.source) || !graph.hasNode(edge.target) || edge.source === edge.target) continue;

    graph.mergeDirectedEdgeWithKey(edge.id, edge.source, edge.target, {
      relationship: edge.data.relationship,
      weight: edge.data.relationship === 'mutuals' ? 1.7 : 1
    });
  }

  return graph;
};

const getInitialPositions = (nodes: SocialGraphNode[], originId: Did) => {
  const positions = new Map<Did, { x: number; y: number }>([[originId, { x: 0, y: 0 }]]);

  for (const relationship of ['follower', 'following', 'mutuals', 'second-hop'] as const) {
    const group = nodes.filter((node) => node.id !== originId && node.data.relationship === relationship);
    const arc = relationshipArcs[relationship];

    group.forEach((node, index) => {
      positions.set(node.id, polarToCartesian(angleAt(index, group.length, arc), arc.radius));
    });
  }

  return positions;
};

const angleAt = (index: number, count: number, arc: GraphArcConfig) => {
  if (count <= 1) return (arc.start + arc.end) / 2;

  return arc.start + ((arc.end - arc.start) * index) / (count - 1);
};

const polarToCartesian = (angle: number, radius: number) => {
  const radians = (angle * Math.PI) / 180;

  return { x: Math.cos(radians) * radius, y: Math.sin(radians) * radius };
};
