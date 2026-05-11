<script lang="ts">
  type LegendItem = { label: string; tone: string };

  type Props = { visible?: boolean };

  let { visible = false }: Props = $props();

  const items: LegendItem[] = [
    { label: 'Origin', tone: 'bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.85)]' },
    { label: 'Following', tone: 'bg-blue-300 shadow-[0_0_8px_rgba(147,197,253,0.75)]' },
    { label: 'Followers', tone: 'bg-blue-700 shadow-[0_0_8px_rgba(29,78,216,0.8)]' },
    { label: 'Mutual', tone: 'bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,0.75)]' }
  ];
</script>

{#if visible}
  <div
    class="pointer-events-auto absolute bottom-5 left-1/2 hidden -translate-x-1/2 gap-5 rounded-md border border-blue-800 bg-black px-4 py-2 shadow-[0_0_22px_rgba(37,99,235,0.16)] md:flex">
    {#each items as item (item.label)}
      <div class="flex items-center gap-2 text-[11px] tracking-[0.04em] text-blue-200/70">
        <span class={`h-2 w-2 rounded-full ${item.tone}`}></span>
        {item.label}
      </div>
    {/each}
  </div>
{/if}
