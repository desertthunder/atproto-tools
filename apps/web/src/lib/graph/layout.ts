import {
  forceCenter,
  forceCollide,
  forceLink,
  forceManyBody,
  forceRadial,
  forceSimulation,
  forceX,
  forceY
} from 'd3-force';

import type { GraphArcConfig, GraphForceLink, GraphForceNode } from '$lib/types/graph';
import type { SocialGraphEdge, SocialGraphNode, SocialGraphRelationship } from '$lib/types/social-graph';

export const SOCIAL_NODE_WIDTH = 260;
export const SOCIAL_NODE_HEIGHT = 92;

const NODE_RADIUS = Math.hypot(SOCIAL_NODE_WIDTH, SOCIAL_NODE_HEIGHT) / 2;

const relationshipArcs = {
  follower: { end: 210, radius: 430, start: 150 },
  following: { end: 30, radius: 430, start: -30 },
  mutuals: { end: -52, radius: 345, start: -128 }
} satisfies Record<Exclude<SocialGraphRelationship, 'origin'>, GraphArcConfig>;

const radiusForRelationship = (relationship: SocialGraphRelationship) => {
  if (relationship === 'origin') return 0;
  return relationshipArcs[relationship].radius;
};

const linkDistanceForRelationship = (relationship: GraphForceLink['relationship']) => {
  if (relationship === 'mutuals') return 285;
  return 380;
};

export const layoutSocialGraph = async (nodes: SocialGraphNode[], edges: SocialGraphEdge[]) => {
  const origin = nodes.find((node) => node.data.relationship === 'origin') ?? nodes[0];
  if (!origin) return [];

  const initialPositions = getInitialPositions(nodes, origin.id);
  const forceNodes: GraphForceNode[] = nodes.map((node) => {
    const position = initialPositions.get(node.id) ?? node.position;

    return {
      id: node.id,
      relationship: node.data.relationship,
      x: node.id === origin.id ? 0 : position.x,
      y: node.id === origin.id ? 0 : position.y,
      fx: node.id === origin.id ? 0 : undefined,
      fy: node.id === origin.id ? 0 : undefined
    };
  });

  const forceLinks: GraphForceLink[] = edges.map((edge) => ({
    source: edge.source,
    target: edge.target,
    relationship: edge.data?.relationship ?? 'mutuals'
  }));

  const simulation = forceSimulation(forceNodes)
    .force(
      'link',
      forceLink<GraphForceNode, GraphForceLink>(forceLinks)
        .id((node) => node.id)
        .distance((link) => linkDistanceForRelationship(link.relationship))
        .strength(0.42)
    )
    .force('charge', forceManyBody().strength(-850).distanceMin(150).distanceMax(760))
    .force(
      'collide',
      forceCollide<GraphForceNode>()
        .radius((node) => NODE_RADIUS + (node.relationship === 'origin' ? 44 : 28))
        .strength(0.9)
        .iterations(3)
    )
    .force(
      'relationship-radius',
      forceRadial<GraphForceNode>((node) => radiusForRelationship(node.relationship), 0, 0).strength(0.3)
    )
    .force('relationship-x', forceX<GraphForceNode>((node) => getRelationshipAnchor(node.relationship).x).strength(0.1))
    .force('relationship-y', forceY<GraphForceNode>((node) => getRelationshipAnchor(node.relationship).y).strength(0.1))
    .force('x', forceX<GraphForceNode>(0).strength(0.035))
    .force('y', forceY<GraphForceNode>(0).strength(0.035))
    .force('center', forceCenter(0, 0))
    .stop();

  for (let index = 0; index < 280; index += 1) {
    simulation.tick();
  }

  const positions = new Map(forceNodes.map((node) => [node.id, { x: node.x ?? 0, y: node.y ?? 0 }]));

  return nodes.map((node) => ({ ...node, position: positions.get(node.id) ?? node.position }));
};

const getInitialPositions = (nodes: SocialGraphNode[], originId: string) => {
  const positions = new Map<string, { x: number; y: number }>([[originId, { x: 0, y: 0 }]]);

  for (const relationship of ['follower', 'following', 'mutuals'] as const) {
    const group = nodes.filter((node) => node.id !== originId && node.data.relationship === relationship);
    const arc = relationshipArcs[relationship];

    group.forEach((node, index) => {
      positions.set(node.id, polarToCartesian(angleAt(index, group.length, arc), arc.radius));
    });
  }

  return positions;
};

const getRelationshipAnchor = (relationship: SocialGraphRelationship) => {
  if (relationship === 'origin') return { x: 0, y: 0 };

  const arc = relationshipArcs[relationship];
  return polarToCartesian((arc.start + arc.end) / 2, arc.radius);
};

const angleAt = (index: number, count: number, arc: GraphArcConfig) => {
  if (count <= 1) return (arc.start + arc.end) / 2;

  return arc.start + ((arc.end - arc.start) * index) / (count - 1);
};

const polarToCartesian = (angle: number, radius: number) => {
  const radians = (angle * Math.PI) / 180;

  return { x: Math.cos(radians) * radius, y: Math.sin(radians) * radius };
};
