import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';

interface AuthState {
  account: InjectedAccountWithMeta | null;
  isAuthenticated: boolean;
  token: string | null;
  setAccount: (account: InjectedAccountWithMeta | null) => void;
  setToken: (token: string | null) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>()(
    persist(
        (set) => ({
            account: null,
            isAuthenticated: false,
            token: null,

            setAccount: (account) =>
                set({ account, isAuthenticated: !!account }),

            setToken: (token) =>
                set({ token }),

            logout: () =>
                set({ account: null, isAuthenticated: false, token: null }),
            }),
            {
            name: 'auth-storage',
            partialize: (state) => ({
                token: state.token,
            }),
        }
    )
);