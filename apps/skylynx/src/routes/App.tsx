import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
  on,
  onCleanup,
  onMount,
  type Accessor
} from 'solid-js';
import { FollowsPanel } from '../components/FollowsPanel';
import { searchActorsTypeahead } from '../lib/api/bluesky';
import { aggregateDigestLinks, generateLinkDigest } from '../lib/api/link-digest';
import { useAuth } from '../lib/auth/AuthContext';
import { getCompletedDigestProgress, getLatestPausedDigestProgress } from '../lib/db/database';
import type { AuthenticatedAccount } from '../lib/auth/oauth';
import type { DigestProgressSnapshot } from '../lib/db/schema';
import type {
  ActorSuggestion,
  Did,
  DigestLink,
  LinkDigestOptions,
  LinkDigestProgress,
  LinkDigestStatusEvent
} from '../lib/types';
import { Icon, type IconKind } from '../components/Icon';

const DEFAULT_OPTIONS: LinkDigestOptions = {
  actor: '',
  feedLimit: 100,
  limit: 25,
  maxPages: 5,
  minScore: 3,
  minShares: 2,
  refreshFollows: false
};

type UpdateOption = <Key extends keyof LinkDigestOptions>(key: Key, value: LinkDigestOptions[Key]) => void;

type DigestSectionProps = {
  error: string;
  hasResults: boolean;
  historyVersion: number;
  isRunning: boolean;
  links: DigestLink[];
  onPause: () => void;
  onResume: () => void;
  onSidebarTabChange: (tab: SidebarTab) => void;
  onSelectHistory: (run: DigestProgressSnapshot) => void;
  onSubmit: (event: SubmitEvent) => void;
  options: LinkDigestOptions;
  pausedRun?: DigestProgressSnapshot;
  progress: LinkDigestProgress;
  progressText: string;
  sidebarTab: SidebarTab;
  statusEvents: StatusItem[];
  updateOption: UpdateOption;
  viewerDid?: Did;
  viewerHandle?: string;
};

type SidebarTab = 'digest' | 'network' | 'history';

type StatusItem = { id: number; text: string };

type DatePickerValue = { date: string; hour: number; minute: number };

type MonthDay = { date: Date; inMonth: boolean; isoDate: string };

export function LinkDigest() {
  const auth = useAuth();
  const [options, setOptions] = createSignal(DEFAULT_OPTIONS);
  const [links, setLinks] = createSignal<DigestLink[]>([]);
  const [progress, setProgress] = createSignal<LinkDigestProgress>({ completed: 0, phase: 'idle', total: 0 });
  const [error, setError] = createSignal('');
  const [isRunning, setIsRunning] = createSignal(false);
  const [pausedRun, setPausedRun] = createSignal<DigestProgressSnapshot>();
  const [sidebarTab, setSidebarTab] = createSignal<SidebarTab>('digest');
  const [historyVersion, setHistoryVersion] = createSignal(0);
  const [statusEvents, setStatusEvents] = createSignal<StatusItem[]>([]);
  let pauseRequested = false;

  const runDigest = async (event?: SubmitEvent, resumeRunId?: string) => {
    event?.preventDefault();
    const account = auth.account();
    if (!account) {
      setError('Sign in before building a digest');
      return;
    }

    const digestOptions = normalizedOptions({ ...options(), actor: account.handle });
    setOptions(digestOptions);
    setError('');
    setLinks([]);
    setStatusEvents([]);
    setIsRunning(true);
    if (!resumeRunId) setPausedRun();
    pauseRequested = false;

    try {
      for await (const statusEvent of generateLinkDigest(digestOptions, {
        resumeRunId,
        shouldPause: () => pauseRequested
      })) {
        setProgress(progressFromEvent(statusEvent));
        setStatusEvents((items) => [...items, { id: items.length + 1, text: statusText(statusEvent) }]);

        if (statusEvent.type === 'paused') {
          await loadPausedRun();
        }

        if (statusEvent.type === 'done') {
          setLinks(statusEvent.result.links);
          setPausedRun();
          setHistoryVersion((version) => version + 1);
        }
      }
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Digest failed');
    } finally {
      setIsRunning(false);
    }
  };

  const pauseDigest = () => {
    pauseRequested = true;
  };

  const resumeDigest = () => {
    const run = pausedRun();
    if (!run) return;

    setOptions(run.options);
    void runDigest(undefined, run.id);
  };

  const loadPausedRun = async () => {
    setPausedRun(await getLatestPausedDigestProgress(options().actor));
  };

  const selectHistoryRun = (run: DigestProgressSnapshot) => {
    const restoredLinks = digestLinksFromSnapshot(run);
    setOptions(run.options);
    setLinks(restoredLinks);
    setError('');
    setProgress({ completed: run.completed, phase: 'done', total: run.total });
    setStatusEvents([{ id: 1, text: `Loaded saved digest from ${formatDigestDate(run.updatedAt)}` }]);
  };

  const progressText = () => {
    const p = progress();
    if (p.phase === 'idle') return 'Ready';
    if (p.phase === 'resolving') return 'Resolving actor…';
    if (p.phase === 'fetching-follows') return 'Fetching follows…';
    if (p.phase === 'paused') return `Paused — scanned ${p.completed} / ${p.total}`;
    if (p.phase === 'done') return `Done — scanned ${p.total} follows`;
    return `Scanning feeds ${p.completed} / ${p.total}`;
  };

  const hasResults = () => links().length > 0;
  const updateOption: UpdateOption = (key, value) => setOptions({ ...options(), [key]: value });

  createEffect(() => {
    const account = auth.account();
    if (!account) return;
    if (options().actor === account.handle) return;

    setOptions({ ...options(), actor: account.handle });
  });

  createEffect(() => {
    const actor = options().actor;
    if (!actor) return;
    void loadPausedRun();
  });

  return (
    <div class="app-route">
      <AuthToolbar
        account={auth.account()}
        authError={auth.error()}
        isLoading={auth.isLoading()}
        onSignIn={auth.signIn}
        onSignOut={auth.signOut}
      />
      <DigestSection
        error={error()}
        hasResults={hasResults()}
        isRunning={isRunning()}
        links={links()}
        onPause={pauseDigest}
        onResume={resumeDigest}
        onSelectHistory={selectHistoryRun}
        onSidebarTabChange={setSidebarTab}
        onSubmit={runDigest}
        options={options()}
        pausedRun={pausedRun()}
        progress={progress()}
        progressText={progressText()}
        sidebarTab={sidebarTab()}
        statusEvents={statusEvents()}
        updateOption={updateOption}
        viewerDid={auth.account()?.did}
        viewerHandle={auth.account()?.handle}
        historyVersion={historyVersion()}
      />
    </div>
  );
}

function AuthToolbar(props: {
  account?: AuthenticatedAccount;
  authError: string;
  isLoading: boolean;
  onSignIn: (identifier: string) => Promise<void>;
  onSignOut: () => Promise<void>;
}) {
  const [identifier, setIdentifier] = createSignal('');
  const suggestedIdentifier = () => identifier().trim() || props.account?.handle || '';

  const submit = (event: SubmitEvent) => {
    event.preventDefault();
    void props.onSignIn(suggestedIdentifier());
  };

  return (
    <section class="auth-strip">
      <div class="min-w-0">
        <div class="flex items-center gap-2 text-[13px] font-semibold text-ink">
          <Icon kind={props.account ? 'user-check' : 'lock'} class="text-accent" />
          <span>{props.account ? `Signed in as @${props.account.handle}` : 'Bluesky sign in'}</span>
        </div>
        <p class="text-[12px] text-ink-muted">
          {props.account
            ? 'Network cache and digest defaults are tied to this account.'
            : 'Sign in to load followers, following, mutuals, and last-post cache.'}
        </p>
        <Show when={props.authError}>
          <p class="mt-1 text-[12px] text-danger">{props.authError}</p>
        </Show>
      </div>
      <Show
        when={props.account}
        fallback={
          <form class="flex items-center gap-2 min-w-[min(100%,26rem)]" onSubmit={submit}>
            <SignInActorField value={identifier()} onUpdate={setIdentifier} />
            <button type="submit" class="btn-primary w-auto! px-4!" disabled={props.isLoading}>
              <Icon kind={props.isLoading ? 'loader' : 'bluesky'} class={props.isLoading ? 'animate-spin' : ''} />
              <span>Sign in</span>
            </button>
          </form>
        }>
        <button type="button" class="btn-ghost" disabled={props.isLoading} onClick={() => void props.onSignOut()}>
          <Icon kind={props.isLoading ? 'loader' : 'x'} class={props.isLoading ? 'animate-spin' : ''} />
          Sign out
        </button>
      </Show>
    </section>
  );
}

function DigestSection(props: DigestSectionProps) {
  return (
    <section class="app-workspace">
      <SidebarTitle />
      <aside class="app-sidebar">
        <SidebarTabs active={props.sidebarTab} onChange={props.onSidebarTabChange} />
        <div class="min-h-0 flex-1 overflow-hidden">
          <SidebarPanel {...props} />
        </div>
      </aside>
      <ResultsPanel {...props} />
    </section>
  );
}

function SidebarPanel(props: DigestSectionProps) {
  return (
    <Switch>
      <Match when={props.sidebarTab === 'network'}>
        <FollowsPanel actorDid={props.viewerDid} actorHandle={props.viewerHandle} />
      </Match>
      <Match when={props.sidebarTab === 'history'}>
        <DigestHistory
          actor={props.viewerHandle}
          refreshKey={props.historyVersion}
          onSelectHistory={props.onSelectHistory}
        />
      </Match>
      <Match when={props.sidebarTab === 'digest'}>
        <DigestControls
          actorHandle={props.viewerHandle}
          isRunning={props.isRunning}
          onPause={props.onPause}
          onResume={props.onResume}
          onSubmit={props.onSubmit}
          options={props.options}
          pausedRun={props.pausedRun}
          signedIn={Boolean(props.viewerDid)}
          updateOption={props.updateOption}
        />
      </Match>
    </Switch>
  );
}

function SidebarTitle() {
  return (
    <div class="app-workspace-title">
      <span class="flex items-center text-accent">
        <i class="i-tabler-bolt"></i>
      </span>
      <h2 class="text-[18px] font-semibold tracking-[-0.01em] text-ink" style={{ 'font-family': 'var(--font-body)' }}>
        Build a digest
      </h2>
    </div>
  );
}

function SidebarTabs(props: { active: SidebarTab; onChange: (tab: SidebarTab) => void }) {
  return (
    <div class="sidebar-tabs">
      <SidebarTabButton
        active={props.active === 'digest'}
        icon="bolt"
        label="Digest"
        onClick={() => props.onChange('digest')}
      />
      <SidebarTabButton
        active={props.active === 'network'}
        icon="users"
        label="Network"
        onClick={() => props.onChange('network')}
      />
      <SidebarTabButton
        active={props.active === 'history'}
        icon="database"
        label="History"
        onClick={() => props.onChange('history')}
      />
    </div>
  );
}

function SidebarTabButton(props: { active: boolean; icon: IconKind; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      class="sidebar-tab"
      classList={{ 'sidebar-tab--active': props.active }}
      onClick={props.onClick}>
      <Icon kind={props.icon} />
      {props.label}
    </button>
  );
}

function DigestHistory(props: {
  actor?: string;
  onSelectHistory: (run: DigestProgressSnapshot) => void;
  refreshKey: number;
}) {
  const [runs, setRuns] = createSignal<DigestProgressSnapshot[]>([]);
  const [isLoading, setIsLoading] = createSignal(true);

  const loadRuns = async () => {
    setIsLoading(true);
    try {
      setRuns(await getCompletedDigestProgress(props.actor));
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(
    on([() => props.actor, () => props.refreshKey], () => {
      void loadRuns();
    })
  );

  return (
    <div class="flex h-full min-h-0 flex-col">
      <div class="border-b border-border px-5 py-4">
        <div class="flex items-center gap-2 text-[14px] font-semibold text-ink">
          <Icon kind="database" class="text-accent" />
          Previous digests
        </div>
        <p class="text-[12px] text-ink-muted">Saved locally from completed runs.</p>
      </div>
      <Show when={!isLoading()} fallback={<HistoryEmpty text="Loading digests..." />}>
        <Show when={runs().length > 0} fallback={<HistoryEmpty text="No completed digests yet." />}>
          <ol class="min-h-0 flex-1 overflow-y-auto p-2">
            <For each={runs()}>{(run) => <HistoryRun run={run} onSelect={props.onSelectHistory} />}</For>
          </ol>
        </Show>
      </Show>
    </div>
  );
}

function HistoryRun(props: { onSelect: (run: DigestProgressSnapshot) => void; run: DigestProgressSnapshot }) {
  const linkCount = () => digestLinksFromSnapshot(props.run).length;

  return (
    <li class="list-none">
      <button
        type="button"
        class="w-full rounded-lg border-0 bg-transparent px-3 py-3 text-left hover:bg-surface-raised transition-colors duration-120"
        onClick={() => props.onSelect(props.run)}>
        <span class="block text-[13px] font-semibold text-ink">{formatDigestDate(props.run.updatedAt)}</span>
        <span class="mt-1 block text-[12px] text-ink-muted">
          @{props.run.actor} · {formatCount(linkCount())} links · {formatCount(props.run.postCount)} posts
        </span>
        <span class="mt-1 block text-[11px] text-ink-faint">
          {formatWindow(props.run.options.since, props.run.options.until)}
        </span>
      </button>
    </li>
  );
}

function HistoryEmpty(props: { text: string }) {
  return (
    <div class="flex flex-1 items-center justify-center px-5 py-10 text-center text-[13px] text-ink-muted">
      {props.text}
    </div>
  );
}

function DigestControls(props: {
  actorHandle?: string;
  isRunning: boolean;
  onPause: () => void;
  onResume: () => void;
  onSubmit: (event: SubmitEvent) => void;
  options: LinkDigestOptions;
  pausedRun?: DigestProgressSnapshot;
  signedIn: boolean;
  updateOption: UpdateOption;
}) {
  return (
    <div class="flex h-full min-h-0 flex-col gap-5 overflow-y-auto p-5">
      <form class="grid gap-3.5 grid-cols-2" onSubmit={props.onSubmit}>
        <DigestIdentity signedIn={props.signedIn} actorHandle={props.actorHandle} />
        <DatePickerField
          label="Since"
          disabled={!props.signedIn}
          value={props.options.since}
          onUpdate={(value) => props.updateOption('since', value)}
        />
        <DatePickerField
          label="Until"
          disabled={!props.signedIn}
          value={props.options.until}
          onUpdate={(value) => props.updateOption('until', value)}
        />
        <NumberField
          label="Max links"
          disabled={!props.signedIn}
          min={1}
          value={props.options.limit}
          onUpdate={(value) => props.updateOption('limit', value)}
        />
        <NumberField
          label="Min score"
          disabled={!props.signedIn}
          min={0}
          value={props.options.minScore}
          onUpdate={(value) => props.updateOption('minScore', value)}
        />
        <NumberField
          label="Min shares"
          disabled={!props.signedIn}
          min={1}
          value={props.options.minShares}
          onUpdate={(value) => props.updateOption('minShares', value)}
        />
        <NumberField
          label="Feed pages"
          disabled={!props.signedIn}
          min={1}
          value={props.options.maxPages}
          onUpdate={(value) => props.updateOption('maxPages', value)}
        />
        <RefreshField
          checked={props.options.refreshFollows}
          disabled={!props.signedIn}
          onUpdate={(value) => props.updateOption('refreshFollows', value)}
        />
        <DigestActionButtons
          isRunning={props.isRunning}
          onPause={props.onPause}
          onResume={props.onResume}
          pausedRun={props.pausedRun}
          signedIn={props.signedIn}
        />
      </form>
    </div>
  );
}

function DigestIdentity(props: { actorHandle?: string; signedIn: boolean }) {
  return (
    <div class="col-span-full rounded-lg border border-border-subtle bg-surface-raised px-3 py-2.5">
      <div class="flex items-center gap-2 text-[13px] font-semibold text-ink">
        <Icon kind={props.signedIn ? 'user-check' : 'user-search'} class="text-accent" />
        <span>{props.signedIn ? `Digesting @${props.actorHandle}` : 'Preview mode'}</span>
      </div>
      <p class="text-[12px] text-ink-muted">
        {props.signedIn ? 'Skylynx will scan links from your following graph.' : 'Sign in to enable digest controls.'}
      </p>
    </div>
  );
}

function NumberField(props: {
  disabled?: boolean;
  label: string;
  min: number;
  onUpdate: (value: number) => void;
  value: number;
}) {
  return (
    <label>
      {props.label}
      <input
        disabled={props.disabled}
        type="number"
        min={props.min}
        value={props.value}
        onInput={(e) => props.onUpdate(e.currentTarget.valueAsNumber)}
      />
    </label>
  );
}

function RefreshField(props: { checked: boolean; disabled?: boolean; onUpdate: (value: boolean) => void }) {
  return (
    <label class="check col-span-full">
      <input
        type="checkbox"
        checked={props.checked}
        disabled={props.disabled}
        onInput={(e) => props.onUpdate(e.currentTarget.checked)}
      />
      Refresh follows cache
    </label>
  );
}

function DigestActionButtons(props: {
  isRunning: boolean;
  onPause: () => void;
  onResume: () => void;
  pausedRun?: DigestProgressSnapshot;
  signedIn: boolean;
}) {
  return (
    <div class="col-span-full grid grid-cols-2 gap-2">
      <button type="submit" class="btn-primary" disabled={props.isRunning || !props.signedIn}>
        <Icon kind="bolt" />
        {props.isRunning ? 'Running...' : 'Build digest'}
      </button>
      <Show
        when={props.isRunning}
        fallback={
          <button type="button" class="btn-ghost justify-center" disabled={!props.pausedRun} onClick={props.onResume}>
            <Icon kind="player-play" />
            Resume
          </button>
        }>
        <button type="button" class="btn-ghost justify-center" onClick={props.onPause}>
          <Icon kind="player-pause" />
          Pause
        </button>
      </Show>
    </div>
  );
}

function ResultsPanel(props: DigestSectionProps) {
  return (
    <div class="app-results">
      <StatusBar {...props} />
      <ErrorState error={props.error} />
      <EmptyState isEmpty={!props.hasResults && !props.isRunning && !props.error} signedIn={Boolean(props.viewerDid)} />
      <StatusFeed events={props.statusEvents} />
      <Show when={props.isRunning && !props.hasResults}>
        <Running />
      </Show>
      <Show when={props.hasResults}>
        <LinkResults links={props.links} />
      </Show>
    </div>
  );
}

function StatusBar(props: DigestSectionProps) {
  return (
    <div class="flex items-center justify-between gap-4 px-5 py-3.5 border-b border-border-subtle bg-surface-raised">
      <StatusLabel hasResults={props.hasResults} isRunning={props.isRunning} text={props.progressText} />
      <Show when={props.isRunning}>
        <progress max={props.progress.total || 1} value={props.progress.completed} />
      </Show>
      <Show when={props.hasResults && !props.isRunning}>
        <ResultCount count={props.links.length} />
      </Show>
    </div>
  );
}

function StatusLabel(props: { hasResults: boolean; isRunning: boolean; text: string }) {
  return (
    <div class="flex items-center gap-2">
      <StatusIcon hasResults={props.hasResults} isRunning={props.isRunning} />
      <span class="text-[13px] text-ink-muted font-medium">{props.text}</span>
    </div>
  );
}

function StatusIcon(props: { hasResults: boolean; isRunning: boolean }) {
  return (
    <>
      <Show when={props.isRunning}>
        <Icon kind="loader" class="text-accent animate-spin" />
      </Show>
      <Show when={!props.isRunning && props.hasResults}>
        <Icon kind="check" class="text-accent" />
      </Show>
    </>
  );
}

function ResultCount(props: { count: number }) {
  return (
    <span class="text-[12px] text-ink-muted tabular-nums">
      {props.count} link{props.count !== 1 ? 's' : ''}
    </span>
  );
}

function LinkResults(props: { links: DigestLink[] }) {
  return (
    <div class="flex min-h-0 flex-1 flex-col divide-y divide-border-subtle overflow-y-auto">
      <For each={props.links}>{(link, index) => <LinkCard index={index()} link={link} />}</For>
    </div>
  );
}

function StatusFeed(props: { events: StatusItem[] }) {
  return (
    <Show when={props.events.length > 0}>
      <ol class="px-5 py-3 border-b border-border-subtle flex flex-col gap-1.5 max-h-36 overflow-y-auto">
        <For each={props.events.slice(-6)}>{(event) => <StatusFeedItem event={event} />}</For>
      </ol>
    </Show>
  );
}

function StatusFeedItem(props: { event: StatusItem }) {
  return (
    <li class="flex items-center gap-2 text-[12px] text-ink-muted">
      <span class="size-1.5 rounded-full bg-accent shrink-0" />
      <span>{props.event.text}</span>
    </li>
  );
}

function LinkCard(props: { index: number; link: DigestLink }) {
  return (
    <article class="px-5 py-4.5 flex flex-col gap-3 hover:bg-surface-raised transition-colors duration-120">
      <div class="flex items-start gap-2.5">
        <LinkRank index={props.index} />
        <LinkSummary link={props.link} />
      </div>
    </article>
  );
}

function LinkRank(props: { index: number }) {
  return (
    <span class="text-[12px] text-ink-faint tabular-nums font-medium mt-0.75 w-5 shrink-0 text-right">
      {props.index + 1}
    </span>
  );
}

function LinkSummary(props: { link: DigestLink }) {
  return (
    <div class="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] gap-4 max-[760px]:grid-cols-1">
      <div class="flex min-w-0 flex-col gap-1.5">
        <LinkTitle link={props.link} />
        <LinkDescription description={props.link.description} />
        <LinkMetrics link={props.link} />
        <SharerList sharers={props.link.sharers} />
      </div>
      <LinkOgImage link={props.link} />
    </div>
  );
}

function LinkOgImage(props: { link: DigestLink }) {
  const [failed, setFailed] = createSignal(false);
  const imageUri = () => (failed() ? undefined : props.link.ogImageUri);

  return (
    <Show when={imageUri()}>
      {(uri) => (
        <a
          href={props.link.uri}
          target="_blank"
          rel="noopener noreferrer"
          class="block size-24 overflow-hidden rounded-lg border border-border-subtle bg-surface-raised max-[760px]:hidden"
          aria-label="Open link">
          <img src={uri()} alt="" loading="lazy" class="h-full w-full object-cover" onError={() => setFailed(true)} />
        </a>
      )}
    </Show>
  );
}

function LinkTitle(props: { link: DigestLink }) {
  return (
    <h2
      class="text-[15px] font-semibold leading-[1.3] tracking-[-0.005em]"
      style={{ 'font-family': 'var(--font-body)' }}>
      <a
        href={props.link.uri}
        target="_blank"
        rel="noopener noreferrer"
        class="text-ink hover:text-accent transition-colors duration-120 no-underline hover:underline underline-offset-[3px]">
        {props.link.title || props.link.uri}
      </a>
    </h2>
  );
}

function LinkDescription(props: { description: string }) {
  return (
    <Show when={props.description}>
      <p class="text-[13px] text-ink-muted leading-normal line-clamp-2">{props.description}</p>
    </Show>
  );
}

function LinkMetrics(props: { link: DigestLink }) {
  const metrics = () => [
    { label: 'Shares', value: props.link.sharers.length },
    { label: 'Score', value: props.link.score },
    { label: 'Window', value: `${digestTime(props.link.firstSeen)}–${digestTime(props.link.lastSeen)}` }
  ];

  return (
    <dl class="flex gap-4 grid-cols-[unset]">
      <For each={metrics()}>{(metric) => <MetricItem label={metric.label} value={metric.value} />}</For>
    </dl>
  );
}

function MetricItem(props: { label: string; value: number | string }) {
  return (
    <div class="flex flex-col gap-px">
      <dt>{props.label}</dt>
      <dd class="text-[14px]">{props.value}</dd>
    </div>
  );
}

function SharerList(props: { sharers: string[] }) {
  return (
    <ul class="flex flex-wrap gap-1.25 list-none m-0 p-0">
      <For each={props.sharers}>{(sharer) => <li class="tag">@{sharer}</li>}</For>
    </ul>
  );
}

function Running() {
  return (
    <div class="min-h-0 flex-1 overflow-y-auto flex flex-col divide-y divide-border-subtle">
      <For each={Array.from({ length: 5 })}>
        {() => (
          <div class="px-5 py-4.5 flex flex-col gap-2.5">
            <div class="skeleton h-4 w-[75%]" />
            <div class="skeleton h-3 w-[50%]" />
            <div class="flex gap-1.5 mt-0.5">
              <div class="skeleton h-5.5 w-15 rounded-full" />
              <div class="skeleton h-5.5 w-18 rounded-full" />
            </div>
          </div>
        )}
      </For>
    </div>
  );
}

function DatePickerField(props: {
  disabled?: boolean;
  label: string;
  onUpdate: (value?: string) => void;
  value?: string;
}) {
  const [container, setContainer] = createSignal<HTMLDivElement>();
  const [isOpen, setIsOpen] = createSignal(false);
  const [visibleMonth, setVisibleMonth] = createSignal(monthStart(props.value ? new Date(props.value) : new Date()));
  const value = createMemo(() => localDatePickerValue(props.value));

  const selectDate = (date: string) => {
    const current = value() ?? defaultDatePickerValue();
    props.onUpdate(localPickerToIso({ ...current, date }));
  };

  const updateTime = (part: 'hour' | 'minute', nextValue: number) => {
    const current = value() ?? { ...defaultDatePickerValue(), date: localDateString(new Date()) };
    props.onUpdate(localPickerToIso({ ...current, [part]: nextValue }));
  };

  const moveMonth = (delta: number) => setVisibleMonth(addMonths(visibleMonth(), delta));

  onMount(() => {
    const closeOnOutsidePointerDown = outsidePointerDownHandler(container, isOpen, () => setIsOpen(false));
    document.addEventListener('pointerdown', closeOnOutsidePointerDown);
    onCleanup(() => document.removeEventListener('pointerdown', closeOnOutsidePointerDown));
  });

  return (
    <div ref={setContainer} class="relative flex flex-col gap-1.5 min-w-0">
      <span class="text-[13px] text-ink-muted">{props.label}</span>
      <DatePickerButton
        disabled={props.disabled}
        isOpen={isOpen()}
        label={props.label}
        value={value()}
        onToggle={() => setIsOpen(!isOpen())}
      />
      <Show when={isOpen()}>
        <DatePickerPopover
          month={visibleMonth()}
          onClear={() => props.onUpdate()}
          onMoveMonth={moveMonth}
          onSelectDate={selectDate}
          onUpdateTime={updateTime}
          selected={value()}
        />
      </Show>
    </div>
  );
}

function DatePickerButton(props: {
  disabled?: boolean;
  isOpen: boolean;
  label: string;
  onToggle: () => void;
  value?: DatePickerValue;
}) {
  return (
    <button
      type="button"
      disabled={props.disabled}
      class="bg-surface border border-border rounded-[7px] min-h-9.5 px-2.5 text-left text-ink flex items-center justify-between gap-2 w-full hover:border-ink-faint transition-colors duration-150"
      aria-expanded={props.isOpen}
      onClick={props.onToggle}>
      <span class={props.value ? 'truncate' : 'truncate text-ink-muted'}>
        {props.value ? formatPickerValue(props.value) : `Choose ${props.label.toLowerCase()}`}
      </span>
      <Icon kind="calendar" class="text-ink-muted" />
    </button>
  );
}

function DatePickerPopover(props: {
  month: Date;
  onClear: () => void;
  onMoveMonth: (delta: number) => void;
  onSelectDate: (date: string) => void;
  onUpdateTime: (part: 'hour' | 'minute', value: number) => void;
  selected?: DatePickerValue;
}) {
  return (
    <div class="absolute left-0 top-full z-30 mt-2 w-80 max-w-[calc(100vw-3rem)] bg-surface-raised border border-border rounded-xl p-3 flex flex-col gap-3 shadow-[0_18px_48px_#00000066]">
      <DatePickerHeader month={props.month} onMoveMonth={props.onMoveMonth} />
      <CalendarGrid month={props.month} onSelectDate={props.onSelectDate} selectedDate={props.selected?.date} />
      <TimeControls selected={props.selected} onClear={props.onClear} onUpdateTime={props.onUpdateTime} />
    </div>
  );
}

function DatePickerHeader(props: { month: Date; onMoveMonth: (delta: number) => void }) {
  return (
    <div class="flex items-center justify-between">
      <IconButton icon="chevron-left" label="Previous month" onClick={() => props.onMoveMonth(-1)} />
      <span class="text-[13px] font-semibold text-ink">{monthLabel(props.month)}</span>
      <IconButton icon="chevron-right" label="Next month" onClick={() => props.onMoveMonth(1)} />
    </div>
  );
}

function IconButton(props: { icon: IconKind; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      aria-label={props.label}
      class="size-9 rounded-lg border border-border bg-surface flex items-center justify-center text-ink-muted hover:text-ink hover:border-ink-faint transition-colors duration-150"
      onClick={props.onClick}>
      <Icon kind={props.icon} />
    </button>
  );
}

function CalendarGrid(props: { month: Date; onSelectDate: (date: string) => void; selectedDate?: string }) {
  const days = createMemo(() => calendarDays(props.month));

  return (
    <div class="grid grid-cols-7 gap-1">
      <For each={WEEKDAYS}>{(day) => <CalendarWeekday day={day} />}</For>
      <For each={days()}>
        {(day) => <CalendarDay day={day} selectedDate={props.selectedDate} onSelect={props.onSelectDate} />}
      </For>
    </div>
  );
}

function CalendarWeekday(props: { day: string }) {
  return <div class="h-6 flex items-center justify-center text-[11px] text-ink-faint font-medium">{props.day}</div>;
}

function CalendarDay(props: { day: MonthDay; onSelect: (date: string) => void; selectedDate?: string }) {
  const isSelected = () => props.selectedDate === props.day.isoDate;

  return (
    <button
      type="button"
      class={calendarDayClass(props.day, isSelected())}
      onClick={() => props.onSelect(props.day.isoDate)}>
      {props.day.date.getDate()}
    </button>
  );
}

function TimeControls(props: {
  onClear: () => void;
  onUpdateTime: (part: 'hour' | 'minute', value: number) => void;
  selected?: DatePickerValue;
}) {
  return (
    <div class="grid grid-cols-[1fr_1fr_auto] items-end gap-2">
      <TimeNumber
        label="Hour"
        max={23}
        value={props.selected?.hour ?? 0}
        onUpdate={(value) => props.onUpdateTime('hour', value)}
      />
      <TimeNumber
        label="Minute"
        max={59}
        value={props.selected?.minute ?? 0}
        onUpdate={(value) => props.onUpdateTime('minute', value)}
      />
      <button
        type="button"
        class="h-9 px-3 rounded-lg border border-border bg-surface text-[13px] text-ink-muted hover:text-ink hover:border-ink-faint transition-colors duration-150"
        onClick={props.onClear}>
        Clear
      </button>
    </div>
  );
}

function TimeNumber(props: { label: string; max: number; onUpdate: (value: number) => void; value: number }) {
  return (
    <label class="flex-1">
      {props.label}
      <input
        type="number"
        min="0"
        max={props.max}
        value={props.value}
        onInput={(event) => props.onUpdate(clampInteger(event.currentTarget.valueAsNumber, 0, props.max))}
      />
    </label>
  );
}

function SignInActorField(props: { onUpdate: (value: string) => void; value: string }) {
  let requestId = 0;
  const [container, setContainer] = createSignal<HTMLDivElement>();
  const [isOpen, setIsOpen] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(false);
  const [suggestions, setSuggestions] = createSignal<ActorSuggestion[]>([]);

  const search = debounce(async (query: string) => {
    const currentRequestId = (requestId += 1);
    const trimmed = query.trim();

    if (trimmed.length < 2) {
      setSuggestions([]);
      setIsLoading(false);
      return;
    }

    setIsLoading(true);

    try {
      const actors = await searchActorsTypeahead({ query: trimmed });
      if (currentRequestId !== requestId) return;

      setSuggestions(actors);
      setIsOpen(true);
    } catch {
      if (currentRequestId !== requestId) return;
      setSuggestions([]);
    } finally {
      if (currentRequestId === requestId) setIsLoading(false);
    }
  }, 250);

  const updateActor = (value: string) => {
    props.onUpdate(value);
    search(value);
  };

  const selectActor = (actor: ActorSuggestion) => {
    props.onUpdate(actor.handle);
    setSuggestions([]);
    setIsOpen(false);
  };

  onMount(() => {
    const closeOnOutsidePointerDown = outsidePointerDownHandler(container, isOpen, () => setIsOpen(false));
    document.addEventListener('pointerdown', closeOnOutsidePointerDown);
    onCleanup(() => document.removeEventListener('pointerdown', closeOnOutsidePointerDown));
  });

  return (
    <div ref={setContainer} class="relative min-w-0 flex-1">
      <div class="relative">
        <input
          type="text"
          autocomplete="off"
          aria-autocomplete="list"
          aria-expanded={isOpen()}
          placeholder="handle.bsky.social"
          required
          value={props.value}
          onFocus={() => setIsOpen(suggestions().length > 0)}
          onInput={(event) => updateActor(event.currentTarget.value)}
        />
        <Show when={isLoading()}>
          <Icon kind="loader" class="absolute right-3 top-1/2 -translate-y-1/2 text-ink-muted animate-spin" />
        </Show>
      </div>
      <Show when={isOpen() && suggestions().length > 0}>
        <ActorSuggestions actors={suggestions()} onSelect={selectActor} />
      </Show>
    </div>
  );
}

function ActorSuggestions(props: { actors: ActorSuggestion[]; onSelect: (actor: ActorSuggestion) => void }) {
  return (
    <div class="absolute left-0 right-0 top-full z-30 mt-2 overflow-hidden rounded-xl border border-border bg-surface-raised shadow-[0_18px_48px_#00000066]">
      <ul class="m-0 list-none p-1">
        <For each={props.actors}>{(actor) => <ActorSuggestionItem actor={actor} onSelect={props.onSelect} />}</For>
      </ul>
    </div>
  );
}

function ActorSuggestionItem(props: { actor: ActorSuggestion; onSelect: (actor: ActorSuggestion) => void }) {
  return (
    <li>
      <button
        type="button"
        class="grid w-full grid-cols-[2rem_1fr] items-center gap-3 rounded-lg border-0 bg-transparent px-2.5 py-2 text-left hover:bg-accent-glow"
        onClick={() => props.onSelect(props.actor)}>
        <ActorAvatar actor={props.actor} />
        <ActorSuggestionText actor={props.actor} />
      </button>
    </li>
  );
}

function ActorAvatar(props: { actor: ActorSuggestion }) {
  return (
    <Show
      when={props.actor.avatar}
      fallback={
        <span class="flex size-8 items-center justify-center rounded-full bg-tag-bg text-[12px] text-accent">@</span>
      }>
      {(avatar) => <img alt="" class="size-8 rounded-full object-cover" src={avatar()} />}
    </Show>
  );
}

function ActorSuggestionText(props: { actor: ActorSuggestion }) {
  return (
    <span class="min-w-0">
      <span class="block truncate text-[13px] font-semibold text-ink">
        {props.actor.displayName || props.actor.handle}
      </span>
      <span class="block truncate text-[12px] text-ink-muted">@{props.actor.handle}</span>
    </span>
  );
}

function ErrorState(props: { error?: string }) {
  const error = () => props.error;
  return (
    <Show when={error()}>
      {(e) => (
        <div class="flex items-start gap-2.5 px-5 py-4 border-b border-border-subtle">
          <Icon kind="warning" class="text-danger mt-1" />
          <p class="text-danger text-[14px]">{e()}</p>
        </div>
      )}
    </Show>
  );
}

function EmptyState(props: { isEmpty: boolean; signedIn: boolean }) {
  const isEmpty = () => props.isEmpty;
  return (
    <Show when={isEmpty()}>
      <div class="flex-1 flex flex-col items-center justify-center gap-3 px-8 py-12 text-center">
        <Icon kind="link" class="text-ink-faint text-[40px]" />
        <p class="text-ink-muted text-[14px] max-w-65">
          {props.signedIn ? 'Build a digest to see shared links.' : 'Sign in to preview your network link digest.'}
        </p>
      </div>
    </Show>
  );
}

const normalizedOptions = (options: LinkDigestOptions): LinkDigestOptions => ({
  ...options,
  actor: options.actor.trim(),
  feedLimit: clampInteger(options.feedLimit, 1, 100),
  limit: clampInteger(options.limit, 1, 100),
  maxPages: clampInteger(options.maxPages, 1, 20),
  minScore: clampInteger(options.minScore, 0, 100_000),
  minShares: clampInteger(options.minShares, 1, 100_000)
});

const digestLinksFromSnapshot = (run: DigestProgressSnapshot) => {
  return aggregateDigestLinks(run.posts)
    .filter((link) => link.score >= run.options.minScore)
    .filter((link) => link.sharers.length >= run.options.minShares)
    .toSorted(compareDigestLinks)
    .slice(0, run.options.limit);
};

const compareDigestLinks = (left: DigestLink, right: DigestLink) => {
  const shareOrder = right.sharers.length - left.sharers.length;
  if (shareOrder !== 0) return shareOrder;

  const scoreOrder = right.score - left.score;
  if (scoreOrder !== 0) return scoreOrder;

  const titleOrder = left.title.localeCompare(right.title);
  if (titleOrder !== 0) return titleOrder;

  return left.uri.localeCompare(right.uri);
};

const digestTime = (value: string) => value.slice(11, 16) || value;
const clampInteger = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, Math.trunc(value || min)));

const formatCount = (count: number) => {
  if (count < 1000) return String(count);
  return new Intl.NumberFormat(undefined, { maximumFractionDigits: 1, notation: 'compact' }).format(count);
};

const formatDigestDate = (iso: string) => {
  return new Intl.DateTimeFormat(undefined, {
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    month: 'short',
    year: 'numeric'
  }).format(new Date(iso));
};

const formatWindow = (since?: string, until?: string) => {
  if (since && until) return `${shortDate(since)} to ${shortDate(until)}`;
  if (since) return `Since ${shortDate(since)}`;
  if (until) return `Until ${shortDate(until)}`;
  return 'All scanned posts';
};

const shortDate = (iso: string) => {
  return new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', year: 'numeric' }).format(new Date(iso));
};

const WEEKDAYS = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];

const progressFromEvent = (event: LinkDigestStatusEvent): LinkDigestProgress => {
  if (event.type === 'resolving-actor' || event.type === 'actor-resolved') {
    return { completed: 0, phase: 'resolving', total: 0 };
  }

  if (event.type === 'loading-follows' || event.type === 'follows-loaded') {
    return { completed: 0, phase: 'fetching-follows', total: event.type === 'follows-loaded' ? event.count : 0 };
  }

  if (event.type === 'fetching-feeds' || event.type === 'follow-feed-fetched') {
    return { completed: event.completed, phase: 'fetching-feeds', total: event.total };
  }

  if (event.type === 'paused') {
    return { completed: event.completed, phase: 'paused', total: event.total };
  }

  if (event.type === 'done') {
    return { completed: event.result.follows.length, phase: 'done', total: event.result.follows.length };
  }

  return { completed: 0, phase: 'fetching-feeds', total: 0 };
};

const statusText = (event: LinkDigestStatusEvent) => {
  if (event.type === 'resolving-actor') return `Resolving ${event.actor}`;
  if (event.type === 'actor-resolved') return `Resolved @${event.actor.handle}`;
  if (event.type === 'loading-follows')
    return event.refresh ? 'Refreshing follows from Bluesky' : 'Checking follows cache';
  if (event.type === 'follows-loaded') return `Loaded ${event.count} follows from ${event.source}`;
  if (event.type === 'fetching-feeds') return `Starting feed scan for ${event.total} follows`;
  if (event.type === 'follow-feed-fetched') return `@${event.follow.handle}: ${event.linkCount} links`;
  if (event.type === 'caching-posts') return `Caching ${event.count} link shares`;
  if (event.type === 'digest-ready') return `Ranked ${event.linkCount} links from ${event.postCount} shares`;
  if (event.type === 'paused') return `Paused after ${event.completed} of ${event.total} follows`;
  return 'Digest complete';
};

const localDatePickerValue = (iso?: string): DatePickerValue | undefined => {
  if (!iso) return;

  const date = new Date(iso);
  return { date: localDateString(date), hour: date.getHours(), minute: date.getMinutes() };
};

const defaultDatePickerValue = (): DatePickerValue => {
  const date = new Date();
  return { date: localDateString(date), hour: 0, minute: 0 };
};

const localPickerToIso = (value: DatePickerValue) => {
  return new Date(`${value.date}T${pad(value.hour)}:${pad(value.minute)}:00`).toISOString();
};

const localDateString = (date: Date) => {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
};

const pad = (value: number) => String(value).padStart(2, '0');
const monthStart = (date: Date) => new Date(date.getFullYear(), date.getMonth(), 1);
const addMonths = (date: Date, delta: number) => new Date(date.getFullYear(), date.getMonth() + delta, 1);
const monthLabel = (date: Date) => new Intl.DateTimeFormat(undefined, { month: 'long', year: 'numeric' }).format(date);

const calendarDays = (month: Date): MonthDay[] => {
  const start = monthStart(month);
  const first = new Date(start);
  first.setDate(1 - start.getDay());

  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(first);
    date.setDate(first.getDate() + index);
    return { date, inMonth: date.getMonth() === month.getMonth(), isoDate: localDateString(date) };
  });
};

const calendarDayClass = (day: MonthDay, selected: boolean) => {
  const base =
    'h-9 min-w-0 rounded-lg text-[13px] flex items-center justify-center border tabular-nums transition-colors duration-120';
  if (selected) return `${base} bg-accent text-bg border-accent font-semibold`;
  if (!day.inMonth) return `${base} bg-transparent text-ink-faint border-transparent hover:text-ink-muted`;
  return `${base} bg-surface text-ink border-border hover:border-accent hover:bg-accent-glow`;
};

const outsidePointerDownHandler = (
  container: Accessor<HTMLElement | undefined>,
  isOpen: Accessor<boolean>,
  close: () => void
) => {
  return (event: PointerEvent) => {
    if (!isOpen()) return;
    if (container()?.contains(event.target as Node)) return;

    close();
  };
};

const debounce = <Arguments extends unknown[]>(callback: (...args: Arguments) => void, delayMs: number) => {
  let timeout: ReturnType<typeof setTimeout> | undefined;

  return (...args: Arguments) => {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => callback(...args), delayMs);
  };
};

const formatPickerValue = (value: DatePickerValue) => {
  const date = new Date(`${value.date}T${pad(value.hour)}:${pad(value.minute)}:00`);
  return new Intl.DateTimeFormat(undefined, {
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    month: 'short',
    year: 'numeric'
  }).format(date);
};
