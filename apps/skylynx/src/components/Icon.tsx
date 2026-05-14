import { type JSX, Match, splitProps, Switch } from 'solid-js';

export type IconKind =
  | 'user-search'
  | 'chart-bar'
  | 'link'
  | 'bluesky'
  | 'bolt'
  | 'users'
  | 'bookmarks'
  | 'lock'
  | 'loader'
  | 'check'
  | 'warning'
  | 'database'
  | 'calendar'
  | 'chevron-left'
  | 'chevron-right'
  | 'player-pause'
  | 'player-play'
  | 'refresh'
  | 'user'
  | 'user-check'
  | 'x'
  | 'layout-sidebar-right';

export function Icon(props: { kind: IconKind } & JSX.HTMLAttributes<HTMLSpanElement>) {
  const [local, rest] = splitProps(props, ['kind']);
  return (
    <Switch>
      <Match when={local.kind === 'link'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-link" />
        </span>
      </Match>
      <Match when={local.kind === 'check'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-check" />
        </span>
      </Match>
      <Match when={local.kind === 'loader'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-loader-2" />
        </span>
      </Match>
      <Match when={local.kind === 'warning'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-alert-circle" />
        </span>
      </Match>
      <Match when={local.kind === 'chart-bar'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-chart-bar" />
        </span>
      </Match>
      <Match when={local.kind === 'user-search'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-user-search" />
        </span>
      </Match>
      <Match when={local.kind === 'bluesky'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-brand-bluesky" />
        </span>
      </Match>
      <Match when={local.kind === 'bolt'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-bolt" />
        </span>
      </Match>
      <Match when={local.kind === 'users'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-users" />
        </span>
      </Match>
      <Match when={local.kind === 'bookmarks'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-bookmarks" />
        </span>
      </Match>
      <Match when={local.kind === 'lock'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-lock" />
        </span>
      </Match>
      <Match when={local.kind === 'database'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-database" />
        </span>
      </Match>
      <Match when={local.kind === 'calendar'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-calendar" />
        </span>
      </Match>
      <Match when={local.kind === 'chevron-left'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-chevron-left" />
        </span>
      </Match>
      <Match when={local.kind === 'chevron-right'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-chevron-right" />
        </span>
      </Match>
      <Match when={local.kind === 'refresh'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-refresh" />
        </span>
      </Match>
      <Match when={local.kind === 'player-pause'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-player-pause" />
        </span>
      </Match>
      <Match when={local.kind === 'player-play'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-player-play" />
        </span>
      </Match>
      <Match when={local.kind === 'user'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-user" />
        </span>
      </Match>
      <Match when={local.kind === 'user-check'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-user-check" />
        </span>
      </Match>
      <Match when={local.kind === 'x'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-x" />
        </span>
      </Match>
      <Match when={local.kind === 'layout-sidebar-right'}>
        <span {...rest} class={`flex items-center ${rest.class ?? ''}`}>
          <i class="i-tabler-layout-sidebar-right" />
        </span>
      </Match>
    </Switch>
  );
}
