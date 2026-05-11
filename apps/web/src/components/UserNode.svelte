<script lang="ts">
  import type { SocialGraphNode } from '$lib/types/social-graph';
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';

  let {
    data,
    selected,
    targetPosition = Position.Left,
    sourcePosition = Position.Right
  }: NodeProps<SocialGraphNode> = $props();

  const profileHref = $derived(`https://bsky.app/profile/${data.handle.replace(/^@/, '')}`);
  const initials = $derived(
    data.displayName
      .split(/\s+/)
      .slice(0, 2)
      .map((part) => part[0])
      .join('')
      .toUpperCase()
  );

  const accentClasses = {
    follower: 'bg-blue-700',
    following: 'bg-rose-500',
    mutual: 'bg-emerald-500',
    origin: 'bg-sky-500'
  };

  const borderClasses = {
    follower: 'border-blue-900',
    following: 'border-blue-900',
    mutual: 'border-blue-900',
    origin: 'border-sky-500'
  };
</script>

<Handle
  type="target"
  position={targetPosition}
  isConnectableStart={false}
  isConnectableEnd={false}
  aria-hidden="true"
  class="pointer-events-none! h-2.5! w-2.5! border-0! bg-transparent! opacity-0!" />

<article
  class={`relative w-65 overflow-hidden rounded-lg border bg-black shadow-[0_18px_48px_rgba(0,0,0,0.52),0_0_24px_rgba(37,99,235,0.18)] transition ${
    selected ? 'border-blue-300 ring-2 ring-blue-500/45' : borderClasses[data.relationship]
  }`}>
  <div class="absolute inset-x-0 top-0 h-px bg-blue-400/80"></div>
  <div class={`absolute inset-y-0 left-0 w-1 ${accentClasses[data.relationship]}`}></div>

  <div class="flex items-center gap-3 p-3 pl-4">
    <div
      class="grid h-12 w-12 shrink-0 place-items-center overflow-hidden rounded-md border border-blue-500/80 bg-blue-950 text-sm font-bold text-blue-100 shadow-[0_0_18px_rgba(37,99,235,0.35)]">
      {#if data.avatarUrl}
        <img class="h-full w-full object-cover" src={data.avatarUrl} alt="" loading="lazy" />
      {:else}
        {initials}
      {/if}
    </div>

    <div class="min-w-0 flex-1">
      <div class="truncate text-[13px] font-bold text-blue-50">{data.displayName || data.name}</div>
      <a
        class="nodrag nowheel block truncate font-mono text-[11px] text-blue-300 transition hover:text-blue-100"
        href={profileHref}
        target="_blank"
        rel="external noopener noreferrer">
        @{data.handle.replace(/^@/, '')}
      </a>
      <div class="mt-2 flex items-center gap-1.5">
        <span class={`h-1.5 w-1.5 rounded-full ${accentClasses[data.relationship]}`}></span>
        <span class="font-mono text-[10px] tracking-[0.08em] text-blue-200/60 uppercase">{data.relationship}</span>
      </div>
    </div>
  </div>
</article>

<Handle
  type="source"
  position={sourcePosition}
  isConnectableStart={false}
  isConnectableEnd={false}
  aria-hidden="true"
  class="pointer-events-none! h-2.5! w-2.5! border-0! bg-transparent! opacity-0!" />
