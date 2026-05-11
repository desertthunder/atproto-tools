import { Client, simpleFetchHandler } from '@atcute/client';

import type {} from '@atcute/bluesky';
import type {} from '@atcute/microcosm';

export const BSKY_PUBLIC_API = 'https://public.api.bsky.app';
export const CONSTELLATION_API = 'https://constellation.microcosm.blue';

export type ApiClientOptions = { fetch?: typeof globalThis.fetch; service?: string | URL };

export const createBlueskyClient = (options: ApiClientOptions = {}) => {
  return new Client({
    handler: simpleFetchHandler({ service: options.service ?? BSKY_PUBLIC_API, fetch: options.fetch })
  });
};

export const createConstellationClient = (options: ApiClientOptions = {}) => {
  return new Client({
    handler: simpleFetchHandler({ service: options.service ?? CONSTELLATION_API, fetch: options.fetch })
  });
};
