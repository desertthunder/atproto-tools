<script lang="ts">
  import ConnectionFilter from './ConnectionFilter.svelte';
  import EmptyState from './EmptyState.svelte';
  import GraphLegend from './GraphLegend.svelte';
  import GraphStats from './GraphStats.svelte';
  import GraphViewport from './GraphViewport.svelte';
  import LoadingOverlay from './LoadingOverlay.svelte';
  import ProfilePanel from './ProfilePanel.svelte';
  import TopBar from './TopBar.svelte';

  import type { SocialGraphNodeData, SocialGraphStats } from '$lib/types/social-graph';

  type FilterType = 'all' | 'following' | 'followers' | 'mutual';

  let handle = $state('@jay.bsky.social');
  let loaded = $state(false);
  let loading = $state(false);
  let filter = $state<FilterType>('all');
  let selectedUser = $state<SocialGraphNodeData | null>(null);
  let stats = $state<SocialGraphStats>({ edges: 0, followers: 0, following: 0, mutuals: 0, nodes: 0 });

  const loadGraph = () => {
    loading = true;
    loaded = false;
    selectedUser = null;

    window.setTimeout(() => {
      loading = false;
      loaded = true;
    }, 450);
  };
</script>

<main class="relative h-screen overflow-hidden bg-black text-blue-50">
  <GraphViewport
    {loaded}
    activeFilter={filter}
    onNodeSelect={(user) => (selectedUser = user)}
    onStatsChange={(nextStats) => (stats = nextStats)} />
  <EmptyState visible={!loaded && !loading} />
  <LoadingOverlay visible={loading} />

  <div class="pointer-events-none relative z-10 h-screen">
    <TopBar {handle} {loading} onHandleInput={(value) => (handle = value)} onLoad={loadGraph} />

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
      mutuals={stats.mutuals} />
  </div>
</main>
