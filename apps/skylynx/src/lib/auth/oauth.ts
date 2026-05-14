import {
  OAuthUserAgent,
  configureOAuth,
  createAuthorizationUrl,
  deleteStoredSession,
  finalizeAuthorization,
  getSession,
  listStoredSessions,
  type Session
} from '@atcute/oauth-browser-client';
import {
  CompositeDidDocumentResolver,
  LocalActorResolver,
  PlcDidDocumentResolver,
  WebDidDocumentResolver,
  XrpcHandleResolver
} from '@atcute/identity-resolver';

import { fetchActorProfile } from '../api/bluesky';
import type { ActorIdentifier } from '@atcute/lexicons';
import type { ActorProfile, Did } from '../types';

const BSKY_PUBLIC_API = 'https://public.api.bsky.app';
const LAST_DID_KEY = 'skylynx:last-authenticated-did';
const OAUTH_SCOPE = import.meta.env.VITE_OAUTH_SCOPE || 'atproto transition:generic';

let configured = false;

export type AuthenticatedAccount = {
  agent: OAuthUserAgent;
  did: Did;
  handle: string;
  profile?: ActorProfile;
  session: Session;
};

export const configureSkylynxOAuth = () => {
  if (configured) return;

  configureOAuth({
    metadata: { client_id: oauthClientId(), redirect_uri: oauthRedirectUri() },
    identityResolver: new LocalActorResolver({
      didDocumentResolver: new CompositeDidDocumentResolver({
        methods: { plc: new PlcDidDocumentResolver(), web: new WebDidDocumentResolver() }
      }),
      handleResolver: new XrpcHandleResolver({ serviceUrl: BSKY_PUBLIC_API })
    }),
    storageName: 'skylynx-oauth'
  });

  configured = true;
};

export const beginSignIn = async (identifier: string) => {
  configureSkylynxOAuth();
  const target = identifier.trim();
  if (!target) throw new Error('Enter a handle or DID to sign in');

  const authUrl = await createAuthorizationUrl({
    scope: OAUTH_SCOPE,
    target: { identifier: target as ActorIdentifier, type: 'account' }
  });

  await delay(200);
  window.location.assign(authUrl);
};

export const finalizeSignInFromLocation = async () => {
  configureSkylynxOAuth();
  const params = new URLSearchParams(window.location.hash.slice(1));
  if (!params.has('code') && !params.has('error')) return;

  const { session } = await finalizeAuthorization(params);
  rememberDid(session.info.sub);
  window.history.replaceState(null, '', '/app');
};

export const resumeLastAccount = async (): Promise<AuthenticatedAccount | undefined> => {
  configureSkylynxOAuth();
  const did = rememberedDid() ?? listStoredSessions()[0];
  if (!did) return;

  try {
    return accountFromSession(await getSession(did, { allowStale: true }));
  } catch {
    forgetDid();
  }
};

export const signOutAccount = async (did: Did) => {
  configureSkylynxOAuth();

  try {
    const session = await getSession(did, { allowStale: true });
    await new OAuthUserAgent(session).signOut();
  } catch {
    deleteStoredSession(did);
  } finally {
    forgetDid();
  }
};

const accountFromSession = async (session: Session): Promise<AuthenticatedAccount> => {
  const agent = new OAuthUserAgent(session);
  const profile = await fetchActorProfile(session.info.sub);
  rememberDid(session.info.sub);

  return { agent, did: session.info.sub, handle: profile.handle, profile, session };
};

const rememberedDid = () => localStorage.getItem(LAST_DID_KEY) as Did | null;
const rememberDid = (did: Did) => localStorage.setItem(LAST_DID_KEY, did);
const forgetDid = () => localStorage.removeItem(LAST_DID_KEY);
const delay = (milliseconds: number) => new Promise((resolve) => setTimeout(resolve, milliseconds));

const oauthRedirectUri = () => {
  return import.meta.env.VITE_OAUTH_REDIRECT_URI || `${window.location.origin}/oauth/callback`;
};

const oauthClientId = () => {
  const configured = import.meta.env.VITE_OAUTH_CLIENT_ID;
  if (configured) return configured;

  return (
    `http://localhost?redirect_uri=${encodeURIComponent(oauthRedirectUri())}` +
    `&scope=${encodeURIComponent(OAUTH_SCOPE)}`
  );
};
