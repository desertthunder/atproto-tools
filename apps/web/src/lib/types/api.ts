export type Did = `did:${string}:${string}`;
export type Handle = `${string}.${string}`;
export type ActorIdentifier = Did | Handle;

export type GraphFetchOptions = {
  actor: ActorIdentifier;
  cursor?: string;
  fetch?: typeof globalThis.fetch;
  limit?: number;
};

export type GraphPage<T> = { cursor?: string; items: T[] };

export type ProfileView = {
  avatar?: string;
  description?: string;
  did: Did;
  displayName?: string;
  handle: Handle;
  indexedAt?: string;
};

export type MutualsOptions = { actorDid: Did; fetch?: typeof globalThis.fetch; followingDids: Did[]; limit?: number };
