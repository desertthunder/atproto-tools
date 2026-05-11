import {
  forceCenter,
  forceCollide,
  forceLink,
  forceManyBody,
  forceRadial,
  forceSimulation,
  forceX,
  forceY,
  type SimulationLinkDatum,
  type SimulationNodeDatum
} from 'd3-force';

import type { SocialGraphEdge, SocialGraphNode, SocialGraphRelationship } from '$lib/types/social-graph';

export const SOCIAL_NODE_WIDTH = 260;
export const SOCIAL_NODE_HEIGHT = 92;

const NODE_RADIUS = Math.hypot(SOCIAL_NODE_WIDTH, SOCIAL_NODE_HEIGHT) / 2;

type ForceNode = SimulationNodeDatum & { id: string; relationship: SocialGraphRelationship };

type ForceLink = SimulationLinkDatum<ForceNode> & {
  relationship: NonNullable<SocialGraphEdge['data']>['relationship'];
};

type ArcConfig = { end: number; radius: number; start: number };

const relationshipArcs = {
  follower: { end: 210, radius: 430, start: 150 },
  following: { end: 30, radius: 430, start: -30 },
  mutual: { end: -52, radius: 345, start: -128 }
} satisfies Record<Exclude<SocialGraphRelationship, 'origin'>, ArcConfig>;

const radiusForRelationship = (relationship: SocialGraphRelationship) => {
  if (relationship === 'origin') return 0;
  return relationshipArcs[relationship].radius;
};

const linkDistanceForRelationship = (relationship: ForceLink['relationship']) => {
  if (relationship === 'mutual') return 285;
  return 380;
};

export const layoutSocialGraph = async (nodes: SocialGraphNode[], edges: SocialGraphEdge[]) => {
  const origin = nodes.find((node) => node.data.relationship === 'origin') ?? nodes[0];
  if (!origin) return [];

  const initialPositions = getInitialPositions(nodes, origin.id);
  const forceNodes: ForceNode[] = nodes.map((node) => {
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

  const forceLinks: ForceLink[] = edges.map((edge) => ({
    source: edge.source,
    target: edge.target,
    relationship: edge.data?.relationship ?? 'mutual'
  }));

  const simulation = forceSimulation(forceNodes)
    .force(
      'link',
      forceLink<ForceNode, ForceLink>(forceLinks)
        .id((node) => node.id)
        .distance((link) => linkDistanceForRelationship(link.relationship))
        .strength(0.42)
    )
    .force('charge', forceManyBody().strength(-850).distanceMin(150).distanceMax(760))
    .force(
      'collide',
      forceCollide<ForceNode>()
        .radius((node) => NODE_RADIUS + (node.relationship === 'origin' ? 44 : 28))
        .strength(0.9)
        .iterations(3)
    )
    .force(
      'relationship-radius',
      forceRadial<ForceNode>((node) => radiusForRelationship(node.relationship), 0, 0).strength(0.3)
    )
    .force('relationship-x', forceX<ForceNode>((node) => getRelationshipAnchor(node.relationship).x).strength(0.1))
    .force('relationship-y', forceY<ForceNode>((node) => getRelationshipAnchor(node.relationship).y).strength(0.1))
    .force('x', forceX<ForceNode>(0).strength(0.035))
    .force('y', forceY<ForceNode>(0).strength(0.035))
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

  for (const relationship of ['follower', 'following', 'mutual'] as const) {
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

const angleAt = (index: number, count: number, arc: ArcConfig) => {
  if (count <= 1) return (arc.start + arc.end) / 2;

  return arc.start + ((arc.end - arc.start) * index) / (count - 1);
};

const polarToCartesian = (angle: number, radius: number) => {
  const radians = (angle * Math.PI) / 180;

  return { x: Math.cos(radians) * radius, y: Math.sin(radians) * radius };
};
