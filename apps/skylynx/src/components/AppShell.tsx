import { type ParentProps } from 'solid-js';
import { A } from '@solidjs/router';
import { Icon } from './Icon';

export function AppShell(props: ParentProps) {
  return (
    <>
      <AppHeader />
      <div class="flex-1">{props.children}</div>
      <AppFooter />
    </>
  );
}

function AppHeader() {
  return (
    <header class="border-b border-border-subtle">
      <div class="px-[clamp(20px,4vw,56px)] max-w-350 mx-auto w-full flex items-center justify-between h-13">
        <HeaderLogo />
        <HeaderNav />
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

function HeaderNav() {
  return (
    <nav class="flex items-center gap-1">
      <A href="/" class="btn-ghost no-underline" end>
        About
      </A>
      <A href="/app" class="btn-ghost no-underline">
        App
      </A>
      <a href="https://bsky.app" target="_blank" rel="noopener noreferrer" class="btn-ghost">
        <Icon kind="bluesky" class="text-[15px]" />
        Bluesky
      </a>
    </nav>
  );
}

function AppFooter() {
  return (
    <footer class="border-t border-border-subtle">
      <div class="px-[clamp(20px,4vw,56px)] max-w-350 mx-auto w-full flex items-center justify-between h-13 py-4 text-[12px] text-ink-muted">
        <span>Skylynx — AT Protocol link aggregator</span>
        <span>Data sourced from the Bluesky public API</span>
      </div>
    </footer>
  );
}
