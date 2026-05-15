import { Show, type ParentProps } from 'solid-js';
import { Icon } from './Icon';

export function AlertMessage(props: ParentProps & { class?: string; error?: string }) {
  const message = () => props.error;

  return (
    <div
      class={`flex items-start gap-2.5 px-5 py-3 border-b border-border-subtle text-danger text-[12px] ${props.class ?? ''}`}>
      <Show when={message()} fallback={<>{props.children}</>}>
        {(text) => (
          <>
            <Icon kind="warning" class="mt-0.5" />
            <span>{text()}</span>
          </>
        )}
      </Show>
    </div>
  );
}
