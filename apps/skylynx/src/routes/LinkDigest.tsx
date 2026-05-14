import { For, Show, createSignal } from 'solid-js';
import { buildLinkDigest } from '../lib/api/link-digest';
import type { DigestLink, LinkDigestOptions, LinkDigestProgress } from '../lib/types';
import { Icon } from '../components/Icon';

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
  isRunning: boolean;
  links: DigestLink[];
  onSubmit: (event: SubmitEvent) => void;
  options: LinkDigestOptions;
  progress: LinkDigestProgress;
  progressText: string;
  updateOption: UpdateOption;
};

export function LinkDigest() {
  const [options, setOptions] = createSignal(DEFAULT_OPTIONS);
  const [links, setLinks] = createSignal<DigestLink[]>([]);
  const [progress, setProgress] = createSignal<LinkDigestProgress>({ completed: 0, phase: 'idle', total: 0 });
  const [error, setError] = createSignal('');
  const [isRunning, setIsRunning] = createSignal(false);

  const runDigest = async (event: SubmitEvent) => {
    event.preventDefault();
    setError('');
    setLinks([]);
    setIsRunning(true);

    try {
      const result = await buildLinkDigest(normalizedOptions(options()), setProgress);
      setLinks(result.links);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Digest failed');
    } finally {
      setIsRunning(false);
    }
  };

  const progressText = () => {
    const p = progress();
    if (p.phase === 'idle') return 'Ready';
    if (p.phase === 'resolving') return 'Resolving actor…';
    if (p.phase === 'fetching-follows') return 'Fetching follows…';
    if (p.phase === 'done') return `Done — scanned ${p.total} follows`;
    return `Scanning feeds ${p.completed} / ${p.total}`;
  };

  const hasResults = () => links().length > 0;
  const updateOption: UpdateOption = (key, value) => setOptions({ ...options(), [key]: value });

  return (
    <div class="px-[clamp(20px,4vw,56px)] max-w-350 mx-auto w-full py-12 flex flex-col gap-14">
      <Hero />
      <DigestSection
        error={error()}
        hasResults={hasResults()}
        isRunning={isRunning()}
        links={links()}
        onSubmit={runDigest}
        options={options()}
        progress={progress()}
        progressText={progressText()}
        updateOption={updateOption}
      />
    </div>
  );
}

function Hero() {
  return (
    <section class="hero-graph -mx-[clamp(20px,4vw,56px)] px-[clamp(20px,4vw,56px)] pt-12 pb-8 max-w-none">
      <div class="max-w-170 flex flex-col gap-5">
        <Eyebrow />
        <HeroTitle />
        <p class="text-[17px] text-ink-muted leading-[1.7] max-w-140">
          Skylynx scans the feeds of everyone an actor follows on Bluesky and surfaces the external links they share most
          — a social reading digest built from your corner of the network.
        </p>
        <HeroStats />
      </div>
    </section>
  );
}

function Eyebrow() {
  return (
    <span class="text-[11px] font-semibold tracking-widest uppercase text-accent px-2 py-0.75 rounded-full border border-tag-border bg-tag-bg">
      AT Protocol
    </span>
  );
}

function HeroTitle() {
  return (
    <h1 class="text-[clamp(36px,5vw,56px)] leading-[1.08] tracking-[-0.02em] text-ink font-semibold">
      What is your network
      <br />
      <em class="text-accent not-italic">reading right now?</em>
    </h1>
  );
}

function HeroStats() {
  return (
    <div class="flex items-center gap-6 text-[13px] text-ink-muted">
      <HeroStat icon="users" label="Follows graph" />
      <HeroStat icon="database" label="Cached locally" />
      <HeroStat icon="lock" label="Public API only" />
    </div>
  );
}

function HeroStat(props: { icon: IconKind; label: string }) {
  return (
    <span class="flex items-center gap-1.5">
      <Icon kind={props.icon} class="text-accent" />
      {props.label}
    </span>
  );
}

function DigestSection(props: DigestSectionProps) {
  return (
    <section class="flex flex-col gap-6">
      <SectionTitle />
      <div class="grid gap-5 grid-cols-[minmax(300px,380px)_minmax(0,1fr)] max-[900px]:grid-cols-[1fr]">
        <DigestControls
          isRunning={props.isRunning}
          onSubmit={props.onSubmit}
          options={props.options}
          updateOption={props.updateOption}
        />
        <ResultsPanel {...props} />
      </div>
    </section>
  );
}

function SectionTitle() {
  return (
    <div class="flex items-center gap-2.5">
      <span class="flex items-center text-accent">
        <i class="i-tabler-bolt"></i>
      </span>
      <h2 class="text-[18px] font-semibold tracking-[-0.01em] text-ink" style={{ 'font-family': 'var(--font-body)' }}>
        Build a digest
      </h2>
    </div>
  );
}

function DigestControls(props: {
  isRunning: boolean;
  onSubmit: (event: SubmitEvent) => void;
  options: LinkDigestOptions;
  updateOption: UpdateOption;
}) {
  return (
    <div class="bg-surface border border-border rounded-xl p-6 self-start flex flex-col gap-5">
      <form class="grid gap-3.5 grid-cols-2" onSubmit={props.onSubmit}>
        <ActorField options={props.options} onUpdate={(value) => props.updateOption('actor', value)} />
        <DateTimeField kind="since" onUpdate={(value) => props.updateOption('since', value)} />
        <DateTimeField kind="until" onUpdate={(value) => props.updateOption('until', value)} />
        <NumberField
          label="Max links"
          min={1}
          value={props.options.limit}
          onUpdate={(value) => props.updateOption('limit', value)}
        />
        <NumberField
          label="Min score"
          min={0}
          value={props.options.minScore}
          onUpdate={(value) => props.updateOption('minScore', value)}
        />
        <NumberField
          label="Min shares"
          min={1}
          value={props.options.minShares}
          onUpdate={(value) => props.updateOption('minShares', value)}
        />
        <NumberField
          label="Feed pages"
          min={1}
          value={props.options.maxPages}
          onUpdate={(value) => props.updateOption('maxPages', value)}
        />
        <RefreshField
          checked={props.options.refreshFollows}
          onUpdate={(value) => props.updateOption('refreshFollows', value)}
        />
        <SubmitButton isRunning={props.isRunning} />
      </form>
    </div>
  );
}

function NumberField(props: { label: string; min: number; onUpdate: (value: number) => void; value: number }) {
  return (
    <label>
      {props.label}
      <input
        type="number"
        min={props.min}
        value={props.value}
        onInput={(e) => props.onUpdate(e.currentTarget.valueAsNumber)}
      />
    </label>
  );
}

function RefreshField(props: { checked: boolean; onUpdate: (value: boolean) => void }) {
  return (
    <label class="check col-span-full">
      <input type="checkbox" checked={props.checked} onInput={(e) => props.onUpdate(e.currentTarget.checked)} />
      Refresh follows cache
    </label>
  );
}

function SubmitButton(props: { isRunning: boolean }) {
  return (
    <button type="submit" class="btn-primary col-span-full" disabled={props.isRunning}>
      <Icon kind="bolt" />
      {props.isRunning ? 'Running…' : 'Build digest'}
    </button>
  );
}

function ResultsPanel(props: DigestSectionProps) {
  return (
    <div class="bg-surface border border-border rounded-xl overflow-hidden flex flex-col min-h-120">
      <StatusBar {...props} />
      <ErrorState error={props.error} />
      <EmptyState isEmpty={!props.hasResults && !props.isRunning && !props.error} />
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
    <div class="flex flex-col divide-y divide-border-subtle overflow-y-auto">
      <For each={props.links}>{(link, index) => <LinkCard index={index()} link={link} />}</For>
    </div>
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
    <div class="flex flex-col gap-1.5 min-w-0">
      <LinkTitle link={props.link} />
      <LinkDescription description={props.link.description} />
      <LinkMetrics link={props.link} />
      <SharerList sharers={props.link.sharers} />
    </div>
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
    <div class="flex flex-col divide-y divide-border-subtle">
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

function DateTimeField(props: { kind: 'since' | 'until'; onUpdate: (value?: string) => void }) {
  return (
    <label>
      <Show when={props.kind === 'since'} fallback="Until">
        Since
      </Show>
      <input
        type="datetime-local"
        onInput={(e) => {
          const v = e.currentTarget.value;
          props.onUpdate(isoDatetime(v));
        }}
      />
    </label>
  );
}

function ActorField(props: { options: LinkDigestOptions; onUpdate: (value: string) => void }) {
  const options = () => props.options;
  return (
    <label class="col-span-full">
      Actor handle
      <input
        type="text"
        autocomplete="off"
        placeholder="handle.bsky.social"
        required
        value={options().actor}
        onInput={(e) => props.onUpdate(e.currentTarget.value)}
      />
    </label>
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

function EmptyState(props: { isEmpty: boolean }) {
  const isEmpty = () => props.isEmpty;
  return (
    <Show when={isEmpty()}>
      <div class="flex-1 flex flex-col items-center justify-center gap-3 px-8 py-12 text-center">
        <Icon kind="link" class="text-ink-faint text-[40px]" />
        <p class="text-ink-muted text-[14px] max-w-65">Enter an actor handle and build a digest to see shared links.</p>
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

const isoDatetime = (value: string) => {
  if (!value) return;
  return new Date(value).toISOString();
};

const digestTime = (value: string) => value.slice(11, 16) || value;
const clampInteger = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, Math.trunc(value || min)));
