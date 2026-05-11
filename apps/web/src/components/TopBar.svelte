<script lang="ts">
  import type { GraphFetchLimit } from '$lib/types/db';
  import type { SocialGraphAvatarMode, SocialGraphSource } from '$lib/types/social-graph';

  type Props = {
    avatarMode?: SocialGraphAvatarMode;
    handle: string;
    lastFetchedAt?: string;
    limit: GraphFetchLimit;
    limits: readonly GraphFetchLimit[];
    loading?: boolean;
    onAvatarModeChange?: (mode: SocialGraphAvatarMode) => void;
    onForceRefresh?: () => void;
    onHandleInput?: (handle: string) => void;
    onLimitChange?: (limit: GraphFetchLimit) => void;
    onLoad?: () => void;
    source?: SocialGraphSource;
  };

  let {
    avatarMode = 'rings',
    handle,
    lastFetchedAt,
    limit,
    limits,
    loading = false,
    onAvatarModeChange,
    onForceRefresh,
    onHandleInput,
    onLimitChange,
    onLoad,
    source
  }: Props = $props();

  const formatFetchedAt = (value: string) => {
    const date = new Date(value);
    if (Number.isNaN(date.valueOf())) return null;

    return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(date);
  };

  const fetchedLabel = $derived(lastFetchedAt ? formatFetchedAt(lastFetchedAt) : null);
  const cacheLabel = $derived(source === 'cache' ? 'Cache' : source === 'network' ? 'Fresh' : null);

  const parseLimit = (value: string): GraphFetchLimit => {
    const nextLimit = Number(value);
    return limits.includes(nextLimit as GraphFetchLimit) ? (nextLimit as GraphFetchLimit) : limit;
  };

  const avatarModeOptions: { icon: string; label: string; value: SocialGraphAvatarMode }[] = [
    { icon: 'i-tabler-circle-dot', label: 'Rings', value: 'rings' },
    { icon: 'i-tabler-user-circle', label: 'Avatars', value: 'avatars' }
  ];
</script>

<header
  class="pointer-events-auto flex items-center gap-3 bg-linear-to-b from-black/95 to-transparent px-4 py-4 sm:px-5">
  <div
    class="h-7 w-7 shrink-0 rounded-full bg-[radial-gradient(circle_at_35%_35%,var(--color-blue-300),var(--color-blue-500)_52%,var(--color-blue-800))] shadow-[0_0_16px_rgba(37,99,235,0.75)]">
  </div>
  <div class="hidden text-sm font-medium tracking-[0.08em] text-blue-50/70 uppercase sm:block">Skymap</div>

  <form
    class="relative ml-auto flex w-full max-w-176 items-center gap-2"
    onsubmit={(event) => {
      event.preventDefault();
      onLoad?.();
    }}>
    <div class="relative min-w-0 flex-1">
      <label for="graph-handle" class="sr-only">Bluesky handle</label>
      <span class="pointer-events-none absolute top-1/2 left-3 flex -translate-y-1/2 items-center text-blue-200/40">
        <i class="i-tabler-search"></i>
      </span>
      <input
        id="graph-handle"
        class="h-9 w-full rounded-md border border-blue-900 bg-black pr-3 pl-9 font-mono text-[13px] text-blue-50 shadow-[0_0_22px_rgba(37,99,235,0.13)] transition outline-none placeholder:text-blue-200/30 focus:border-blue-500 focus:shadow-[0_0_0_2px_rgba(37,99,235,0.32),inset_0_0_0_1px_rgba(59,130,246,0.9)]"
        value={handle}
        placeholder="@handle.bsky.social"
        autocomplete="off"
        spellcheck="false"
        oninput={(event) => onHandleInput?.(event.currentTarget.value)}
        onkeydown={(event) => {
          if (event.key !== 'Enter') return;

          event.preventDefault();
          onLoad?.();
        }} />
    </div>

    <div class="relative h-9 w-18 shrink-0">
      <label class="sr-only" for="graph-limit">Relationship limit</label>
      <select
        id="graph-limit"
        class="h-9 w-full appearance-none rounded-md border border-blue-900 bg-black py-0 pr-7 pl-3 font-mono text-[12px] leading-9 text-blue-100 tabular-nums transition-colors outline-none hover:border-blue-700 focus:border-blue-500 disabled:cursor-wait disabled:border-blue-950 disabled:text-blue-200/35"
        value={limit}
        disabled={loading}
        onchange={(event) => onLimitChange?.(parseLimit(event.currentTarget.value))}>
        {#each limits as option (option)}
          <option value={option}>{option}</option>
        {/each}
      </select>
      <span class="pointer-events-none absolute top-1/2 right-2.5 flex -translate-y-1/2 items-center text-blue-200/55">
        <i class="i-tabler-chevron-down"></i>
      </span>
    </div>

    <div class="flex h-9 overflow-hidden rounded-md border border-blue-900 bg-black">
      {#each avatarModeOptions as option (option.value)}
        <button
          type="button"
          class={`flex h-9 items-center gap-1.5 border-r border-blue-900 px-2.5 text-[12px] font-medium whitespace-nowrap transition-colors last:border-r-0 ${
            avatarMode === option.value
              ? 'bg-blue-600/25 text-blue-50'
              : 'text-blue-200/55 hover:bg-blue-950/70 hover:text-blue-100'
          }`}
          aria-pressed={avatarMode === option.value}
          onclick={() => onAvatarModeChange?.(option.value)}>
          <span class="flex items-center">
            <i class={option.icon}></i>
          </span>
          {option.label}
        </button>
      {/each}
    </div>

    <button
      type="submit"
      class="flex h-9 items-center gap-1.5 rounded bg-blue-600 px-3 text-[12px] font-medium whitespace-nowrap text-white transition hover:bg-blue-500 hover:shadow-[inset_0_0_0_1px_rgba(191,219,254,0.65)] disabled:cursor-wait disabled:bg-blue-950 disabled:text-blue-200/60"
      disabled={loading}>
      <span class="flex items-center">
        <i class="i-tabler-database"></i>
      </span>
      {loading ? 'Loading' : 'Load Graph'}
    </button>

    <button
      type="button"
      class="flex h-9 items-center gap-1.5 rounded border border-blue-700 bg-black px-2.5 text-[12px] font-medium whitespace-nowrap text-blue-100 transition hover:border-blue-400 hover:bg-blue-950 disabled:cursor-wait disabled:border-blue-950 disabled:text-blue-200/35"
      disabled={loading}
      onclick={() => onForceRefresh?.()}>
      <span class="flex items-center">
        <i class="i-tabler-refresh"></i>
      </span>
      Refresh
    </button>
  </form>

  {#if fetchedLabel || cacheLabel}
    <div class="hidden min-w-44 font-mono text-[11px] text-blue-200/45 lg:block">
      {#if cacheLabel}
        <span class="text-blue-300/70">{cacheLabel}</span>
      {/if}
      {#if fetchedLabel}
        <span>{cacheLabel ? ' / ' : ''}{fetchedLabel}</span>
      {/if}
    </div>
  {/if}
</header>
