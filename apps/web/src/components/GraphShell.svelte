<script lang="ts">
  import ConnectionFilter from './ConnectionFilter.svelte';
  import EmptyState from './EmptyState.svelte';
  import GraphLegend from './GraphLegend.svelte';
  import GraphStats from './GraphStats.svelte';
  import GraphViewport from './GraphViewport.svelte';
  import LoadingOverlay from './LoadingOverlay.svelte';
  import ProfilePanel from './ProfilePanel.svelte';
  import TopBar from './TopBar.svelte';

  import { GRAPH_FETCH_LIMITS, loadSocialGraph } from '$lib/graph/load';
  import type { GraphFetchLimit } from '$lib/types/db';
  import type { SocialGraph, SocialGraphNodeData, SocialGraphStats } from '$lib/types/social-graph';

  type FilterType = 'all' | 'following' | 'followers' | 'mutual';

  let handle = $state('desertthunder.dev');
  let limit = $state<GraphFetchLimit>(5);
  let graph = $state.raw<SocialGraph | null>(null);
  let loaded = $state(false);
  let loading = $state(false);
  let loadingMessage = $state('Fetching social graph...');
  let errorMessage = $state<string | null>(null);
  let filter = $state<FilterType>('all');
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
</script>

<main class="relative h-screen overflow-hidden bg-black text-blue-50">
  <GraphViewport
    {graph}
    {loaded}
    activeFilter={filter}
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
      lastFetchedAt={graph?.fetchedAt}
      source={graph?.source}
      onForceRefresh={() => loadGraph(true)}
      onHandleInput={(value) => (handle = value)}
      onLimitChange={(value) => {
        limit = value;
        loaded = false;
        graph = null;
        selectedUser = null;
      }}
      onLoad={() => loadGraph(false)} />

    <div class="absolute top-15 right-5 flex w-70 flex-col items-end gap-3">
      <ProfilePanel profile={selectedUser} onClose={() => (selectedUser = null)} />
      <ConnectionFilter
        visible={loaded}
        active={filter}
        onSelect={(value) => {
          filter = value;
          selectedUser = null;
        }} />
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
