import { MarkerType } from '@xyflow/svelte';

import type {
  SocialGraphEdge,
  SocialGraphNode,
  SocialGraphRelationship,
  SocialGraphStats
} from '$lib/types/social-graph';

const avatar = (seed: string) => `https://api.dicebear.com/9.x/glass/svg?seed=${encodeURIComponent(seed)}`;

export const sampleSocialNodes: SocialGraphNode[] = [
  {
    id: 'jay',
    type: 'user',
    data: {
      avatarUrl: avatar('jay-graber'),
      description: 'Building decentralized social.',
      displayName: 'Jay Graber',
      handle: 'jay.bsky.social',
      name: 'Jay Graber',
      relationship: 'origin'
    },
    position: { x: 0, y: 0 }
  },
  {
    id: 'pfrazee',
    type: 'user',
    data: {
      avatarUrl: avatar('pfrazee'),
      displayName: 'Paul Frazee',
      handle: 'pfrazee.com',
      name: 'Paul Frazee',
      relationship: 'mutual'
    },
    position: { x: 0, y: 0 }
  },
  {
    id: 'atproto',
    type: 'user',
    data: {
      avatarUrl: avatar('atproto'),
      displayName: 'AT Protocol',
      handle: 'atproto.com',
      name: 'AT Protocol',
      relationship: 'following'
    },
    position: { x: 0, y: 0 }
  },
  {
    id: 'bnewbold',
    type: 'user',
    data: {
      avatarUrl: avatar('bnewbold'),
      displayName: 'Bryan Newbold',
      handle: 'bnewbold.net',
      name: 'Bryan Newbold',
      relationship: 'mutual'
    },
    position: { x: 0, y: 0 }
  },
  {
    id: 'alice',
    type: 'user',
    data: {
      avatarUrl: avatar('alice'),
      displayName: 'Alice Nwachukwu',
      handle: 'alice.bsky.social',
      name: 'Alice Nwachukwu',
      relationship: 'follower'
    },
    position: { x: 0, y: 0 }
  },
  {
    id: 'bo',
    type: 'user',
    data: {
      avatarUrl: avatar('bo-chen'),
      displayName: 'Bo Chen',
      handle: 'bochen.dev',
      name: 'Bo Chen',
      relationship: 'following'
    },
    position: { x: 0, y: 0 }
  },
  {
    id: 'maya',
    type: 'user',
    data: {
      avatarUrl: avatar('maya-patel'),
      displayName: 'Maya Patel',
      handle: 'mayapatel.dev',
      name: 'Maya Patel',
      relationship: 'follower'
    },
    position: { x: 0, y: 0 }
  }
];

export const sampleSocialEdges: SocialGraphEdge[] = [
  edge('jay-pfrazee', 'jay', 'pfrazee', 'mutual'),
  edge('jay-atproto', 'jay', 'atproto', 'following'),
  edge('jay-bnewbold', 'jay', 'bnewbold', 'mutual'),
  edge('alice-jay', 'alice', 'jay', 'follower'),
  edge('jay-bo', 'jay', 'bo', 'following'),
  edge('maya-jay', 'maya', 'jay', 'follower'),
  edge('pfrazee-atproto', 'pfrazee', 'atproto', 'mutual'),
  edge('bnewbold-pfrazee', 'bnewbold', 'pfrazee', 'mutual')
];

export const getSocialGraphStats = (nodes: SocialGraphNode[], edges: SocialGraphEdge[]): SocialGraphStats => {
  return {
    edges: edges.length,
    followers: count(nodes, 'follower'),
    following: count(nodes, 'following'),
    mutuals: count(nodes, 'mutual'),
    nodes: nodes.length
  };
};

function edge(
  id: string,
  source: string,
  target: string,
  relationship: Exclude<SocialGraphRelationship, 'origin'>
): SocialGraphEdge {
  return {
    id,
    type: 'floating',
    source,
    target,
    data: { relationship },
    markerEnd: { type: MarkerType.Arrow },
    style: `stroke:${edgeColor(relationship)};stroke-width:${relationship === 'mutual' ? '2' : '1.65'}`,
    animated: relationship === 'mutual'
  };
}

function edgeColor(relationship: Exclude<SocialGraphRelationship, 'origin'>) {
  if (relationship === 'mutual') return 'rgb(96 165 250 / 0.95)';
  if (relationship === 'following') return 'rgb(147 197 253 / 0.82)';
  return 'rgb(37 99 235 / 0.86)';
}

const count = (nodes: SocialGraphNode[], relationship: SocialGraphRelationship) => {
  return nodes.filter((node) => node.data.relationship === relationship).length;
};
