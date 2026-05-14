import type {
  ActorProfile,
  AuthorActivity,
  Did,
  ExternalLinkPost,
  Follow,
  GraphRelationship,
  RelationshipAccount
} from '../types';

export const DB_NAME = 'skylynx-link-digest';
export const DB_VERSION = 4;

export type CachedActor = ActorProfile & { updatedAt: string };

export type FollowEdge = {
  actorDid: Did;
  followedDid: Did;
  followedHandle: string;
  id: string;
  profileUrl: string;
  updatedAt: string;
};

export type RelationshipEdge = {
  actorDid: Did;
  avatar?: string;
  displayName?: string;
  id: string;
  lastPostAt?: string;
  lastPostUri?: string;
  profileUrl: string;
  relationship: GraphRelationship;
  subjectDid: Did;
  subjectHandle: string;
  updatedAt: string;
};

export type CachedAuthorActivity = AuthorActivity & { updatedAt: string };

export type ExternalLink = { description: string; title: string; updatedAt: string; uri: string };

export type LinkShare = ExternalLinkPost & { id: string; updatedAt: string };

export type DigestRun = {
  actor: string;
  actorDid: Did;
  completedAt: string;
  id: string;
  linkCount: number;
  options: Pick<LinkDigestOptionsForStorage, 'feedLimit' | 'maxPages' | 'minScore' | 'minShares' | 'since' | 'until'>;
};

export type DigestProgressStatus = 'running' | 'paused' | 'completed' | 'failed';

export type DigestProgressSnapshot = {
  actor: string;
  actorDid?: Did;
  completed: number;
  createdAt: string;
  error?: string;
  follows: Follow[];
  id: string;
  options: LinkDigestOptionsForStorage & { actor: string; limit: number; refreshFollows: boolean };
  phase: 'resolving' | 'fetching-follows' | 'fetching-feeds' | 'caching-posts' | 'done';
  postCount: number;
  posts: ExternalLinkPost[];
  processedFollowDids: Did[];
  status: DigestProgressStatus;
  total: number;
  updatedAt: string;
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

export const followEdgeFromFollow = (
  actorDid: Did,
  follow: Follow,
  updatedAt = new Date().toISOString()
): FollowEdge => {
  return {
    actorDid,
    followedDid: follow.did,
    followedHandle: follow.handle,
    id: followEdgeId(actorDid, follow.did),
    profileUrl: follow.profileUrl,
    updatedAt
  };
};

export const relationshipEdgeFromAccount = (
  actorDid: Did,
  account: RelationshipAccount,
  updatedAt = new Date().toISOString()
): RelationshipEdge => {
  return {
    actorDid,
    avatar: account.avatar,
    displayName: account.displayName,
    id: relationshipEdgeId(actorDid, account.relationship, account.did),
    lastPostAt: account.lastPostAt,
    lastPostUri: account.lastPostUri,
    profileUrl: account.profileUrl,
    relationship: account.relationship,
    subjectDid: account.did,
    subjectHandle: account.handle,
    updatedAt
  };
};

export const externalLinkFromPost = (post: ExternalLinkPost, updatedAt = new Date().toISOString()): ExternalLink => {
  return { description: post.description, title: post.title, updatedAt, uri: post.externalUri };
};

export const linkShareFromPost = (post: ExternalLinkPost, updatedAt = new Date().toISOString()): LinkShare => {
  return { ...post, id: linkShareId(post.postUri, post.sharedByDid), updatedAt };
};

export const authorActivityForStorage = (
  activity: AuthorActivity,
  updatedAt = new Date().toISOString()
): CachedAuthorActivity => {
  return { ...activity, updatedAt };
};

export const accountFromRelationshipEdge = (edge: RelationshipEdge): RelationshipAccount => {
  return {
    avatar: edge.avatar,
    did: edge.subjectDid,
    displayName: edge.displayName,
    handle: edge.subjectHandle,
    lastPostAt: edge.lastPostAt,
    lastPostUri: edge.lastPostUri,
    profileUrl: edge.profileUrl,
    relationship: edge.relationship
  };
};

export const digestRunId = (actorDid: Did) => `${actorDid}:${Date.now()}`;
export const digestProgressId = () => `digest:${Date.now()}:${Math.random().toString(36).slice(2)}`;
export const followEdgeId = (actorDid: Did, followedDid: Did) => `${actorDid}->${followedDid}`;
export const linkShareId = (postUri: string, sharedByDid: Did) => `${postUri}:${sharedByDid}`;
export const relationshipEdgeId = (actorDid: Did, relationship: GraphRelationship, subjectDid: Did) =>
  `${actorDid}:${relationship}:${subjectDid}`;
