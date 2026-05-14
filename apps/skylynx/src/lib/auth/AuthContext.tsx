import { createContext, createSignal, onMount, useContext, type ParentProps } from 'solid-js';

import {
  beginSignIn,
  finalizeSignInFromLocation,
  resumeLastAccount,
  signOutAccount,
  type AuthenticatedAccount
} from './oauth';

type AuthContextValue = {
  account: () => AuthenticatedAccount | undefined;
  error: () => string;
  isLoading: () => boolean;
  signIn: (identifier: string) => Promise<void>;
  signOut: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue>();

export function AuthProvider(props: ParentProps) {
  const [account, setAccount] = createSignal<AuthenticatedAccount>();
  const [error, setError] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(true);

  const loadAccount = async () => {
    setError('');
    setIsLoading(true);

    try {
      await finalizeSignInFromLocation();
      setAccount(await resumeLastAccount());
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Unable to finish sign in');
    } finally {
      setIsLoading(false);
    }
  };

  const signIn = async (identifier: string) => {
    setError('');
    await beginSignIn(identifier);
  };

  const signOut = async () => {
    const current = account();
    if (!current) return;

    setError('');
    setIsLoading(true);

    try {
      await signOutAccount(current.did);
      setAccount();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Unable to sign out');
    } finally {
      setIsLoading(false);
    }
  };

  onMount(() => {
    void loadAccount();
  });

  return (
    <AuthContext.Provider value={{ account, error, isLoading, signIn, signOut }}>{props.children}</AuthContext.Provider>
  );
}

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) throw new Error('AuthProvider is missing');
  return context;
};
