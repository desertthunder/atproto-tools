<script lang="ts">
  type Props = {
    edges?: number;
    followers?: number;
    following?: number;
    mutuals?: number;
    nodes?: number;
    visible?: boolean;
  };

  let { edges = 0, followers = 0, following = 0, mutuals = 0, nodes = 0, visible = false }: Props = $props();
</script>

{#if visible}
  <aside class="pointer-events-auto absolute right-5 bottom-5 flex w-70 flex-col gap-1.5">
    <div
      class="rounded-md border border-blue-800 bg-black px-3 py-1.5 font-mono text-[11px] whitespace-nowrap text-blue-200/70 shadow-[0_0_18px_rgba(37,99,235,0.12)]">
      <span class="font-medium text-blue-400">{nodes}</span> nodes
    </div>
    <div
      class="rounded-md border border-blue-800 bg-black px-3 py-1.5 font-mono text-[11px] whitespace-nowrap text-blue-200/70 shadow-[0_0_18px_rgba(37,99,235,0.12)]">
      <span class="font-medium text-blue-400">{edges}</span> edges
    </div>
    <div
      class="rounded-md border border-blue-800 bg-black px-3 py-1.5 font-mono text-[11px] whitespace-nowrap text-blue-200/70 shadow-[0_0_18px_rgba(37,99,235,0.12)]">
      <span class="font-medium text-blue-400">{following}</span> following /
      <span class="font-medium text-blue-400">{followers}</span>
      followers / <span class="font-medium text-blue-400">{mutuals}</span> mutual
    </div>
  </aside>
{/if}
