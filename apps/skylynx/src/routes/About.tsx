import { For } from 'solid-js';
import { A } from '@solidjs/router';
import { Icon, type IconKind } from '../components/Icon';

export function About() {
  return (
    <div class="max-w-350 mx-auto w-full flex flex-col gap-24">
      <PageHero />
      <div class="px-[clamp(20px,4vw,56px)] flex flex-col gap-24 pb-16">
        <HowItWorks />
        <Ethos />
        <Philosophy />
        <CallToAction />
      </div>
    </div>
  );
}

function PageHero() {
  return (
    <section class="hero-graph px-[clamp(20px,4vw,56px)] pt-16 pb-12 max-w-none">
      <div class="max-w-180 flex flex-col gap-5">
        <span class="text-[11px] font-semibold tracking-widest uppercase text-accent px-2 py-0.75 rounded-full border border-tag-border bg-tag-bg self-start">
          About Skylynx
        </span>
        <h1 class="text-[clamp(32px,4.5vw,52px)] leading-[1.1] tracking-[-0.02em] text-ink font-semibold">
          Reading shaped by people,
          <br />
          <em class="text-accent not-italic">not by algorithms.</em>
        </h1>
        <p class="text-[17px] text-ink-muted leading-[1.75] max-w-140">
          Skylynx is a link aggregator built on the AT Protocol. It surfaces what the people you follow are reading and
          sharing — without engagement scores, recommendation engines, or ads deciding what you see.
        </p>
      </div>
    </section>
  );
}

/* ── How it works ───────────────────────────── */

function HowItWorks() {
  return (
    <section class="flex flex-col gap-8">
      <SectionLabel>How it works</SectionLabel>
      <div class="grid gap-4 grid-cols-[repeat(auto-fit,minmax(220px,1fr))]">
        <For each={HOW_IT_WORKS}>{(step) => <StepCard step={step} />}</For>
      </div>
    </section>
  );
}

function StepCard(props: { step: (typeof HOW_IT_WORKS)[number] }) {
  return (
    <div class="flex flex-col gap-3 p-5 bg-surface border border-border-subtle rounded-[10px]">
      <Icon kind={props.step.icon} class="text-accent text-[22px]" />
      <h3 class="text-[14px] font-semibold text-ink">{props.step.title}</h3>
      <p class="text-[13px] text-ink-muted leading-[1.65]">{props.step.body}</p>
    </div>
  );
}

function Ethos() {
  return (
    <section class="flex flex-col gap-10">
      <SectionLabel>The idea</SectionLabel>
      <div class="grid gap-px bg-border-subtle border border-border-subtle rounded-xl overflow-hidden">
        <For each={ETHOS_ITEMS}>{(item) => <EthosRow item={item} />}</For>
      </div>
    </section>
  );
}

function EthosRow(props: { item: (typeof ETHOS_ITEMS)[number] }) {
  return (
    <div class="flex gap-6 p-6 bg-surface max-[700px]:flex-col max-[700px]:gap-3">
      <EthosIcon kind={props.item.icon} />
      <EthosBody item={props.item} />
    </div>
  );
}

function EthosIcon(props: { kind: IconKind }) {
  return (
    <div class="shrink-0 w-10 h-10 rounded-lg bg-tag-bg border border-tag-border flex items-center justify-center">
      <Icon kind={props.kind} class="text-accent text-[18px]" />
    </div>
  );
}

function EthosBody(props: { item: (typeof ETHOS_ITEMS)[number] }) {
  return (
    <div class="flex flex-col gap-1.5">
      <h3 class="text-[15px] font-semibold text-ink tracking-[-0.01em]">{props.item.title}</h3>
      <p class="text-[14px] text-ink-muted leading-[1.7]">{props.item.body}</p>
    </div>
  );
}

function Philosophy() {
  return (
    <section class="flex flex-col gap-10 max-w-200">
      <SectionLabel>Why this exists</SectionLabel>
      <div class="flex flex-col gap-8">
        <For each={PHILOSOPHY_BLOCKS}>{(block) => <PhilosophyBlock block={block} />}</For>
      </div>
    </section>
  );
}

function PhilosophyBlock(props: { block: (typeof PHILOSOPHY_BLOCKS)[number] }) {
  return (
    <div class="flex flex-col gap-3 border-l-2 border-accent/30 pl-6">
      <h3
        class="text-[20px] font-semibold text-ink leading-[1.2] tracking-[-0.015em]"
        style={{ 'font-family': 'var(--font-display)' }}>
        {props.block.heading}
      </h3>
      <p class="text-[15px] text-ink-muted leading-[1.75]">{props.block.body}</p>
    </div>
  );
}

function CallToAction() {
  return (
    <section class="flex flex-col gap-5 p-8 bg-surface border border-border rounded-xl max-w-160">
      <h2
        class="text-[24px] font-semibold text-ink leading-[1.2] tracking-[-0.015em]"
        style={{ 'font-family': 'var(--font-display)' }}>
        Start with your own network
      </h2>
      <p class="text-[15px] text-ink-muted leading-[1.7]">
        Enter any Bluesky handle and Skylynx will map the links being shared across that actor's follows — no account
        required, no data leaves your browser.
      </p>
      <A href="/app" class="btn-primary self-start w-auto! px-5 no-underline">
        <Icon kind="bolt" />
        Build a digest
      </A>
    </section>
  );
}

function SectionLabel(props: { children: string }) {
  return <h2 class="text-[13px] font-semibold tracking-[0.08em] uppercase text-ink-muted">{props.children}</h2>;
}

const HOW_IT_WORKS: { icon: IconKind; title: string; body: string }[] = [
  {
    icon: 'user-search',
    title: 'Resolve the actor',
    body: 'Enter any Bluesky handle. Skylynx resolves their DID and fetches their full follows graph from the public API.'
  },
  {
    icon: 'database',
    title: 'Cache locally',
    body: 'Follows and posts are stored in IndexedDB so repeat runs are fast. Toggle "Refresh follows" to pull fresh data.'
  },
  {
    icon: 'chart-bar',
    title: 'Score & rank',
    body: 'Links are scored by likes, reposts, and share count. Set thresholds to filter signal from noise.'
  },
  {
    icon: 'link',
    title: 'Surface the digest',
    body: 'The top external URLs from your network appear ranked — see who shared each one and when.'
  }
];

const ETHOS_ITEMS: { icon: IconKind; title: string; body: string }[] = [
  {
    icon: 'users',
    title: 'Curation by follows, not feeds',
    body: "Your follows are a deliberate act. Each one is a signal that you trust that person's taste. Skylynx treats your follows graph as a collaborative filter — the oldest and most honest recommendation engine on the internet."
  },
  {
    icon: 'bookmarks',
    title: 'Long-form and considered reading',
    body: 'External links on Bluesky skew toward articles, essays, papers, and deep-dives. Skylynx is designed around that — it surfaces content worth reading properly, not content optimised for quick engagement.'
  },
  {
    icon: 'chart-bar',
    title: 'Shared by many, not shouted by one',
    body: 'A link that twelve people in your network posted independently carries more signal than one post with a thousand likes. Skylynx ranks by breadth of sharing across your graph, not by individual reach.'
  },
  {
    icon: 'lock',
    title: 'Your data stays in your browser',
    body: "All caching happens in your browser's IndexedDB. Nothing is sent to a server. Skylynx is a tool you run, not a service you subscribe to."
  }
];

const PHILOSOPHY_BLOCKS = [
  {
    heading: 'The timeline is not the whole picture',
    body: 'A chronological feed is a firehose. Most of what matters gets buried. Skylynx steps back from the stream and asks a different question: across all the people I follow, what have they collectively decided is worth sharing with the world? The answer is usually more interesting than what appeared in your timeline this morning.'
  },
  {
    heading: 'Important reading has always been social',
    body: 'Before algorithmic feeds, people passed articles to each other. They forwarded emails, clipped magazines, left bookmarks in library copies. Skylynx is a digital version of that — a way of seeing what the people you respect are pointing at, aggregated into something you can actually sit down with.'
  },
  {
    heading: 'The AT Protocol makes this possible cleanly',
    body: "Because Bluesky runs on an open protocol with a public API, Skylynx can read the social graph without special access, without scraping, and without depending on a platform's continued goodwill. The data is yours and the network's — Skylynx just helps you read it differently."
  }
];
