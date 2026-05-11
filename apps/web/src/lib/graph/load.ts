import { MarkerType } from '@xyflow/svelte';
import {
  fetchActorProfile,
  fetchFollowersPage,
  fetchFollowingPage,
  fetchRenderedRelationshipsWithConstellation
} from '$lib/api/graph';
import {
  cacheActors,
  cacheFollowers,
  cacheFollowing,
  cacheGraphSnapshot,
  cacheRelationships,
  getCachedActorByHandle,
  getCachedActors,
  getCachedRelationshipsBetween,
  getGraphSnapshot
} from '$lib/db/database';
import type { Did, ProfileView } from '$lib/types/api';
import type { GraphFetchLimit } from '$lib/types/db';
import type {
  GraphActorRecord,
  GraphExpandOptions,
  GraphLoadOptions,
  GraphLoadProgress,
  GraphRelationship,
  GraphRelationshipFetchResult,
  RenderedGraphRelationship
} from '$lib/types/graph';
import type {
  SocialGraph,
  SocialGraphEdge,
  SocialGraphNode,
  SocialGraphNodeData,
  SocialGraphRelationship
} from '$lib/types/social-graph';
import { SOCIAL_GRAPH_EDGE_COLORS } from './colors';

export const GRAPH_FETCH_LIMITS = [3, 5, 10] as const;

const DEFAULT_LIMIT = 5 satisfies GraphFetchLimit;
type ResolvedGraphActor = { avatar?: string; description?: string; did: Did; displayName?: string; handle?: string };

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

export const expandSocialGraph = async ({
  actor,
  fetch,
  graph,
  limit = graph.limit,
  onProgress
}: GraphExpandOptions): Promise<SocialGraph> => {
  onProgress?.({ message: 'Checking second hop cache...', phase: 'second-hop' });

  const resolvedActor = await resolveExpandedActor({ actor, fetch, graph });
  const relationships = await loadRelationshipDidsForActor({ actorDid: resolvedActor.did, fetch, limit, onProgress });
  const expanded = await mergeSecondHopGraph({ actor: resolvedActor, graph, limit, relationships });

  return loadExpandedRenderedRelationships({
    expandedNodeIds: expanded.expandedNodeIds,
    fetch,
    graph: expanded.graph,
    onProgress
  });
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

const resolveExpandedActor = async ({
  actor,
  fetch,
  graph
}: {
  actor: string;
  fetch?: typeof globalThis.fetch;
  graph: SocialGraph;
}): Promise<ResolvedGraphActor> => {
  const normalizedActor = normalizeActorIdentifier(actor);
  const graphNode = graph.nodes.find((node) => {
    const nodeHandle = normalizeActorIdentifier(node.data.handle);
    return node.data.did === normalizedActor || nodeHandle === normalizedActor;
  });

  if (graphNode) {
    return {
      avatar: graphNode.data.avatarUrl,
      description: graphNode.data.description,
      did: graphNode.data.did,
      displayName: graphNode.data.displayName,
      handle: graphNode.data.handle
    };
  }

  const cachedActor = isDid(normalizedActor)
    ? await getCachedActors([normalizedActor]).then((actors) => actors.get(normalizedActor))
    : await getCachedActorByHandle(normalizedActor);

  if (cachedActor) return cachedActor;

  const profile = await fetchActorProfile({ actor: normalizedActor as Did, fetch });
  await cacheActors([profile]);

  return profile;
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
    fetch,
    followersDids: followers.dids,
    followingDids: following.dids,
    limit,
    mutualDids,
    onProgress,
    source: 'network'
  });
};

const loadRelationshipDidsForActor = async ({
  actorDid,
  fetch,
  limit,
  onProgress
}: {
  actorDid: Did;
  fetch?: typeof globalThis.fetch;
  limit: GraphFetchLimit;
  onProgress?: (progress: GraphLoadProgress) => void;
}) => {
  const [followingSnapshot, followersSnapshot, mutualSnapshot] = await Promise.all([
    getGraphSnapshot(actorDid, 'following', limit),
    getGraphSnapshot(actorDid, 'followers', limit),
    getGraphSnapshot(actorDid, 'mutuals', limit)
  ]);

  if (followingSnapshot && followersSnapshot) {
    return {
      fetchedAt: oldestFetchedAt([followingSnapshot, followersSnapshot, mutualSnapshot]),
      followersDids: followersSnapshot.dids,
      followingDids: followingSnapshot.dids,
      mutualDids: mutualSnapshot?.dids ?? findMutualDids(followingSnapshot.dids, followersSnapshot.dids)
    };
  }

  const following = await fetchLimitedRelationships({
    actorDid,
    cachePage: cacheFollowing,
    fetchPage: (cursor, pageLimit) => fetchFollowingPage({ actor: actorDid, cursor, fetch, limit: pageLimit }),
    kind: 'following',
    limit,
    onProgress
  });

  const followers = await fetchLimitedRelationships({
    actorDid,
    cachePage: cacheFollowers,
    fetchPage: (cursor, pageLimit) => fetchFollowersPage({ actor: actorDid, cursor, fetch, limit: pageLimit }),
    kind: 'followers',
    limit,
    onProgress
  });

  const mutualDids = findMutualDids(following.dids, followers.dids);
  const [nextFollowingSnapshot, nextFollowersSnapshot, nextMutualSnapshot] = await Promise.all([
    cacheGraphSnapshot({
      actor: actorDid,
      complete: following.complete,
      dids: following.dids,
      kind: 'following',
      limit,
      source: 'bluesky'
    }),
    cacheGraphSnapshot({
      actor: actorDid,
      complete: followers.complete,
      dids: followers.dids,
      kind: 'followers',
      limit,
      source: 'bluesky'
    }),
    cacheGraphSnapshot({
      actor: actorDid,
      complete: following.complete && followers.complete,
      dids: mutualDids,
      kind: 'mutuals',
      limit,
      source: 'derived'
    })
  ]);

  return {
    fetchedAt: oldestFetchedAt([nextFollowingSnapshot, nextFollowersSnapshot, nextMutualSnapshot]),
    followersDids: followers.dids,
    followingDids: following.dids,
    mutualDids
  };
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
}): Promise<GraphRelationshipFetchResult> => {
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
  fetch,
  followersDids,
  followingDids,
  limit,
  mutualDids,
  onProgress,
  source
}: {
  actor: GraphActorRecord;
  fetchedAt?: string;
  fetch?: typeof globalThis.fetch;
  followersDids: Did[];
  followingDids: Did[];
  limit: GraphFetchLimit;
  mutualDids?: Did[];
  onProgress?: (progress: GraphLoadProgress) => void;
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

  const renderedRelationships = await loadRenderedRelationships({ fetch, onProgress, selectedDids, source });
  edges.push(...createRenderedRelationshipEdges(renderedRelationships, edges));

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

const mergeSecondHopGraph = async ({
  actor,
  graph,
  limit,
  relationships
}: {
  actor: ResolvedGraphActor;
  graph: SocialGraph;
  limit: GraphFetchLimit;
  relationships: { fetchedAt?: string; followersDids: Did[]; followingDids: Did[]; mutualDids: Did[] };
}): Promise<{ expandedNodeIds: Did[]; graph: SocialGraph }> => {
  const actorDid = actor.did;
  const followingSet = new Set(relationships.followingDids);
  const followerSet = new Set(relationships.followersDids);
  const mutualSet = new Set(relationships.mutualDids);
  const selectedDids = selectRenderableDids({ followerSet, followingSet, mutualSet, limit }).filter(
    (did) => did !== actorDid
  );
  const existingNodeIds = new Set(graph.nodes.map((node) => node.id));
  const actorMap = await getCachedActors(selectedDids);
  const sourceNode = graph.nodes.find((node) => node.id === actorDid);
  const sourcePosition = sourceNode?.position ?? { x: 0, y: 0 };
  const nodes = [...graph.nodes];
  const edgeTargets: Did[] = [];
  const expandedNodeIds: Did[] = [];

  if (!existingNodeIds.has(actorDid)) {
    nodes.push({ id: actorDid, type: 'user', data: actorToNodeData(actor, 'second-hop'), position: sourcePosition });
    existingNodeIds.add(actorDid);
    expandedNodeIds.push(actorDid);
  }

  for (const did of selectedDids) {
    const cachedActor = actorMap.get(did);

    if (existingNodeIds.has(did)) {
      edgeTargets.push(did);
      continue;
    }

    if (!cachedActor) continue;

    nodes.push({
      id: did,
      type: 'user',
      data: actorToNodeData(cachedActor, 'second-hop'),
      position: { x: sourcePosition.x, y: sourcePosition.y }
    });
    existingNodeIds.add(did);
    edgeTargets.push(did);
    expandedNodeIds.push(did);
  }

  const secondHopEdges = edgeTargets.map((did) => {
    return createRelationshipEdge(actorDid, did, getRelationship(did, followingSet, followerSet, mutualSet));
  });
  const edges = mergeSocialGraphEdges(graph.edges, secondHopEdges);
  const fetchedAt = oldestFetchedAt([{ fetchedAt: graph.fetchedAt }, { fetchedAt: relationships.fetchedAt }]);

  return {
    expandedNodeIds,
    graph: {
      ...graph,
      edges,
      fetchedAt,
      limit,
      nodes,
      totalEdges: edges.length,
      totalNodes: nodes.length,
      truncated: graph.truncated || edgeTargets.length < selectedDids.length
    }
  };
};

const loadExpandedRenderedRelationships = async ({
  expandedNodeIds,
  fetch,
  graph,
  onProgress
}: {
  expandedNodeIds: Did[];
  fetch?: typeof globalThis.fetch;
  graph: SocialGraph;
  onProgress?: (progress: GraphLoadProgress) => void;
}) => {
  if (expandedNodeIds.length === 0) return graph;

  const renderedNodeIds = graph.nodes.map((node) => node.id as Did);
  const [cachedIncomingRelationships, cachedOutgoingRelationships] = await Promise.all([
    getCachedRelationshipsBetween(renderedNodeIds, expandedNodeIds),
    getCachedRelationshipsBetween(expandedNodeIds, renderedNodeIds)
  ]);
  const cachedRelationships = [...cachedIncomingRelationships, ...cachedOutgoingRelationships];

  onProgress?.({
    count: expandedNodeIds.length,
    message: `Checking relationships for ${formatCount(expandedNodeIds.length)} second hop nodes...`,
    phase: 'rendered-relationships'
  });

  try {
    const [incomingRelationships, outgoingRelationships] = await Promise.all([
      fetchRenderedRelationshipsWithConstellation({ fetch, sourceDids: renderedNodeIds, targetDids: expandedNodeIds }),
      fetchRenderedRelationshipsWithConstellation({ fetch, sourceDids: expandedNodeIds, targetDids: renderedNodeIds })
    ]);
    const relationships = uniqueRenderedRelationships([...incomingRelationships, ...outgoingRelationships]);
    await cacheRelationships(relationships);
    const edges = mergeSocialGraphEdges(graph.edges, createRenderedRelationshipEdges(relationships, graph.edges));

    return { ...graph, edges, totalEdges: edges.length };
  } catch (error) {
    console.warn('Unable to refresh second hop relationships. Falling back to cached relationships.', error);
    const relationships = uniqueRenderedRelationships(cachedRelationships);
    const edges = mergeSocialGraphEdges(graph.edges, createRenderedRelationshipEdges(relationships, graph.edges));

    return { ...graph, edges, totalEdges: edges.length };
  }
};

const mergeSocialGraphEdges = (existingEdges: SocialGraphEdge[], nextEdges: SocialGraphEdge[]) => {
  const edges = [...existingEdges];
  const edgeIds = new Set(edges.map((edge) => edge.id));

  for (const edge of nextEdges) {
    if (edgeIds.has(edge.id)) continue;

    edges.push(edge);
    edgeIds.add(edge.id);
  }

  return edges;
};

const loadRenderedRelationships = async ({
  fetch,
  onProgress,
  selectedDids,
  source
}: {
  fetch?: typeof globalThis.fetch;
  onProgress?: (progress: GraphLoadProgress) => void;
  selectedDids: Did[];
  source: SocialGraph['source'];
}) => {
  if (selectedDids.length < 2) return [];

  if (source === 'cache') {
    return getCachedRelationshipsBetween(selectedDids, selectedDids);
  }

  onProgress?.({
    count: selectedDids.length,
    message: `Checking relationships between ${formatCount(selectedDids.length)} rendered nodes...`,
    phase: 'rendered-relationships'
  });

  const cachedRelationships = await getCachedRelationshipsBetween(selectedDids, selectedDids);

  try {
    const relationships = await fetchRenderedRelationshipsWithConstellation({
      fetch,
      sourceDids: selectedDids,
      targetDids: selectedDids
    });
    await cacheRelationships(relationships);

    return relationships;
  } catch (error) {
    console.warn('Unable to refresh rendered node relationships. Falling back to cached relationships.', error);
    return cachedRelationships;
  }
};

const createRenderedRelationshipEdges = (
  relationships: RenderedGraphRelationship[],
  existingEdges: SocialGraphEdge[]
): SocialGraphEdge[] => {
  const existingEdgeIds = new Set(existingEdges.map((edge) => edge.id));
  const relationshipIds = new Set(relationships.map((relationship) => relationshipId(relationship)));

  return relationships.flatMap((relationship) => {
    const id = relationshipId(relationship);

    if (existingEdgeIds.has(id)) return [];

    const hasReverseRelationship = relationshipIds.has(reverseRelationshipId(relationship));

    if (hasReverseRelationship && existingEdgeIds.has(reverseRelationshipId(relationship))) return [];
    if (hasReverseRelationship && relationship.sourceDid > relationship.targetDid) return [];

    const relationshipType = hasReverseRelationship ? 'mutuals' : 'following';
    return [createDirectedRelationshipEdge(relationship.sourceDid, relationship.targetDid, relationshipType)];
  });
};

const uniqueRenderedRelationships = (relationships: RenderedGraphRelationship[]) => {
  const relationshipMap = new Map<string, RenderedGraphRelationship>();

  for (const relationship of relationships) {
    relationshipMap.set(relationshipId(relationship), relationship);
  }

  return [...relationshipMap.values()];
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
): GraphRelationship => {
  if (mutualSet.has(did)) return 'mutuals';
  if (followingSet.has(did)) return 'following';
  if (followerSet.has(did)) return 'follower';
  return 'follower';
};

const createRelationshipEdge = (originDid: Did, did: Did, relationship: GraphRelationship): SocialGraphEdge => {
  const source = relationship === 'follower' ? did : originDid;
  const target = relationship === 'follower' ? originDid : did;

  return createDirectedRelationshipEdge(source, target, relationship);
};

const createDirectedRelationshipEdge = (source: Did, target: Did, relationship: GraphRelationship): SocialGraphEdge => {
  return {
    id: `${source}->${target}`,
    type: 'floating',
    source,
    target,
    data: { relationship },
    markerEnd: { type: MarkerType.Arrow, color: edgeColor(relationship) },
    style: `stroke:${edgeColor(relationship)};stroke-width:${relationship === 'mutuals' ? '2' : '1.65'}`,
    animated: relationship === 'mutuals'
  };
};

const relationshipId = (relationship: RenderedGraphRelationship) =>
  `${relationship.sourceDid}->${relationship.targetDid}`;

const reverseRelationshipId = (relationship: RenderedGraphRelationship) =>
  `${relationship.targetDid}->${relationship.sourceDid}`;

const actorToNodeData = (
  actor: GraphActorRecord | ResolvedGraphActor,
  relationship: SocialGraphRelationship
): SocialGraphNodeData => {
  const name = actor.displayName || actor.handle || actor.did;

  return {
    avatarUrl: actor.avatar,
    description: actor.description,
    did: actor.did,
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

const oldestFetchedAt = (snapshots: Array<{ fetchedAt?: string } | undefined>) => {
  const timestamps = snapshots.flatMap((snapshot) => (snapshot?.fetchedAt ? [snapshot.fetchedAt] : []));
  if (timestamps.length === 0) return undefined;

  return timestamps.sort()[0];
};

const normalizeActorIdentifier = (actor: string) => actor.trim().replace(/^@/, '').toLowerCase();

const isDid = (actor: string): actor is Did => actor.startsWith('did:');

const edgeColor = (relationship: GraphRelationship) => {
  return SOCIAL_GRAPH_EDGE_COLORS[relationship];
};

const formatCount = (count: number) => new Intl.NumberFormat().format(count);

const yieldToMain = () => new Promise((resolve) => window.setTimeout(resolve, 0));
