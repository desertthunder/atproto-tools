import { MarkerType } from '@xyflow/svelte';

import { fetchActorProfile, fetchFollowersPage, fetchFollowingPage } from '$lib/api/graph';
import {
  cacheActors,
  cacheFollowers,
  cacheFollowing,
  cacheGraphSnapshot,
  getCachedActorByHandle,
  getCachedActors,
  getGraphSnapshot
} from '$lib/db/database';
import type { Did, ProfileView } from '$lib/types/api';
import type { CachedActor, GraphFetchLimit, GraphSnapshot } from '$lib/types/db';
import type {
  SocialGraph,
  SocialGraphEdge,
  SocialGraphNode,
  SocialGraphNodeData,
  SocialGraphRelationship
} from '$lib/types/social-graph';

export const GRAPH_FETCH_LIMITS = [5, 10, 25, 50] as const;

const DEFAULT_LIMIT = 5 satisfies GraphFetchLimit;

type ActorRecord = CachedActor | ProfileView;
type Relationship = Exclude<SocialGraphRelationship, 'origin'>;

export type GraphLoadProgress = {
  count?: number;
  message: string;
  phase: 'cache' | 'followers' | 'following' | 'profile';
};

export type GraphLoadOptions = {
  actor: string;
  fetch?: typeof globalThis.fetch;
  forceRefresh?: boolean;
  limit?: GraphFetchLimit;
  onProgress?: (progress: GraphLoadProgress) => void;
};

export const loadSocialGraph = async ({
  actor,
  fetch,
  forceRefresh = false,
  limit = DEFAULT_LIMIT,
  onProgress
}: GraphLoadOptions): Promise<SocialGraph> => {
  const normalizedActor = normalizeActorIdentifier(actor);

  if (!forceRefresh) {
    onProgress?.({ message: `Checking ${limit} relationship cache...`, phase: 'cache' });
    const cachedGraph = await loadCachedGraph(normalizedActor, limit);

    if (cachedGraph) return cachedGraph;
  }

  return refreshGraph({ actor: normalizedActor, fetch, limit, onProgress });
};

const loadCachedGraph = async (actor: string, limit: GraphFetchLimit) => {
  const cachedActor = isDid(actor)
    ? await getCachedActors([actor]).then((actors) => actors.get(actor))
    : await getCachedActorByHandle(actor);

  if (!cachedActor) return null;

  const [followingSnapshot, followersSnapshot, mutualSnapshot] = await Promise.all([
    getGraphSnapshot(cachedActor.did, 'following', limit),
    getGraphSnapshot(cachedActor.did, 'followers', limit),
    getGraphSnapshot(cachedActor.did, 'mutuals', limit)
  ]);

  if (!followingSnapshot || !followersSnapshot) return null;

  return buildSocialGraph({
    actor: cachedActor,
    fetchedAt: oldestFetchedAt([followingSnapshot, followersSnapshot, mutualSnapshot]),
    followersDids: followersSnapshot.dids,
    followingDids: followingSnapshot.dids,
    limit,
    mutualDids: mutualSnapshot?.dids,
    source: 'cache'
  });
};

const refreshGraph = async ({
  actor,
  fetch,
  limit,
  onProgress
}: Required<Pick<GraphLoadOptions, 'actor' | 'limit'>> &
  Pick<GraphLoadOptions, 'fetch' | 'onProgress'>): Promise<SocialGraph> => {
  onProgress?.({ message: 'Resolving profile...', phase: 'profile' });
  const profile = await fetchActorProfile({ actor: actor as Did, fetch });
  await cacheActors([profile]);

  const following = await fetchLimitedRelationships({
    actorDid: profile.did,
    cachePage: cacheFollowing,
    fetchPage: (cursor, pageLimit) => fetchFollowingPage({ actor: profile.did, cursor, fetch, limit: pageLimit }),
    kind: 'following',
    limit,
    onProgress
  });

  const followers = await fetchLimitedRelationships({
    actorDid: profile.did,
    cachePage: cacheFollowers,
    fetchPage: (cursor, pageLimit) => fetchFollowersPage({ actor: profile.did, cursor, fetch, limit: pageLimit }),
    kind: 'followers',
    limit,
    onProgress
  });

  const mutualDids = findMutualDids(following.dids, followers.dids);
  const [followingSnapshot, followersSnapshot, mutualSnapshot] = await Promise.all([
    cacheGraphSnapshot({
      actor: profile.did,
      complete: following.complete,
      dids: following.dids,
      kind: 'following',
      limit,
      source: 'bluesky'
    }),
    cacheGraphSnapshot({
      actor: profile.did,
      complete: followers.complete,
      dids: followers.dids,
      kind: 'followers',
      limit,
      source: 'bluesky'
    }),
    cacheGraphSnapshot({
      actor: profile.did,
      complete: following.complete && followers.complete,
      dids: mutualDids,
      kind: 'mutuals',
      limit,
      source: 'derived'
    })
  ]);

  return buildSocialGraph({
    actor: profile,
    fetchedAt: oldestFetchedAt([followingSnapshot, followersSnapshot, mutualSnapshot]),
    followersDids: followers.dids,
    followingDids: following.dids,
    limit,
    mutualDids,
    source: 'network'
  });
};

const fetchLimitedRelationships = async ({
  actorDid,
  cachePage,
  fetchPage,
  kind,
  limit,
  onProgress
}: {
  actorDid: Did;
  cachePage: (actorDid: Did, profiles: ProfileView[]) => Promise<void>;
  fetchPage: (cursor: string | undefined, pageLimit: number) => Promise<{ cursor?: string; items: ProfileView[] }>;
  kind: 'followers' | 'following';
  limit: GraphFetchLimit;
  onProgress?: (progress: GraphLoadProgress) => void;
}) => {
  const dids: Did[] = [];
  let cursor: string | undefined;

  do {
    const remaining = limit - dids.length;
    const page = await fetchPage(cursor, remaining);
    const items = page.items.slice(0, remaining);
    dids.push(...items.map((item) => item.did));
    await cachePage(actorDid, items);
    onProgress?.({
      count: dids.length,
      message: `Fetched ${formatCount(dids.length)} of ${limit} ${kind}...`,
      phase: kind
    });
    cursor = page.cursor;
    await yieldToMain();
  } while (cursor && dids.length < limit);

  return { complete: !cursor, dids };
};

const buildSocialGraph = async ({
  actor,
  fetchedAt,
  followersDids,
  followingDids,
  limit,
  mutualDids,
  source
}: {
  actor: ActorRecord;
  fetchedAt?: string;
  followersDids: Did[];
  followingDids: Did[];
  limit: GraphFetchLimit;
  mutualDids?: Did[];
  source: SocialGraph['source'];
}): Promise<SocialGraph> => {
  const followingSet = new Set(followingDids);
  const followerSet = new Set(followersDids);
  const mutualSet = new Set(mutualDids ?? findMutualDids(followingDids, followersDids));
  const selectedDids = selectRenderableDids({ followerSet, followingSet, mutualSet, limit });
  const actorMap = await getCachedActors(selectedDids);
  const actorData = actorToNodeData(actor, 'origin');
  const nodes: SocialGraphNode[] = [{ id: actor.did, type: 'user', data: actorData, position: { x: 0, y: 0 } }];

  const edges: SocialGraphEdge[] = [];

  for (const did of selectedDids) {
    const cachedActor = actorMap.get(did);
    if (!cachedActor) continue;

    const relationship = getRelationship(did, followingSet, followerSet, mutualSet);
    nodes.push({ id: did, type: 'user', data: actorToNodeData(cachedActor, relationship), position: { x: 0, y: 0 } });
    edges.push(createRelationshipEdge(actor.did, did, relationship));
  }

  const totalMutuals = mutualSet.size;
  const totalRelationshipNodes = followingSet.size + [...followerSet].filter((did) => !followingSet.has(did)).length;

  return {
    actor: actorData,
    edges,
    fetchedAt,
    limit,
    nodes,
    source,
    totalEdges: totalRelationshipNodes,
    totalFollowers: followerSet.size - totalMutuals,
    totalFollowing: followingSet.size - totalMutuals,
    totalMutuals,
    totalNodes: totalRelationshipNodes + 1,
    truncated: nodes.length < totalRelationshipNodes + 1
  };
};

const selectRenderableDids = ({
  followerSet,
  followingSet,
  mutualSet,
  limit
}: {
  followerSet: Set<Did>;
  followingSet: Set<Did>;
  limit: GraphFetchLimit;
  mutualSet: Set<Did>;
}) => {
  const selected = new Set<Did>();
  const cap = Math.max(1, limit * 2);

  addUntilLimit(selected, mutualSet, cap);
  addUntilLimit(selected, followingSet, cap, mutualSet);
  addUntilLimit(selected, followerSet, cap, mutualSet);

  return [...selected];
};

const addUntilLimit = (selected: Set<Did>, dids: Iterable<Did>, limit: number, skip?: Set<Did>) => {
  for (const did of dids) {
    if (selected.size >= limit) return;
    if (skip?.has(did)) continue;

    selected.add(did);
  }
};

const getRelationship = (
  did: Did,
  followingSet: Set<Did>,
  followerSet: Set<Did>,
  mutualSet: Set<Did>
): Relationship => {
  if (mutualSet.has(did)) return 'mutual';
  if (followingSet.has(did)) return 'following';
  if (followerSet.has(did)) return 'follower';
  return 'follower';
};

const createRelationshipEdge = (originDid: Did, did: Did, relationship: Relationship): SocialGraphEdge => {
  const source = relationship === 'follower' ? did : originDid;
  const target = relationship === 'follower' ? originDid : did;

  return {
    id: `${source}->${target}`,
    type: 'floating',
    source,
    target,
    data: { relationship },
    markerEnd: { type: MarkerType.Arrow },
    style: `stroke:${edgeColor(relationship)};stroke-width:${relationship === 'mutual' ? '2' : '1.65'}`,
    animated: relationship === 'mutual'
  };
};

const actorToNodeData = (actor: ActorRecord, relationship: SocialGraphRelationship): SocialGraphNodeData => {
  const name = actor.displayName || actor.handle || actor.did;

  return {
    avatarUrl: actor.avatar,
    description: actor.description,
    displayName: name,
    handle: actor.handle ?? actor.did,
    name,
    relationship
  };
};

const findMutualDids = (followingDids: Did[], followersDids: Did[]) => {
  const followerSet = new Set(followersDids);
  return followingDids.filter((did) => followerSet.has(did));
};

const oldestFetchedAt = (snapshots: Array<GraphSnapshot | undefined>) => {
  const timestamps = snapshots.flatMap((snapshot) => (snapshot ? [snapshot.fetchedAt] : []));
  if (timestamps.length === 0) return undefined;

  return timestamps.sort()[0];
};

const normalizeActorIdentifier = (actor: string) => actor.trim().replace(/^@/, '').toLowerCase();

const isDid = (actor: string): actor is Did => actor.startsWith('did:');

const edgeColor = (relationship: Relationship) => {
  if (relationship === 'mutual') return 'rgb(96 165 250 / 0.95)';
  if (relationship === 'following') return 'rgb(147 197 253 / 0.82)';
  return 'rgb(37 99 235 / 0.86)';
};

const formatCount = (count: number) => new Intl.NumberFormat().format(count);

const yieldToMain = () => new Promise((resolve) => window.setTimeout(resolve, 0));
