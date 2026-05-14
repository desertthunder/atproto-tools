import { createSignal, Show, type ParentProps } from 'solid-js';
import { A } from '@solidjs/router';
import { Icon } from './Icon';
import { FollowsPanel } from './FollowsPanel';

export function AppShell(props: ParentProps) {
  const [sidebarOpen, setSidebarOpen] = createSignal(false);
  const openSidebar = () => setSidebarOpen(true);
  const closeSidebar = () => setSidebarOpen(false);

  return (
    <>
      <AppHeader onOpenSidebar={openSidebar} />
      <div class="flex-1">{props.children}</div>
      <AppFooter />
      <Show when={sidebarOpen()}>
        <div class="follows-overlay" onClick={closeSidebar} aria-hidden="true" />
        <FollowsPanel onClose={closeSidebar} />
      </Show>
    </>
  );
}

function AppHeader(props: { onOpenSidebar: () => void }) {
  return (
    <header class="border-b border-border-subtle">
      <div class="flex items-center justify-between h-13 px-[clamp(20px,4vw,56px)] max-w-350 mx-auto w-full">
        <HeaderLogo />
        <HeaderNav onOpenSidebar={props.onOpenSidebar} />
      </div>
    </header>
  );
}

function HeaderLogo() {
  return (
    <A href="/" class="no-underline flex items-center gap-2">
      <Icon kind="link" class="text-[18px] text-accent" />
      <span
        class="text-[17px] font-semibold tracking-[-0.01em] text-ink"
        style={{ 'font-family': 'var(--font-display)' }}>
        Skylynx
      </span>
    </A>
  );
}

function HeaderNav(props: { onOpenSidebar: () => void }) {
  return (
    <nav class="flex items-center gap-1">
      <A href="/about" class="btn-ghost no-underline">
        About
      </A>
      <a href="https://bsky.app" target="_blank" rel="noopener noreferrer" class="btn-ghost">
        <Icon kind="bluesky" class="text-[15px]" />
        Bluesky
      </a>
      <button
        type="button"
        class="btn-ghost"
        onClick={props.onOpenSidebar}
        aria-label="Open follows panel">
        <Icon kind="users" class="text-[15px]" />
        Follows
      </button>
    </nav>
  );
}

function AppFooter() {
  return (
    <footer class="border-t border-border-subtle">
      <div class="flex items-center justify-between px-[clamp(20px,4vw,56px)] max-w-350 mx-auto w-full py-4 text-[12px] text-ink-muted">
        <span>Skylynx — AT Protocol link aggregator</span>
        <span>Data sourced from the Bluesky public API</span>
      </div>
    </footer>
  );
}
