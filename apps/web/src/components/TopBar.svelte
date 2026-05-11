<script lang="ts">
  type Props = { handle: string; loading?: boolean; onHandleInput?: (handle: string) => void; onLoad?: () => void };

  let { handle, loading = false, onHandleInput, onLoad }: Props = $props();
</script>

<header
  class="pointer-events-auto flex items-center gap-3 bg-linear-to-b from-black/95 to-transparent px-4 py-4 sm:px-5">
  <div
    class="h-7 w-7 shrink-0 rounded-full bg-[radial-gradient(circle_at_35%_35%,var(--color-blue-300),var(--color-blue-500)_52%,var(--color-blue-800))] shadow-[0_0_16px_rgba(37,99,235,0.75)]">
  </div>
  <div class="hidden text-sm font-medium tracking-[0.08em] text-blue-50/70 uppercase sm:block">Skymap</div>

  <form
    class="relative ml-auto w-full max-w-140"
    onsubmit={(event) => {
      event.preventDefault();
      onLoad?.();
    }}>
    <label for="graph-handle" class="sr-only">Bluesky handle</label>
    <span class="pointer-events-none absolute top-1/2 left-3 h-3.5 w-3.5 -translate-y-1/2 text-blue-200/40">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
        <circle cx="6.5" cy="6.5" r="4.5" />
        <path d="M10.5 10.5l3 3" />
      </svg>
    </span>
    <input
      id="graph-handle"
      class="h-9 w-full rounded-md border border-blue-900 bg-black pr-31 pl-9 font-mono text-[13px] text-blue-50 shadow-[0_0_22px_rgba(37,99,235,0.13)] transition outline-none placeholder:text-blue-200/30 focus:border-blue-500 focus:shadow-[0_0_0_2px_rgba(37,99,235,0.32),inset_0_0_0_1px_rgba(59,130,246,0.9)]"
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

    <button
      type="submit"
      class="absolute top-1 right-1 h-7 rounded bg-blue-600 px-3.5 text-[12px] font-medium whitespace-nowrap text-white transition hover:bg-blue-500 hover:shadow-[inset_0_0_0_1px_rgba(191,219,254,0.65)] disabled:cursor-wait disabled:bg-blue-950 disabled:text-blue-200/60"
      disabled={loading}>
      {loading ? 'Loading' : 'Load Graph'}
    </button>
  </form>
</header>
