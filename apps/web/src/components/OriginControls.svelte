<script lang="ts">
  import { ControlButton, Controls, useSvelteFlow } from '@xyflow/svelte';

  import type { SocialGraphEdge, SocialGraphNode } from '$lib/types/social-graph';

  const { setCenter } = useSvelteFlow<SocialGraphNode, SocialGraphEdge>();

  const centerOrigin = () => {
    setCenter(0, 0, { duration: 420, zoom: 0.82 });
  };
</script>

<Controls
  position="bottom-left"
  showFitView={false}
  showLock={false}
  buttonBgColor="rgb(0 0 0)"
  buttonBgColorHover="rgb(23 37 84)"
  buttonBorderColor="rgb(37 99 235)"
  buttonColor="rgb(147 197 253)"
  buttonColorHover="rgb(239 246 255)">
  <ControlButton onclick={centerOrigin} title="Center origin" aria-label="Center origin">
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M12 3v4M12 17v4M3 12h4M17 12h4"
        fill="none"
        stroke="currentColor"
        stroke-linecap="round"
        stroke-width="2.2" />
      <circle cx="12" cy="12" r="3.75" fill="none" stroke="currentColor" stroke-width="2.2" />
      <circle cx="12" cy="12" r="1.15" fill="currentColor" />
    </svg>
  </ControlButton>
</Controls>
