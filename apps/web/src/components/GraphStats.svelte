<script lang="ts">
  type Props = {
    edges?: number;
    followers?: number;
    following?: number;
    mutuals?: number;
    nodes?: number;
    truncated?: boolean;
    visible?: boolean;
  };

  let {
    edges = 0,
    followers = 0,
    following = 0,
    mutuals = 0,
    nodes = 0,
    truncated = false,
    visible = false
  }: Props = $props();

  const formatCount = (count: number) => new Intl.NumberFormat().format(count);
</script>

{#if visible}
  <aside class="pointer-events-auto absolute right-5 bottom-5 flex w-70 flex-col gap-1.5">
    <div
      class="rounded-md border border-blue-800 bg-black px-3 py-1.5 font-mono text-[11px] whitespace-nowrap text-blue-200/70 shadow-[0_0_18px_rgba(37,99,235,0.12)]">
      <span class="font-medium text-blue-400">{formatCount(nodes)}</span> nodes
      {#if truncated}
        <span class="text-blue-200/35">cached</span>
      {/if}
    </div>
    <div
      class="rounded-md border border-blue-800 bg-black px-3 py-1.5 font-mono text-[11px] whitespace-nowrap text-blue-200/70 shadow-[0_0_18px_rgba(37,99,235,0.12)]">
      <span class="font-medium text-blue-400">{formatCount(edges)}</span> edges
    </div>
    <div
      class="rounded-md border border-blue-800 bg-black px-3 py-1.5 font-mono text-[11px] whitespace-nowrap text-blue-200/70 shadow-[0_0_18px_rgba(37,99,235,0.12)]">
      <span class="font-medium text-blue-400">{formatCount(following)}</span> following /
      <span class="font-medium text-blue-400">{formatCount(followers)}</span>
      followers / <span class="font-medium text-blue-400">{formatCount(mutuals)}</span> mutual
    </div>
  </aside>
{/if}
