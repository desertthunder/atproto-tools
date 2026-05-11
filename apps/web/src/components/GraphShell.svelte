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
  import { GRAPH_FETCH_LIMITS, loadSocialGraph } from '$lib/graph/load';
  import { normalizeGraphHandle, socialGraphPath } from '$lib/graph/routes';
  import type { GraphFetchLimit } from '$lib/types/db';
  import type {
    SocialGraph,
    SocialGraphAvatarMode,
    SocialGraphFilter,
    SocialGraphNodeData,
    SocialGraphStats
  } from '$lib/types/social-graph';

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
  let routedHandle = $state<string | null>(null);
  let selectedUser = $state<SocialGraphNodeData | null>(null);
  let stats = $state<SocialGraphStats>({ edges: 0, followers: 0, following: 0, mutuals: 0, nodes: 0 });

  const loadGraph = async (forceRefresh = false) => {
    loading = true;
    loaded = false;
    errorMessage = null;
    loadingMessage = forceRefresh ? 'Refreshing social graph...' : 'Checking graph cache...';
    selectedUser = null;

    try {
      graph = await loadSocialGraph({
        actor: handle,
        forceRefresh,
        limit,
        onProgress: (progress) => {
          loadingMessage = progress.message;
        }
      });

      loaded = true;
    } catch (error) {
      graph = null;
      errorMessage = error instanceof Error ? error.message : 'Unable to load this graph.';
    } finally {
      loading = false;
    }
  };

  const resetGraph = () => {
    loaded = false;
    graph = null;
    selectedUser = null;
    stats = { edges: 0, followers: 0, following: 0, mutuals: 0, nodes: 0 };
  };

  const loadCurrentRoute = async () => {
    const nextHandle = normalizeGraphHandle(initialHandle);

    if (routedHandle === nextHandle) return;

    routedHandle = nextHandle;
    handle = nextHandle;
    resetGraph();

    if (nextHandle) await loadGraph(false);
  };

  const loadFromInput = async () => {
    const nextHandle = normalizeGraphHandle(handle);
    const nextPath = socialGraphPath(nextHandle, activeFilter);

    if (!nextHandle) return;

    if (nextHandle !== routedHandle) {
      await goto(nextPath);
      return;
    }

    await loadGraph(false);
  };

  $effect(() => {
    void loadCurrentRoute();
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
      onForceRefresh={() => loadGraph(true)}
      onHandleInput={(value) => (handle = value)}
      onLimitChange={(value) => {
        limit = value;
        resetGraph();
      }}
      onLoad={() => void loadFromInput()} />

    <div class="absolute top-15 right-5 flex w-70 flex-col items-end gap-3">
      <ProfilePanel profile={selectedUser} {avatarMode} onClose={() => (selectedUser = null)} />
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
