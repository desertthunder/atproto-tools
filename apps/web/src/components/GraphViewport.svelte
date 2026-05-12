<script lang="ts">
  import { SOCIAL_NODE_SIZE, SOCIAL_ORIGIN_NODE_SIZE, layoutSocialGraph } from '$lib/graph/layout';
  import { getSocialGraphStats } from '$lib/graph/stats';
  import type {
    SocialGraph,
    SocialGraphAvatarMode,
    SocialGraphEdge,
    SocialGraphEdgeRelationship,
    SocialGraphFilter,
    SocialGraphNode,
    SocialGraphNodeData,
    SocialGraphRelationship,
    SocialGraphStats
  } from '$lib/types/social-graph';
  import { DirectedGraph } from 'graphology';
  import { untrack } from 'svelte';
  import type Sigma from 'sigma';
  import type { EdgeDisplayData, NodeDisplayData } from 'sigma/types';

  type Props = {
    activeFilter?: SocialGraphFilter;
    avatarMode?: SocialGraphAvatarMode;
    graph?: SocialGraph | null;
    loaded?: boolean;
    onNodeSelect?: (data: SocialGraphNodeData | null) => void;
    onStatsChange?: (stats: SocialGraphStats) => void;
  };

  type SigmaNodeAttributes = {
    color: string;
    forceLabel?: boolean;
    highlighted?: boolean;
    label: string;
    profile: SocialGraphNodeData;
    relationship: SocialGraphRelationship;
    size: number;
    x: number;
    y: number;
    zIndex: number;
  };

  type SigmaEdgeAttributes = {
    color: string;
    relationship: SocialGraphEdgeRelationship;
    size: number;
    source: string;
    target: string;
    type: 'arrow';
  };

  type SigmaSocialGraph = DirectedGraph<SigmaNodeAttributes, SigmaEdgeAttributes>;

  const SIGMA_NODE_COLORS = {
    follower: '#1d4ed8',
    following: '#f43f5e',
    mutuals: '#10b981',
    origin: '#0ea5e9',
    'second-hop': '#6366f1'
  } satisfies Record<SocialGraphRelationship, string>;

  const SIGMA_EDGE_COLORS = { follower: '#2563eb', following: '#f43f5e', mutuals: '#10b981' } satisfies Record<
    SocialGraphEdgeRelationship,
    string
  >;

  let {
    activeFilter = 'all',
    avatarMode = 'rings',
    graph = null,
    loaded = false,
    onNodeSelect,
    onStatsChange
  }: Props = $props();

  let container = $state<HTMLDivElement>();
  let rendererReady = $state(false);
  let hoveredNodeId: string | null = null;
  let selectedNodeId: string | null = null;
  let renderer: Sigma<SigmaNodeAttributes, SigmaEdgeAttributes> | null = null;
  let renderedGraph: SigmaSocialGraph = new DirectedGraph<SigmaNodeAttributes, SigmaEdgeAttributes>();

  $effect(() => {
    if (!container) return;

    const currentContainer = container;
    let sigma: Sigma<SigmaNodeAttributes, SigmaEdgeAttributes> | null = null;
    let disposed = false;

    untrack(() => {
      void import('sigma').then(({ default: SigmaRenderer }) => {
        if (disposed) return;

        sigma = new SigmaRenderer<SigmaNodeAttributes, SigmaEdgeAttributes>(renderedGraph, currentContainer, {
          allowInvalidContainer: true,
          defaultEdgeType: 'arrow',
          defaultNodeColor: SIGMA_NODE_COLORS.follower,
          edgeReducer,
          hideEdgesOnMove: true,
          hideLabelsOnMove: true,
          itemSizesReference: 'screen',
          labelColor: { color: 'rgb(219 234 254)' },
          labelDensity: 0.55,
          labelFont: 'Google Sans, Arial, sans-serif',
          labelGridCellSize: 110,
          labelRenderedSizeThreshold: 9,
          labelSize: 11,
          labelWeight: '700',
          maxCameraRatio: 2.2,
          minCameraRatio: 0.18,
          minEdgeThickness: 1.2,
          nodeReducer,
          renderEdgeLabels: false,
          stagePadding: 72,
          zIndex: true
        });

        sigma.on('clickNode', ({ node }) => {
          selectedNodeId = node;
          onNodeSelect?.(sigma?.getGraph().getNodeAttribute(node, 'profile') ?? null);
          sigma?.refresh();
        });
        sigma.on('clickStage', () => {
          selectedNodeId = null;
          onNodeSelect?.(null);
          sigma?.refresh();
        });
        sigma.on('enterNode', ({ node }) => {
          hoveredNodeId = node;
          if (sigma) sigma.getContainer().style.cursor = 'pointer';
          sigma?.refresh();
        });
        sigma.on('leaveNode', () => {
          hoveredNodeId = null;
          if (sigma) sigma.getContainer().style.cursor = '';
          sigma?.refresh();
        });

        renderer = sigma;
        rendererReady = true;
      });
    });

    return () => {
      disposed = true;
      sigma?.kill();
      if (renderer === sigma) renderer = null;
      rendererReady = false;
    };
  });

  $effect(() => {
    const currentRendererReady = rendererReady;
    const currentRenderer = renderer;
    const currentLoaded = loaded;
    const currentGraph = graph;
    const currentFilter = activeFilter;
    const currentAvatarMode = avatarMode;

    if (!currentRenderer || !currentRendererReady) return;

    untrack(() => {
      if (!currentLoaded || !currentGraph) {
        clearGraphSelection();
        const emptyGraph = new DirectedGraph<SigmaNodeAttributes, SigmaEdgeAttributes>();
        renderedGraph = emptyGraph;
        currentRenderer.setGraph(emptyGraph);
        onStatsChange?.({ edges: 0, followers: 0, following: 0, mutuals: 0, nodes: 0 });
        return;
      }

      const { edges, nodes, sigmaGraph } = buildSigmaGraph(currentGraph, currentFilter, currentAvatarMode);
      pruneSelection(sigmaGraph);
      renderedGraph = sigmaGraph;
      currentRenderer.setGraph(sigmaGraph);
      currentRenderer.refresh();
      onStatsChange?.(getSocialGraphStats(nodes, edges, currentGraph));
    });
  });

  const clearGraphSelection = () => {
    if (!selectedNodeId && !hoveredNodeId) return;

    selectedNodeId = null;
    hoveredNodeId = null;
    onNodeSelect?.(null);
  };

  const pruneSelection = (sigmaGraph: SigmaSocialGraph) => {
    if (selectedNodeId && !sigmaGraph.hasNode(selectedNodeId)) {
      selectedNodeId = null;
      onNodeSelect?.(null);
    }

    if (hoveredNodeId && !sigmaGraph.hasNode(hoveredNodeId)) {
      hoveredNodeId = null;
    }
  };

  const buildSigmaGraph = (sourceGraph: SocialGraph, filter: SocialGraphFilter, mode: SocialGraphAvatarMode) => {
    const layoutedNodes = layoutSocialGraph(sourceGraph.nodes, sourceGraph.edges);
    const nodes = filterNodes(layoutedNodes, filter, mode);
    const edges = filterEdges(sourceGraph.edges, nodes);
    const sigmaGraph = new DirectedGraph<SigmaNodeAttributes, SigmaEdgeAttributes>({ allowSelfLoops: false });

    for (const node of nodes) {
      const label = node.data.displayName || `@${node.data.handle.replace(/^@/, '')}`;
      sigmaGraph.addNode(node.id, {
        color: nodeColor(node.data.relationship),
        forceLabel: node.data.relationship === 'origin',
        label,
        profile: node.data,
        relationship: node.data.relationship,
        size: node.data.relationship === 'origin' ? SOCIAL_ORIGIN_NODE_SIZE : SOCIAL_NODE_SIZE,
        x: node.position.x,
        y: node.position.y,
        zIndex: nodeZIndex(node.data.relationship)
      });
    }

    for (const edge of edges) {
      sigmaGraph.addDirectedEdgeWithKey(edge.id, edge.source, edge.target, {
        color: SIGMA_EDGE_COLORS[edge.data.relationship],
        relationship: edge.data.relationship,
        size: edge.data.relationship === 'mutuals' ? 2.35 : 1.65,
        source: edge.source,
        target: edge.target,
        type: 'arrow'
      });
    }

    return { edges, nodes, sigmaGraph };
  };

  const filterNodes = (sourceNodes: SocialGraphNode[], filter: SocialGraphFilter, mode: SocialGraphAvatarMode) => {
    return sourceNodes
      .filter((node) => shouldShowRelationship(node.data.relationship, filter))
      .map((node) => ({ ...node, data: { ...node.data, avatarMode: mode } }));
  };

  const filterEdges = (sourceEdges: SocialGraphEdge[], visibleNodes: SocialGraphNode[]) => {
    const visibleIds = new Set(visibleNodes.map((node) => node.id));

    return sourceEdges.filter((edge) => {
      return visibleIds.has(edge.source) && visibleIds.has(edge.target);
    });
  };

  const shouldShowRelationship = (relationship: SocialGraphNodeData['relationship'], filter: SocialGraphFilter) => {
    if (relationship === 'origin' || filter === 'all') return true;
    if (relationship === 'second-hop') return true;
    if (filter === 'followers') return relationship === 'follower';
    return relationship === filter;
  };

  const nodeReducer = (node: string, data: SigmaNodeAttributes): Partial<NodeDisplayData> => {
    const activeNodeId = selectedNodeId ?? hoveredNodeId;
    const isActive = node === activeNodeId;
    const isAdjacent = activeNodeId ? renderedGraph.areNeighbors(node, activeNodeId) : false;

    if (!activeNodeId) return data;

    if (isActive) {
      return { ...data, color: '#eff6ff', forceLabel: true, highlighted: true, size: data.size * 1.4, zIndex: 20 };
    }

    if (isAdjacent) {
      return {
        ...data,
        forceLabel: data.relationship === 'origin',
        highlighted: true,
        size: data.size * 1.12,
        zIndex: 12
      };
    }

    return { ...data, color: mutedNodeColor(data.relationship), zIndex: 1 };
  };

  const edgeReducer = (_edge: string, data: SigmaEdgeAttributes): Partial<EdgeDisplayData> => {
    const activeNodeId = selectedNodeId ?? hoveredNodeId;

    if (!activeNodeId) return data;
    if (data.source === activeNodeId || data.target === activeNodeId) {
      return { ...data, color: SIGMA_EDGE_COLORS[data.relationship], size: data.size * 1.45, zIndex: 12 };
    }

    return { ...data, color: '#1e3a8a', size: Math.max(1, data.size * 0.72), zIndex: 1 };
  };

  const nodeColor = (relationship: SocialGraphRelationship) => SIGMA_NODE_COLORS[relationship];

  const mutedNodeColor = (relationship: SocialGraphRelationship) => {
    if (relationship === 'following') return '#7f1d1d';
    if (relationship === 'mutuals') return '#064e3b';
    if (relationship === 'second-hop') return '#312e81';
    return '#1e293b';
  };

  const nodeZIndex = (relationship: SocialGraphRelationship) => {
    if (relationship === 'origin') return 10;
    if (relationship === 'mutuals') return 6;
    if (relationship === 'second-hop') return 3;
    return 4;
  };

  const centerOrigin = () => {
    void renderer?.getCamera().animate({ angle: 0, ratio: 0.78, x: 0, y: 0 }, { duration: 420 });
  };

  const fitGraph = () => {
    void renderer?.getCamera().animatedReset({ duration: 420 });
  };

  const zoomIn = () => {
    void renderer?.getCamera().animatedZoom({ duration: 220 });
  };

  const zoomOut = () => {
    void renderer?.getCamera().animatedUnzoom({ duration: 220 });
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

  <div bind:this={container} class="sigma-viewport absolute inset-0" aria-label="Social graph"></div>

  {#if loaded}
    <div
      class="sigma-controls absolute bottom-38 left-5 z-10 flex overflow-hidden rounded-lg border border-blue-600 bg-black">
      <button type="button" title="Center origin" aria-label="Center origin" onclick={centerOrigin}>
        <span class="flex items-center">
          <i class="i-tabler-focus-centered"></i>
        </span>
      </button>
      <button type="button" title="Fit graph" aria-label="Fit graph" onclick={fitGraph}>
        <span class="flex items-center">
          <i class="i-tabler-arrows-maximize"></i>
        </span>
      </button>
      <button type="button" title="Zoom in" aria-label="Zoom in" onclick={zoomIn}>
        <span class="flex items-center">
          <i class="i-tabler-plus"></i>
        </span>
      </button>
      <button type="button" title="Zoom out" aria-label="Zoom out" onclick={zoomOut}>
        <span class="flex items-center">
          <i class="i-tabler-minus"></i>
        </span>
      </button>
    </div>
  {/if}
</div>

<style>
  .sigma-viewport {
    background: transparent;
  }

  .sigma-viewport :global(canvas) {
    background: transparent !important;
  }

  .sigma-controls {
    box-shadow:
      0 0 0 1px rgb(0 0 0),
      0 16px 40px rgb(0 0 0 / 0.45),
      0 0 22px rgb(37 99 235 / 0.22);
  }

  .sigma-controls button {
    display: grid;
    width: 32px;
    height: 32px;
    place-items: center;
    color: rgb(147 197 253);
    background: rgb(0 0 0);
    border-right: 1px solid rgb(30 64 175);
    transition:
      color 120ms ease,
      background 120ms ease;
  }

  .sigma-controls button:last-child {
    border-right: 0;
  }

  .sigma-controls button:hover {
    color: rgb(239 246 255);
    background: rgb(23 37 84);
  }

  .sigma-controls i {
    width: 16px;
    height: 16px;
  }
</style>
