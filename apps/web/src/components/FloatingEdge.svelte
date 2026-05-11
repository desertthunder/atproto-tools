<script lang="ts">
  import { BaseEdge, getBezierPath, useSvelteFlow, type EdgeProps } from '@xyflow/svelte';

  import { getFloatingEdgeParams } from '$lib/graph/floating-edge';
  import type { SocialGraphEdge, SocialGraphNode } from '$lib/types/social-graph';

  let {
    id,
    interactionWidth,
    label,
    labelStyle,
    markerEnd,
    markerStart,
    source,
    sourcePosition,
    sourceX,
    sourceY,
    style,
    target,
    targetPosition,
    targetX,
    targetY
  }: EdgeProps<SocialGraphEdge> = $props();

  const { getInternalNode } = useSvelteFlow<SocialGraphNode, SocialGraphEdge>();

  const floatingParams = $derived.by(() => {
    const fallbackParams = { sourcePosition, sourceX, sourceY, targetPosition, targetX, targetY };
    const sourceNode = getInternalNode(source);
    const targetNode = getInternalNode(target);

    if (!sourceNode || !targetNode) return fallbackParams;

    return getFloatingEdgeParams(sourceNode, targetNode);
  });

  let [path, labelX, labelY] = $derived(getBezierPath(floatingParams));
</script>

<BaseEdge {id} {path} {labelX} {labelY} {label} {labelStyle} {markerStart} {markerEnd} {interactionWidth} {style} />
