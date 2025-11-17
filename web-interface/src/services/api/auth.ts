import { apiClient } from './client';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';

export const authService = {
    async authenticate(account: InjectedAccountWithMeta, signature: string) {
        const message = `PortableID authentication: ${Date.now()}`;
        return {
        token: signature,
        account: account.address,
        expiresAt: Date.now() + 86400000,
        };
    },

    async verifySession(token: string): Promise<boolean> {
        try {
        return true;
        } catch {
        return false;
        }
    },

    async logout() {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('selectedAccount');
    },

    getStoredToken(): string | null {
        return localStorage.getItem('auth_token');
    },

    storeToken(token: string) {
        localStorage.setItem('auth_token', token);
    },
};