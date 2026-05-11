<script lang="ts">
  import { SOCIAL_GRAPH_COLORS } from '$lib/graph/colors';
  import { layoutSocialGraph } from '$lib/graph/layout';
  import { getSocialGraphStats } from '$lib/graph/stats';
  import type {
    SocialGraph,
    SocialGraphAvatarMode,
    SocialGraphEdge,
    SocialGraphFilter,
    SocialGraphNode,
    SocialGraphNodeData,
    SocialGraphStats
  } from '$lib/types/social-graph';
  import { Background, BackgroundVariant, MiniMap, SvelteFlow, type EdgeTypes, type NodeTypes } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';
  import FloatingEdge from './FloatingEdge.svelte';
  import OriginControls from './OriginControls.svelte';
  import UserNode from './UserNode.svelte';

  type Props = {
    activeFilter?: SocialGraphFilter;
    avatarMode?: SocialGraphAvatarMode;
    graph?: SocialGraph | null;
    loaded?: boolean;
    onNodeSelect?: (data: SocialGraphNodeData) => void;
    onStatsChange?: (stats: SocialGraphStats) => void;
  };

  let {
    activeFilter = 'all',
    avatarMode = 'rings',
    graph = null,
    loaded = false,
    onNodeSelect,
    onStatsChange
  }: Props = $props();

  let nodes = $state.raw<SocialGraphNode[]>([]);
  let edges = $state.raw<SocialGraphEdge[]>([]);
  let layoutedNodes = $state.raw<SocialGraphNode[]>([]);
  let layoutRun = 0;

  const nodeTypes: NodeTypes = { user: UserNode };
  const edgeTypes: EdgeTypes = { floating: FloatingEdge };

  $effect(() => {
    if (!loaded || !graph) {
      layoutRun += 1;
      nodes = [];
      edges = [];
      return;
    }

    const currentRun = (layoutRun += 1);

    layoutSocialGraph(graph.nodes, graph.edges).then((nextNodes) => {
      if (currentRun !== layoutRun) return;

      layoutedNodes = nextNodes;
      setVisibleGraph(nextNodes, graph.edges, activeFilter, avatarMode);
    });
  });

  $effect(() => {
    if (!loaded || !graph || layoutedNodes.length === 0) return;

    setVisibleGraph(layoutedNodes, graph.edges, activeFilter, avatarMode);
  });

  const nodeColor = (node: { data: Record<string, unknown> }) => {
    if (node.data.relationship === 'origin') return SOCIAL_GRAPH_COLORS.origin;
    if (node.data.relationship === 'mutuals') return SOCIAL_GRAPH_COLORS.mutuals;
    if (node.data.relationship === 'following') return SOCIAL_GRAPH_COLORS.following;
    return SOCIAL_GRAPH_COLORS.follower;
  };

  const handleNodeClick = ({ node }: { node: SocialGraphNode }) => {
    onNodeSelect?.(node.data);
  };

  const setVisibleGraph = (
    sourceNodes: SocialGraphNode[],
    sourceEdges: SocialGraphEdge[],
    filter: SocialGraphFilter,
    mode: SocialGraphAvatarMode
  ) => {
    const nextNodes = filterNodes(sourceNodes, filter, mode);
    const nextEdges = filterEdges(sourceEdges, sourceNodes, filter);

    nodes = nextNodes;
    edges = nextEdges;
    onStatsChange?.(getSocialGraphStats(nextNodes, nextEdges, graph ?? undefined));
  };

  const filterNodes = (sourceNodes: SocialGraphNode[], filter: SocialGraphFilter, mode: SocialGraphAvatarMode) => {
    return sourceNodes
      .filter((node) => shouldShowRelationship(node.data.relationship, filter))
      .map((node) => ({ ...node, data: { ...node.data, avatarMode: mode } }));
  };

  const filterEdges = (sourceEdges: SocialGraphEdge[], sourceNodes: SocialGraphNode[], filter: SocialGraphFilter) => {
    const visibleIds = new Set(
      sourceNodes.filter((node) => shouldShowRelationship(node.data.relationship, filter)).map((node) => node.id)
    );

    return sourceEdges.filter((edge) => {
      return visibleIds.has(edge.source) && visibleIds.has(edge.target) && shouldShowEdge(edge, filter);
    });
  };

  const shouldShowEdge = (edge: SocialGraphEdge, filter: SocialGraphFilter) => {
    if (filter === 'all') return true;
    if (filter === 'followers') return edge.data?.relationship === 'follower';
    return edge.data?.relationship === filter;
  };

  const shouldShowRelationship = (relationship: SocialGraphNodeData['relationship'], filter: SocialGraphFilter) => {
    if (relationship === 'origin' || filter === 'all') return true;
    if (filter === 'followers') return relationship === 'follower';
    return relationship === filter;
  };
</script>

<div class="absolute inset-0 overflow-hidden bg-black text-blue-50">
  <div
    class="absolute inset-0 bg-[radial-gradient(circle_at_48%_42%,rgba(29,78,216,0.26),transparent_30%),radial-gradient(circle_at_78%_22%,rgba(37,99,235,0.14),transparent_22%),linear-gradient(180deg,#020617_0%,#000_44%,#020617_100%)]">
  </div>
  <div
    class="absolute inset-0 bg-[linear-gradient(rgba(37,99,235,0.095)_1px,transparent_1px),linear-gradient(90deg,rgba(37,99,235,0.075)_1px,transparent_1px)] bg-size-[56px_56px] opacity-55">
  </div>
  <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_35%,rgba(0,0,0,0.92)_100%)]"></div>
  <div class="absolute inset-x-0 top-0 h-px bg-blue-500/60"></div>

  {#if loaded}
    <SvelteFlow
      bind:nodes
      bind:edges
      {edgeTypes}
      {nodeTypes}
      fitView
      fitViewOptions={{ padding: 0.28, minZoom: 0.55, maxZoom: 1.05 }}
      nodeOrigin={[0.5, 0.5]}
      minZoom={0.3}
      maxZoom={1.6}
      nodesConnectable={false}
      defaultMarkerColor={SOCIAL_GRAPH_COLORS.origin}
      noDragClass="nodrag"
      noWheelClass="nowheel"
      onnodeclick={handleNodeClick}
      colorMode="dark"
      colorModeSSR="dark"
      class="social-flow">
      <Background
        variant={BackgroundVariant.Lines}
        gap={56}
        lineWidth={1}
        bgColor="rgb(0 0 0)"
        patternColor="rgb(37 99 235 / 0.18)" />
      <OriginControls />
      <MiniMap
        position="bottom-right"
        width={140}
        height={86}
        class="graph-minimap"
        {nodeColor}
        nodeStrokeColor={nodeColor}
        nodeBorderRadius={8}
        bgColor="rgb(0 0 0)"
        maskColor="rgb(15 23 42 / 0.62)"
        pannable
        zoomable />
    </SvelteFlow>
  {/if}
</div>

<style>
  :global(.social-flow) {
    --xy-background-color: rgb(0 0 0);
    --xy-background-color-props: rgb(0 0 0);
    --xy-minimap-background-color: rgb(0 0 0);
    --xy-minimap-mask-background-color: rgb(15 23 42 / 0.62);
    --xy-node-background-color: rgb(0 0 0);
    --xy-node-color: rgb(239 246 255);
    --xy-controls-button-background-color: rgb(0 0 0);
    --xy-controls-button-background-color-hover: rgb(23 37 84);
    --xy-controls-button-border-color: rgb(30 64 175);
    --xy-controls-button-color: rgb(147 197 253);
    --xy-controls-button-color-hover: rgb(239 246 255);
    --xy-edge-label-background-color: rgb(0 0 0);
    background: rgb(0 0 0) !important;
  }

  :global(.social-flow .svelte-flow__pane),
  :global(.social-flow .svelte-flow__renderer),
  :global(.social-flow .svelte-flow__viewport),
  :global(.social-flow .svelte-flow__nodes),
  :global(.social-flow .svelte-flow__edges) {
    background: transparent !important;
  }

  :global(.social-flow .svelte-flow__background) {
    background-color: rgb(0 0 0) !important;
  }

  :global(.social-flow .svelte-flow__attribution) {
    display: none;
  }

  :global(.social-flow .svelte-flow__edge-path) {
    stroke-linecap: round;
    filter: drop-shadow(0 0 5px rgb(37 99 235 / 0.3));
  }

  :global(.social-flow .svelte-flow__controls) {
    overflow: hidden;
    border: 1px solid rgb(37 99 235);
    border-radius: 8px;
    background: rgb(0 0 0);
    box-shadow:
      0 0 0 1px rgb(0 0 0),
      0 16px 40px rgb(0 0 0 / 0.45),
      0 0 22px rgb(37 99 235 / 0.22);
  }

  :global(.social-flow .svelte-flow__controls-button) {
    border-bottom-color: rgb(30 64 175);
  }

  :global(.social-flow .svelte-flow__minimap) {
    overflow: hidden;
    border: 1px solid rgb(37 99 235);
    border-radius: 8px;
    box-shadow:
      0 0 0 1px rgb(0 0 0),
      0 16px 40px rgb(0 0 0 / 0.45),
      0 0 22px rgb(37 99 235 / 0.2);
  }

  :global(.social-flow .graph-minimap) {
    right: 1.25rem !important;
    bottom: 9.75rem !important;
  }
</style>
