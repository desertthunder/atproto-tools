<script lang="ts">
  type Props = { visible?: boolean };

  let { visible = false }: Props = $props();
</script>

{#if visible}
  <div class="pointer-events-none absolute inset-0 z-[5] flex flex-col items-center justify-center gap-4">
    <div class="h-10 w-10 animate-spin rounded-full border-2 border-transparent border-t-blue-500 border-r-blue-500/35">
    </div>
    <div class="font-mono text-xs tracking-[0.08em] text-blue-200/50">Fetching social graph...</div>
  </div>
{/if}
