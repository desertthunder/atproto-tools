import { type ParentProps } from 'solid-js';

export function SectionLabel(props: ParentProps) {
  return <h2 class="text-[13px] font-semibold tracking-[0.08em] uppercase text-ink-muted">{props.children}</h2>;
}
