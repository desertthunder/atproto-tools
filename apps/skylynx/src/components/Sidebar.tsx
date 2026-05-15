import { Match, Switch, type JSX } from 'solid-js';
import { Icon, type IconKind } from './Icon';

export type SidebarTab = 'digest' | 'network' | 'history';

type SidebarPanelRenderer = () => JSX.Element;

export function Sidebar(props: {
  active: SidebarTab;
  digest: SidebarPanelRenderer;
  history: SidebarPanelRenderer;
  network: SidebarPanelRenderer;
  onChange: (tab: SidebarTab) => void;
}) {
  return (
    <>
      <SidebarTitle />
      <aside class="app-sidebar">
        <SidebarTabs active={props.active} onChange={props.onChange} />
        <div class="min-h-0 flex-1 overflow-hidden">
          <Switch>
            <Match when={props.active === 'network'}>{props.network()}</Match>
            <Match when={props.active === 'history'}>{props.history()}</Match>
            <Match when={props.active === 'digest'}>{props.digest()}</Match>
          </Switch>
        </div>
      </aside>
    </>
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
