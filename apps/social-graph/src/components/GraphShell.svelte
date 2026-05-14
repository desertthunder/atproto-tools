<script lang="ts">
  import ConnectionFilter from './ConnectionFilter.svelte';
  import EmptyState from './EmptyState.svelte';
  import GraphLegend from './GraphLegend.svelte';
  import GraphStats from './GraphStats.svelte';
  import GraphViewport from './GraphViewport.svelte';
  import LoadingOverlay from './LoadingOverlay.svelte';
  import ProfilePanel from './ProfilePanel.svelte';
  import TopBar from './TopBar.svelte';

  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { GRAPH_FETCH_LIMITS, expandSocialGraph, loadSocialGraph } from '$lib/graph/load';
  import {
    normalizeGraphHandle,
    parseSecondHopHandles,
    serializeSecondHopHandles,
    socialGraphPath
  } from '$lib/graph/routes';
  import type { GraphFetchLimit } from '$lib/types/db';
  import type {
    SocialGraph,
    SocialGraphAvatarMode,
    SocialGraphFilter,
    SocialGraphNodeData,
    SocialGraphStats
  } from '$lib/types/social-graph';
  import { untrack } from 'svelte';

  type Props = { activeFilter?: SocialGraphFilter; initialHandle?: string };

  let { activeFilter = 'all', initialHandle = '' }: Props = $props();

  let handle = $state('');
  let limit = $state<GraphFetchLimit>(5);
  let graph = $state.raw<SocialGraph | null>(null);
  let loaded = $state(false);
  let loading = $state(false);
  let loadingMessage = $state('Fetching social graph...');
  let errorMessage = $state<string | null>(null);
  let avatarMode = $state<SocialGraphAvatarMode>('rings');
  let syncedBaseKey = $state('');
  let appliedSecondHopHandles = $state<string[]>([]);
  let syncRun = 0;
  let selectedUser = $state<SocialGraphNodeData | null>(null);
  let stats = $state<SocialGraphStats>({ edges: 0, followers: 0, following: 0, mutuals: 0, nodes: 0 });

  const secondHopHandles = $derived(parseSecondHopHandles(page.url.searchParams.get('hop')));

  const syncCurrentRoute = async (
    nextHandle: string,
    nextSecondHopHandles: string[],
    nextLimit: GraphFetchLimit,
    forceRefresh = false
  ) => {
    const normalizedHandle = normalizeGraphHandle(nextHandle).toLowerCase();
    const nextBaseKey = `${normalizedHandle}:${nextLimit}`;
    const shouldReloadBase =
      forceRefresh ||
      syncedBaseKey !== nextBaseKey ||
      !graph ||
      !loaded ||
      !isSecondHopSubset(appliedSecondHopHandles, nextSecondHopHandles);

    syncRun += 1;
    const currentRun = syncRun;

    if (!normalizedHandle) {
      handle = '';
      syncedBaseKey = '';
      appliedSecondHopHandles = [];
      resetGraph();
      return;
    }

    loading = true;
    errorMessage = null;
    let nextGraph = graph;

    try {
      if (shouldReloadBase) {
        loaded = false;
        selectedUser = null;
        loadingMessage = forceRefresh ? 'Refreshing social graph...' : 'Checking graph cache...';
        nextGraph = await loadSocialGraph({
          actor: normalizedHandle,
          forceRefresh,
          limit: nextLimit,
          onProgress: (progress) => {
            loadingMessage = progress.message;
          }
        });

        if (currentRun !== syncRun) return;

        graph = nextGraph;
        syncedBaseKey = nextBaseKey;
        appliedSecondHopHandles = [];
      }

      if (!nextGraph) return;

      nextGraph = await applySecondHops(nextGraph, nextSecondHopHandles, currentRun, nextLimit);
      if (currentRun !== syncRun) return;

      graph = nextGraph;
      handle = normalizedHandle;
      loaded = true;
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Unable to load this graph.';
    } finally {
      if (currentRun === syncRun) loading = false;
    }
  };

  const resetGraph = () => {
    loaded = false;
    graph = null;
    selectedUser = null;
    stats = { edges: 0, followers: 0, following: 0, mutuals: 0, nodes: 0 };
  };

  const applySecondHops = async (
    sourceGraph: SocialGraph,
    nextSecondHopHandles: string[],
    currentRun: number,
    nextLimit: GraphFetchLimit
  ) => {
    let nextGraph = sourceGraph;
    let appliedHandles = [...appliedSecondHopHandles];
    const originHandle = normalizeGraphHandle(nextGraph.actor.handle).toLowerCase();

    for (const secondHopHandle of nextSecondHopHandles) {
      if (currentRun !== syncRun) return nextGraph;
      if (secondHopHandle === originHandle || appliedHandles.includes(secondHopHandle)) continue;

      loadingMessage = `Fetching second hop for @${secondHopHandle}...`;
      nextGraph = await expandSocialGraph({
        actor: secondHopHandle,
        graph: nextGraph,
        limit: nextLimit,
        onProgress: (progress) => {
          loadingMessage = progress.message;
        }
      });

      appliedHandles = [...appliedHandles, secondHopHandle];
      appliedSecondHopHandles = appliedHandles;
      graph = nextGraph;
    }

    return nextGraph;
  };

  const loadFromInput = async () => {
    const nextHandle = normalizeGraphHandle(handle);
    const nextPath = socialGraphPath(nextHandle, activeFilter);

    if (!nextHandle) return;

    if (nextHandle.toLowerCase() !== normalizeGraphHandle(initialHandle).toLowerCase()) {
      await goto(nextPath);
      return;
    }

    await syncCurrentRoute(nextHandle, secondHopHandles, limit);
  };

  const addSecondHop = async (profile: SocialGraphNodeData) => {
    if (profile.relationship === 'origin') return;

    const nextHandle = normalizeGraphHandle(profile.handle).toLowerCase();
    const handles = parseSecondHopHandles(page.url.searchParams.get('hop'));
    if (handles.includes(nextHandle)) return;

    await goto(secondHopUrl([...handles, nextHandle]), { keepFocus: true, noScroll: true });
  };

  const secondHopUrl = (handles: string[]) => {
    const url = new URL(page.url);
    const value = serializeSecondHopHandles(handles);
    const params = [...url.searchParams.entries()]
      .filter(([key]) => key !== 'hop')
      .map(([key, paramValue]) => `${encodeURIComponent(key)}=${encodeURIComponent(paramValue)}`);

    if (value) params.push(`hop=${value}`);

    const query = params.join('&');
    return `${url.pathname}${query ? `?${query}` : ''}${url.hash}`;
  };

  const isSecondHopSubset = (appliedHandles: string[], nextHandles: string[]) => {
    return appliedHandles.every((handle) => nextHandles.includes(handle));
  };

  $effect(() => {
    const nextHandle = initialHandle;
    const nextLimit = limit;
    const nextSecondHopHandles = secondHopHandles;

    untrack(() => {
      void syncCurrentRoute(nextHandle, nextSecondHopHandles, nextLimit);
    });
  });
</script>

<main class="relative h-screen overflow-hidden bg-black text-blue-50">
  <GraphViewport
    {graph}
    {loaded}
    {activeFilter}
    {avatarMode}
    onNodeSelect={(user) => (selectedUser = user)}
    onStatsChange={(nextStats) => (stats = nextStats)} />
  <EmptyState
    visible={!loaded && !loading}
    title={errorMessage ? 'Graph unavailable' : 'No graph loaded'}
    subtitle={errorMessage ?? 'Enter a Bluesky handle and press Load'} />
  <LoadingOverlay visible={loading} message={loadingMessage} />

  <div class="pointer-events-none relative z-10 h-screen">
    <TopBar
      {handle}
      {limit}
      limits={GRAPH_FETCH_LIMITS}
      {loading}
      {avatarMode}
      lastFetchedAt={graph?.fetchedAt}
      source={graph?.source}
      onAvatarModeChange={(mode) => (avatarMode = mode)}
      onForceRefresh={() => void syncCurrentRoute(initialHandle, secondHopHandles, limit, true)}
      onHandleInput={(value) => (handle = value)}
      onLimitChange={(value) => {
        limit = value;
      }}
      onLoad={() => void loadFromInput()} />

    <div class="absolute top-15 right-5 flex w-70 flex-col items-end gap-3">
      <ProfilePanel
        profile={selectedUser}
        {avatarMode}
        {loading}
        {secondHopHandles}
        onClose={() => (selectedUser = null)}
        onFetchSecondHop={(profile) => void addSecondHop(profile)} />
      <ConnectionFilter visible={loaded} active={activeFilter} {handle} onSelect={() => (selectedUser = null)} />
    </div>

    <GraphLegend visible={loaded} />
    <GraphStats
      visible={loaded}
      nodes={stats.nodes}
      edges={stats.edges}
      following={stats.following}
      followers={stats.followers}
      mutuals={stats.mutuals}
      truncated={graph?.truncated} />
  </div>
</main>
