import type { ActorProfile, Did, ExternalLinkPost, Follow } from '../types';

export const DB_NAME = 'skylynx-link-digest';
export const DB_VERSION = 2;

export type CachedActor = ActorProfile & {
  updatedAt: string;
};

export type FollowEdge = {
  actorDid: Did;
  followedDid: Did;
  followedHandle: string;
  id: string;
  profileUrl: string;
  updatedAt: string;
};

export type ExternalLink = {
  description: string;
  title: string;
  updatedAt: string;
  uri: string;
};

export type LinkShare = ExternalLinkPost & {
  id: string;
  updatedAt: string;
};

export type DigestRun = {
  actor: string;
  actorDid: Did;
  completedAt: string;
  id: string;
  linkCount: number;
  options: Pick<LinkDigestOptionsForStorage, 'feedLimit' | 'maxPages' | 'minScore' | 'minShares' | 'since' | 'until'>;
};

type LinkDigestOptionsForStorage = {
  feedLimit: number;
  maxPages: number;
  minScore: number;
  minShares: number;
  since?: string;
  until?: string;
};

export const actorFromProfile = (profile: ActorProfile, updatedAt = new Date().toISOString()): CachedActor => {
  return { ...profile, updatedAt };
};

export const followEdgeFromFollow = (actorDid: Did, follow: Follow, updatedAt = new Date().toISOString()): FollowEdge => {
  return {
    actorDid,
    followedDid: follow.did,
    followedHandle: follow.handle,
    id: followEdgeId(actorDid, follow.did),
    profileUrl: follow.profileUrl,
    updatedAt
  };
};

export const externalLinkFromPost = (post: ExternalLinkPost, updatedAt = new Date().toISOString()): ExternalLink => {
  return {
    description: post.description,
    title: post.title,
    updatedAt,
    uri: post.externalUri
  };
};

export const linkShareFromPost = (post: ExternalLinkPost, updatedAt = new Date().toISOString()): LinkShare => {
  return {
    ...post,
    id: linkShareId(post.postUri, post.sharedByDid),
    updatedAt
  };
};

export const digestRunId = (actorDid: Did) => `${actorDid}:${Date.now()}`;
export const followEdgeId = (actorDid: Did, followedDid: Did) => `${actorDid}->${followedDid}`;
export const linkShareId = (postUri: string, sharedByDid: Did) => `${postUri}:${sharedByDid}`;
