import { type JSX, splitProps } from 'solid-js';
import { Icon, type IconKind } from './Icon';

type IconHeadingProps = { children: JSX.Element; class?: string; icon: IconKind; iconClass?: string };

export function IconHeading(props: IconHeadingProps) {
  const [local, rest] = splitProps(props, ['children', 'class', 'icon', 'iconClass']);

  return (
    <div {...rest} class={`flex items-center gap-2 text-[13px] font-semibold text-ink ${local.class ?? ''}`}>
      <Icon kind={local.icon} class={`text-accent ${local.iconClass ?? ''}`} />
      <span>{local.children}</span>
    </div>
  );
}
