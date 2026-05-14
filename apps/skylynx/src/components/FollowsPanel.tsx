import { createSignal, For, Show } from 'solid-js';
import { Icon } from './Icon';

type Tab = 'following' | 'followers';

type MockAccount = { handle: string; displayName: string; mutualFollow: boolean; postCount: number };

const MOCK_FOLLOWING: MockAccount[] = [
  { handle: 'emily.bsky.social', displayName: 'Emily Chen', mutualFollow: true, postCount: 1240 },
  { handle: 'devweekly.bsky.social', displayName: 'Dev Weekly', mutualFollow: false, postCount: 408 },
  { handle: 'miriamk.bsky.social', displayName: 'Miriam Kaur', mutualFollow: true, postCount: 3871 },
  { handle: 'thenewstack.bsky.social', displayName: 'The New Stack', mutualFollow: false, postCount: 902 },
  { handle: 'jrosenberg.bsky.social', displayName: 'Jordan Rosenberg', mutualFollow: true, postCount: 559 },
  { handle: 'celine.bsky.social', displayName: 'Céline Dubois', mutualFollow: false, postCount: 2104 },
  { handle: 'opensrc.bsky.social', displayName: 'Open Source Daily', mutualFollow: false, postCount: 347 },
  { handle: 'tariq.bsky.social', displayName: 'Tariq Al-Hassan', mutualFollow: true, postCount: 715 },
  { handle: 'lenadev.bsky.social', displayName: 'Lena Vogel', mutualFollow: true, postCount: 1988 },
  { handle: 'protocolist.bsky.social', displayName: 'The Protocolist', mutualFollow: false, postCount: 231 }
];

const MOCK_FOLLOWERS: MockAccount[] = [
  { handle: 'omar.bsky.social', displayName: 'Omar Siddiqui', mutualFollow: true, postCount: 883 },
  { handle: 'nullptr.bsky.social', displayName: 'null_ptr', mutualFollow: false, postCount: 2460 },
  { handle: 'jess.bsky.social', displayName: 'Jessica Park', mutualFollow: true, postCount: 1132 },
  { handle: 'cloudwatcher.bsky.social', displayName: 'Cloud Watcher', mutualFollow: false, postCount: 54 },
  { handle: 'rdev.bsky.social', displayName: 'Rodrigo Lima', mutualFollow: true, postCount: 3200 },
  { handle: 'ainews.bsky.social', displayName: 'AI News Daily', mutualFollow: false, postCount: 1601 },
  { handle: 'sophieT.bsky.social', displayName: 'Sophie Tran', mutualFollow: false, postCount: 427 },
  { handle: 'yuki.bsky.social', displayName: 'Yuki Tanaka', mutualFollow: true, postCount: 768 }
];

export function FollowsPanel(props: { onClose: () => void }) {
  const [tab, setTab] = createSignal<Tab>('following');
  const accounts = () => (tab() === 'following' ? MOCK_FOLLOWING : MOCK_FOLLOWERS);

  return (
    <aside class="follows-panel">
      <PanelHeader onClose={props.onClose} />
      <TabBar tab={tab()} onTabChange={setTab} />
      <AccountList accounts={accounts()} />
      <PanelFooter tab={tab()} count={accounts().length} />
    </aside>
  );
}

function PanelHeader(props: { onClose: () => void }) {
  return (
    <div class="flex items-center justify-between px-5 py-4 border-b border-border">
      <div class="flex items-center gap-2">
        <Icon kind="users" class="text-accent text-[16px]" />
        <span class="text-[14px] font-semibold text-ink tracking-[-0.01em]">Follows</span>
      </div>
      <button type="button" class="btn-ghost min-h-7.5! px-2!" onClick={props.onClose} aria-label="Close follows panel">
        <Icon kind="x" class="text-[15px]" />
      </button>
    </div>
  );
}

function TabBar(props: { tab: Tab; onTabChange: (tab: Tab) => void }) {
  return (
    <div class="flex border-b border-border">
      <TabButton label="Following" active={props.tab === 'following'} onClick={() => props.onTabChange('following')} />
      <TabButton label="Followers" active={props.tab === 'followers'} onClick={() => props.onTabChange('followers')} />
    </div>
  );
}

function TabButton(props: { active: boolean; label: string; onClick: () => void }) {
  return (
    <button type="button" class="tab-btn" classList={{ 'tab-btn--active': props.active }} onClick={props.onClick}>
      {props.label}
    </button>
  );
}

function AccountList(props: { accounts: MockAccount[] }) {
  return (
    <div class="flex-1 overflow-y-auto">
      <For each={props.accounts}>{(account) => <AccountRow account={account} />}</For>
    </div>
  );
}

function AccountRow(props: { account: MockAccount }) {
  return (
    <div class="flex items-center gap-3 px-5 py-3 border-b border-border-subtle hover:bg-surface-raised transition-colors duration-120 cursor-default">
      <Avatar handle={props.account.handle} />
      <AccountInfo account={props.account} />
    </div>
  );
}

function Avatar(props: { handle: string }) {
  const initials = () => props.handle.slice(0, 2).toUpperCase();
  const hue = () => (props.handle.codePointAt(0) ?? 0) % 360;
  return (
    <div
      class="w-9 h-9 rounded-full flex items-center justify-center text-[12px] font-semibold text-white shrink-0 select-none"
      style={{ background: `hsl(${hue()}, 45%, 35%)` }}>
      {initials()}
    </div>
  );
}

function AccountInfo(props: { account: MockAccount }) {
  return (
    <div class="flex flex-col gap-0.5 min-w-0 flex-1">
      <AccountNameRow account={props.account} />
      <span class="text-[12px] text-ink-muted truncate tabular-nums">
        @{props.account.handle} · {props.account.postCount.toLocaleString()} posts
      </span>
    </div>
  );
}

function AccountNameRow(props: { account: MockAccount }) {
  return (
    <div class="flex items-center gap-1.5 min-w-0">
      <span class="text-[13px] font-medium text-ink truncate">{props.account.displayName}</span>
      <Show when={props.account.mutualFollow}>
        <MutualBadge />
      </Show>
    </div>
  );
}

function MutualBadge() {
  return (
    <span class="shrink-0 flex items-center gap-0.75 text-[10px] font-semibold text-accent bg-tag-bg border border-tag-border rounded-full px-1.5 py-px">
      <Icon kind="user-check" class="text-[10px]" />
      mutual
    </span>
  );
}

function PanelFooter(props: { count: number; tab: Tab }) {
  return (
    <div class="px-5 py-3 border-t border-border text-[12px] text-ink-muted flex items-center justify-between">
      <span class="tabular-nums">
        {props.count} {props.tab}
      </span>
      <span class="text-ink-faint italic">mock data</span>
    </div>
  );
}
