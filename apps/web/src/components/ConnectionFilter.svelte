<script lang="ts">
  import { socialGraphPath } from '$lib/graph/routes';
  import type { SocialGraphFilter } from '$lib/types/social-graph';

  type Props = {
    active?: SocialGraphFilter;
    handle?: string;
    onSelect?: (filter: SocialGraphFilter) => void;
    visible?: boolean;
  };

  let { active = 'all', handle = '', onSelect, visible = false }: Props = $props();

  const filters: { label: string; value: SocialGraphFilter }[] = [
    { label: 'All', value: 'all' },
    { label: 'Following', value: 'following' },
    { label: 'Followers', value: 'followers' },
    { label: 'Mutuals', value: 'mutuals' }
  ];
</script>

{#if visible}
  <div class="pointer-events-auto flex flex-col items-end gap-1.5">
    <div class="text-right text-[9px] tracking-[0.12em] text-blue-200/55 uppercase">Connection type</div>
    <div class="flex overflow-hidden rounded-md border border-blue-800 bg-black shadow-[0_0_22px_rgba(37,99,235,0.16)]">
      {#each filters as filter (filter.value)}
        <a
          href={socialGraphPath(handle, filter.value)}
          class={`border-r border-blue-900 px-2.5 py-1 text-[11px] transition last:border-r-0 ${
            active === filter.value
              ? 'bg-blue-600/30 text-blue-100 shadow-[inset_0_1px_0_rgba(147,197,253,0.28)]'
              : 'bg-black text-blue-200/55 hover:bg-blue-950/70 hover:text-blue-200'
          }`}
          onclick={() => onSelect?.(filter.value)}>
          {filter.label}
        </a>
      {/each}
    </div>
  </div>
{/if}
