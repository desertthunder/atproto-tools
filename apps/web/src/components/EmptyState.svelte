<script lang="ts">
  type Props = { visible?: boolean };

  let { visible = true }: Props = $props();
</script>

{#if visible}
  <div class="pointer-events-none absolute inset-0 z-[2] flex flex-col items-center justify-center gap-3">
    <div class="grid h-16 w-16 place-items-center border border-blue-950 bg-black/65 text-3xl text-blue-200/30">◎</div>
    <div class="text-base font-medium text-blue-50/35">No graph loaded</div>
    <div class="font-mono text-xs text-blue-200/40">Enter a Bluesky handle and press Load</div>
  </div>
{/if}
