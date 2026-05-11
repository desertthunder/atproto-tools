import type { SocialGraphFilter } from '$lib/types/social-graph';

const filterSegments: Record<SocialGraphFilter, string> = {
  all: '',
  followers: 'follows',
  following: 'following',
  mutuals: 'mutuals'
};

export const normalizeGraphHandle = (handle: string) => handle.trim().replace(/^@/, '');

export const socialGraphPath = (handle: string, filter: SocialGraphFilter) => {
  const normalized = normalizeGraphHandle(handle);
  if (!normalized) return '/';

  const segment = filterSegments[filter];
  const base = `/${encodeURIComponent(normalized)}`;

  return segment ? `${base}/${segment}` : base;
};
