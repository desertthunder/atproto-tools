import { Show, type ParentProps } from 'solid-js';
import { Icon, type IconKind } from './Icon';

export function CenteredState(props: ParentProps & { class?: string; icon?: IconKind; iconClass?: string }) {
  return (
    <div class={`flex flex-1 flex-col items-center justify-center gap-2 px-5 py-10 text-center ${props.class ?? ''}`}>
      <Show when={props.icon}>
        {(icon) => <Icon kind={icon()} class={`text-ink-faint text-[32px] ${props.iconClass ?? ''}`} />}
      </Show>
      {props.children}
    </div>
  );
}
