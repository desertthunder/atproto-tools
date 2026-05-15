import { Show, type JSX, type ParentProps } from 'solid-js';
import { IconHeading } from './IconHeading';
import { type IconKind } from './Icon';

type PanelHeaderProps = ParentProps<{
  class?: string;
  icon: IconKind;
  iconClass?: string;
  subtitle?: JSX.Element;
  title: JSX.Element;
  titleClass?: string;
}>;

export function PanelHeader(props: PanelHeaderProps) {
  return (
    <div class={`flex items-center justify-between gap-3 px-5 py-4 border-b border-border ${props.class ?? ''}`}>
      <div class="min-w-0">
        <IconHeading icon={props.icon} iconClass={props.iconClass} class={props.titleClass}>
          {props.title}
        </IconHeading>
        <Show when={props.subtitle}>
          {(subtitle) => <span class="block truncate text-[12px] text-ink-muted">{subtitle()}</span>}
        </Show>
      </div>
      <Show when={props.children}>
        <div class="flex items-center gap-2">{props.children}</div>
      </Show>
    </div>
  );
}
