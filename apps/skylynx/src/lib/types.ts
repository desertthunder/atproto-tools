import type { AppBskyActorDefs } from '@atcute/bluesky';
import type { Did } from '@atcute/lexicons';

export type Follow = { did: Did; handle: string; profileUrl: string };

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
  phase: 'idle' | 'resolving' | 'fetching-follows' | 'fetching-feeds' | 'done';
  total: number;
};

export type LinkDigestResult = {
  actor: AppBskyActorDefs.ProfileViewDetailed;
  follows: Follow[];
  links: DigestLink[];
  posts: ExternalLinkPost[];
};
