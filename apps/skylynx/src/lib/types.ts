import type { AppBskyActorDefs } from '@atcute/bluesky';
import type { Did } from '@atcute/lexicons';

export type { Did } from '@atcute/lexicons';

export type ActorProfile = AppBskyActorDefs.ProfileViewDetailed;
export type ActorSuggestion = AppBskyActorDefs.ProfileViewBasic;

export type Follow = { did: Did; handle: string; profileUrl: string };
export type GraphRelationship = 'followers' | 'following' | 'mutuals';

export type RelationshipAccount = Follow & {
  avatar?: string;
  displayName?: string;
  lastPostAt?: string;
  lastPostUri?: string;
  relationship: GraphRelationship;
};

export type AuthorActivity = { authorDid: Did; handle: string; lastPostAt: string; lastPostUri: string };

export type ExternalLinkPost = {
  author: string;
  authorDid: Did;
  bookmarkCount: number;
  createdAt?: string;
  description: string;
  externalUri: string;
  indexedAt: string;
  likeCount: number;
  postUri: string;
  repostCount: number;
  sharedAt: string;
  sharedBy: string;
  sharedByDid: Did;
  title: string;
};

export type DigestLink = {
  bookmarkCount: number;
  description: string;
  firstSeen: string;
  lastSeen: string;
  likeCount: number;
  repostCount: number;
  score: number;
  shares: ExternalLinkPost[];
  sharers: string[];
  title: string;
  uri: string;
};

export type LinkDigestOptions = {
  actor: string;
  feedLimit: number;
  limit: number;
  maxPages: number;
  minScore: number;
  minShares: number;
  refreshFollows: boolean;
  since?: string;
  until?: string;
};

export type LinkDigestProgress = {
  completed: number;
  phase: 'idle' | 'resolving' | 'fetching-follows' | 'fetching-feeds' | 'paused' | 'done';
  total: number;
};

export type LinkDigestResult = {
  actor: AppBskyActorDefs.ProfileViewDetailed;
  follows: Follow[];
  links: DigestLink[];
  posts: ExternalLinkPost[];
};

export type LinkDigestStatusEvent =
  | { type: 'resolving-actor'; actor: string }
  | { type: 'actor-resolved'; actor: AppBskyActorDefs.ProfileViewDetailed }
  | { type: 'loading-follows'; actorDid: Did; refresh: boolean }
  | { type: 'follows-loaded'; count: number; source: 'cache' | 'network' }
  | { type: 'fetching-feeds'; completed: number; total: number }
  | { type: 'follow-feed-fetched'; completed: number; follow: Follow; linkCount: number; total: number }
  | { type: 'caching-posts'; count: number }
  | { type: 'digest-ready'; linkCount: number; postCount: number }
  | { type: 'paused'; completed: number; runId: string; total: number }
  | { type: 'done'; result: LinkDigestResult };
