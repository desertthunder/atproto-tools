import { createEffect, createSignal, For, Show } from 'solid-js';
import { fetchAllFollowers, fetchAllFollowingAccounts } from '../lib/api/bluesky';
import { cacheRelationships, getCachedRelationships } from '../lib/db/database';
import { Icon } from './Icon';

import type { Did, GraphRelationship, RelationshipAccount } from '../lib/types';

type Tab = GraphRelationship;

type RelationshipState = Record<Tab, RelationshipAccount[]>;

const EMPTY_RELATIONSHIPS: RelationshipState = { followers: [], following: [], mutuals: [] };

export function FollowsPanel(props: {
  actorDid?: Did;
  actorHandle?: string;
  collapsed: boolean;
  onToggleCollapsed: () => void;
}) {
  const [tab, setTab] = createSignal<Tab>('following');
  const [relationships, setRelationships] = createSignal<RelationshipState>(EMPTY_RELATIONSHIPS);
  const [error, setError] = createSignal('');
  const [isRefreshing, setIsRefreshing] = createSignal(false);

  const accounts = () => relationships()[tab()];
  const hasAccount = () => Boolean(props.actorDid);

  const loadCached = async (actorDid: Did) => {
    const [following, followers, mutuals] = await Promise.all([
      getCachedRelationships(actorDid, 'following'),
      getCachedRelationships(actorDid, 'followers'),
      getCachedRelationships(actorDid, 'mutuals')
    ]);

    setRelationships({ followers, following, mutuals });
  };

  const refresh = async () => {
    if (!props.actorDid) return;

    setError('');
    setIsRefreshing(true);

    try {
      const [following, followers] = await Promise.all([
        fetchAllFollowingAccounts(props.actorDid),
        fetchAllFollowers(props.actorDid)
      ]);
      const mutuals = buildMutuals(following, followers);

      await Promise.all([
        cacheRelationships(props.actorDid, 'following', following),
        cacheRelationships(props.actorDid, 'followers', followers),
        cacheRelationships(props.actorDid, 'mutuals', mutuals)
      ]);

      await loadCached(props.actorDid);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Unable to refresh relationships');
    } finally {
      setIsRefreshing(false);
    }
  };

  createEffect(() => {
    const actorDid = props.actorDid;
    if (!actorDid) {
      setRelationships(EMPTY_RELATIONSHIPS);
      return;
    }

    void loadCached(actorDid);
  });

  return (
    <aside class="relationships-panel">
      <Show when={props.collapsed}>
        <CollapsedPanel
          actorHandle={props.actorHandle}
          disabled={!hasAccount()}
          isRefreshing={isRefreshing()}
          onExpand={props.onToggleCollapsed}
          onRefresh={refresh}
          totals={relationshipTotals(relationships())}
        />
      </Show>
      <Show when={!props.collapsed}>
        <PanelHeader
          actorHandle={props.actorHandle}
          disabled={!hasAccount()}
          isRefreshing={isRefreshing()}
          onCollapse={props.onToggleCollapsed}
          onRefresh={refresh}
        />
        <TabBar tab={tab()} totals={relationshipTotals(relationships())} onTabChange={setTab} />
        <ErrorState error={error()} />
        <Show when={hasAccount()} fallback={<SignedOutState />}>
          <AccountList accounts={accounts()} isRefreshing={isRefreshing()} />
        </Show>
        <PanelFooter count={accounts().length} isRefreshing={isRefreshing()} tab={tab()} />
      </Show>
    </aside>
  );
}

function CollapsedPanel(props: {
  actorHandle?: string;
  disabled: boolean;
  isRefreshing: boolean;
  onExpand: () => void;
  onRefresh: () => void;
  totals: Record<Tab, number>;
}) {
  return (
    <div class="relationships-collapsed">
      <button type="button" class="btn-ghost px-2!" aria-label="Expand network panel" onClick={props.onExpand}>
        <Icon kind="chevron-left" />
      </button>
      <Icon kind="users" class="text-accent text-[18px]" />
      <div class="flex flex-col items-center gap-2 text-[11px] text-ink-muted tabular-nums">
        <span>{formatCount(props.totals.following)}</span>
        <span>{formatCount(props.totals.followers)}</span>
        <span>{formatCount(props.totals.mutuals)}</span>
      </div>
      <button
        type="button"
        class="btn-ghost px-2!"
        aria-label="Refresh network"
        disabled={props.disabled || props.isRefreshing}
        onClick={props.onRefresh}>
        <Icon kind={props.isRefreshing ? 'loader' : 'refresh'} class={props.isRefreshing ? 'animate-spin' : ''} />
      </button>
      <Show when={props.actorHandle}>{(handle) => <span class="vertical-label">@{handle()}</span>}</Show>
    </div>
  );
}

function PanelHeader(props: {
  actorHandle?: string;
  disabled: boolean;
  isRefreshing: boolean;
  onCollapse: () => void;
  onRefresh: () => void;
}) {
  return (
    <div class="flex items-center justify-between gap-3 px-5 py-4 border-b border-border">
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <Icon kind="users" class="text-accent text-[16px]" />
          <span class="text-[14px] font-semibold text-ink tracking-[-0.01em]">Network</span>
        </div>
        <Show when={props.actorHandle}>
          {(handle) => <span class="block truncate text-[12px] text-ink-muted">@{handle()}</span>}
        </Show>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="btn-ghost px-2!"
          disabled={props.disabled || props.isRefreshing}
          onClick={props.onRefresh}>
          <Icon kind={props.isRefreshing ? 'loader' : 'refresh'} class={props.isRefreshing ? 'animate-spin' : ''} />
          Refresh
        </button>
        <button type="button" class="btn-ghost px-2!" aria-label="Collapse network panel" onClick={props.onCollapse}>
          <Icon kind="chevron-right" />
        </button>
      </div>
    </div>
  );
}

function TabBar(props: { onTabChange: (tab: Tab) => void; tab: Tab; totals: Record<Tab, number> }) {
  return (
    <div class="grid grid-cols-3 border-b border-border">
      <TabButton
        active={props.tab === 'following'}
        count={props.totals.following}
        label="Following"
        onClick={() => props.onTabChange('following')}
      />
      <TabButton
        active={props.tab === 'followers'}
        count={props.totals.followers}
        label="Followers"
        onClick={() => props.onTabChange('followers')}
      />
      <TabButton
        active={props.tab === 'mutuals'}
        count={props.totals.mutuals}
        label="Mutuals"
        onClick={() => props.onTabChange('mutuals')}
      />
    </div>
  );
}

function TabButton(props: { active: boolean; count: number; label: string; onClick: () => void }) {
  return (
    <button type="button" class="tab-btn" classList={{ 'tab-btn--active': props.active }} onClick={props.onClick}>
      <span>{props.label}</span>
      <span class="tabular-nums text-[11px] opacity-70">{formatCount(props.count)}</span>
    </button>
  );
}

function AccountList(props: { accounts: RelationshipAccount[]; isRefreshing: boolean }) {
  return (
    <Show when={props.accounts.length > 0} fallback={<EmptyRelationshipState isRefreshing={props.isRefreshing} />}>
      <div class="flex-1 overflow-y-auto">
        <For each={props.accounts}>{(account) => <AccountRow account={account} />}</For>
      </div>
    </Show>
  );
}

function AccountRow(props: { account: RelationshipAccount }) {
  return (
    <a
      href={props.account.profileUrl}
      target="_blank"
      rel="noopener noreferrer"
      class="grid grid-cols-[2.25rem_1fr] items-center gap-3 px-5 py-3 border-b border-border-subtle hover:bg-surface-raised transition-colors duration-120 no-underline">
      <Avatar account={props.account} />
      <AccountInfo account={props.account} />
    </a>
  );
}

function Avatar(props: { account: RelationshipAccount }) {
  return (
    <Show when={props.account.avatar} fallback={<InitialsAvatar handle={props.account.handle} />}>
      {(avatar) => <img alt="" class="size-9 rounded-full object-cover" src={avatar()} />}
    </Show>
  );
}

function InitialsAvatar(props: { handle: string }) {
  const initials = () => props.handle.slice(0, 2).toUpperCase();
  const hue = () => (props.handle.codePointAt(0) ?? 0) % 360;
  return (
    <span
      class="size-9 rounded-full flex items-center justify-center text-[12px] font-semibold text-white shrink-0 select-none"
      style={{ background: `hsl(${hue()}, 45%, 35%)` }}>
      {initials()}
    </span>
  );
}

function AccountInfo(props: { account: RelationshipAccount }) {
  return (
    <div class="flex flex-col gap-0.5 min-w-0">
      <div class="flex items-center gap-1.5 min-w-0">
        <span class="text-[13px] font-medium text-ink truncate">
          {props.account.displayName || props.account.handle}
        </span>
        <Show when={props.account.relationship === 'mutuals'}>
          <MutualBadge />
        </Show>
      </div>
      <span class="text-[12px] text-ink-muted truncate">@{props.account.handle}</span>
      <LastPost account={props.account} />
    </div>
  );
}

function LastPost(props: { account: RelationshipAccount }) {
  return (
    <Show
      when={props.account.lastPostAt}
      fallback={<span class="text-[11px] text-ink-faint">Last post not scanned</span>}>
      {(lastPostAt) => (
        <span class="text-[11px] text-ink-faint">
          Last posted{' '}
          <Show when={props.account.lastPostUri} fallback={relativeTime(lastPostAt())}>
            {(uri) => (
              <a href={bskyPostUrl(uri())} target="_blank" rel="noopener noreferrer" class="hover:text-accent">
                {relativeTime(lastPostAt())}
              </a>
            )}
          </Show>
        </span>
      )}
    </Show>
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

function ErrorState(props: { error: string }) {
  return (
    <Show when={props.error}>
      <div class="flex items-start gap-2.5 px-5 py-3 border-b border-border-subtle text-danger text-[12px]">
        <Icon kind="warning" class="mt-0.5" />
        <span>{props.error}</span>
      </div>
    </Show>
  );
}

function SignedOutState() {
  return (
    <div class="flex-1 flex flex-col items-center justify-center gap-2 px-5 py-10 text-center">
      <Icon kind="lock" class="text-ink-faint text-[32px]" />
      <p class="text-[13px] text-ink-muted">Sign in to load your followers, following, and mutuals.</p>
    </div>
  );
}

function EmptyRelationshipState(props: { isRefreshing: boolean }) {
  const text = () => {
    if (props.isRefreshing) return 'Refreshing network...';
    return 'No cached accounts yet.';
  };

  return (
    <div class="flex-1 flex items-center justify-center px-5 py-10 text-center text-[13px] text-ink-muted">
      {text()}
    </div>
  );
}

function PanelFooter(props: { count: number; isRefreshing: boolean; tab: Tab }) {
  return (
    <div class="px-5 py-3 border-t border-border text-[12px] text-ink-muted flex items-center justify-between">
      <span class="tabular-nums">
        {props.count} {props.tab}
      </span>
      <span>{props.isRefreshing ? 'refreshing' : 'cached'}</span>
    </div>
  );
}

const buildMutuals = (following: RelationshipAccount[], followers: RelationshipAccount[]) => {
  const followerDids = new Set(followers.map((account) => account.did));
  return following
    .filter((account) => followerDids.has(account.did))
    .map((account) => ({ ...account, relationship: 'mutuals' as const }));
};

const relationshipTotals = (state: RelationshipState) => {
  return { followers: state.followers.length, following: state.following.length, mutuals: state.mutuals.length };
};

const bskyPostUrl = (uri: string) => {
  const parts = uri.split('/');
  const did = parts[2] ?? '';
  const rkey = parts.at(-1) ?? '';
  return `https://bsky.app/profile/${did}/post/${rkey}`;
};

const formatCount = (count: number) => {
  if (count < 1000) return String(count);
  return new Intl.NumberFormat(undefined, { maximumFractionDigits: 1, notation: 'compact' }).format(count);
};

const relativeTime = (iso: string) => {
  const deltaSeconds = Math.round((new Date(iso).getTime() - Date.now()) / 1000);
  const abs = Math.abs(deltaSeconds);

  if (abs < 60) return 'just now';
  if (abs < 3600) return formatRelative(deltaSeconds, 60, 'minute');
  if (abs < 86_400) return formatRelative(deltaSeconds, 3600, 'hour');
  if (abs < 2_592_000) return formatRelative(deltaSeconds, 86_400, 'day');

  return new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', year: 'numeric' }).format(new Date(iso));
};

const formatRelative = (deltaSeconds: number, divisor: number, unit: Intl.RelativeTimeFormatUnit) => {
  return new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' }).format(Math.round(deltaSeconds / divisor), unit);
};
