import type { EntryGenerator, RequestHandler } from './$types';

const relationshipOptions = {
  follower: { ringColor: '1d4ed8', seed: 'FOLLOWER' },
  following: { ringColor: 'f43f5e', seed: 'FOLLOWING' },
  mutuals: { ringColor: '10b981', seed: 'MUTUAL' },
  origin: { ringColor: '0ea5e9', seed: 'ORIGIN' }
} as const;

export const prerender = true;

export const entries: EntryGenerator = () => {
  return Object.keys(relationshipOptions).map((relationship) => ({ relationship }));
};

export const GET: RequestHandler = async ({ fetch, params }) => {
  const options = relationshipOptions[params.relationship as keyof typeof relationshipOptions];

  if (!options) return new Response('Not found', { status: 404 });

  const url = new URL('https://api.dicebear.com/9.x/rings/svg');
  url.searchParams.set('seed', options.seed);
  url.searchParams.set('ringColor', options.ringColor);
  url.searchParams.set('backgroundColor', '020617');
  url.searchParams.set('radius', '18');
  url.searchParams.set('size', '96');

  const response = await fetch(url);

  if (!response.ok) {
    return new Response('Unable to prerender DiceBear avatar', { status: response.status });
  }

  return new Response(await response.text(), {
    headers: { 'cache-control': 'public, immutable, max-age=31536000', 'content-type': 'image/svg+xml' }
  });
};
