import { create } from 'zustand';
import { substrateQueries } from '../services/substrate/queries';
import { substrateCalls } from '../services/substrate/calls';
import { substrateAPI } from '../services/substrate/api';
import { useIdentityStore } from './identityStore';
import type { Credential, CredentialType, CredentialStatus } from '../types/substrate';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { STORAGE_KEYS, TIME, ERROR_MESSAGES } from '../utils/constants';

interface CredentialState {
    credentials: Credential[];
    isLoading: boolean;
    error: string | null;
    lastSync: number | null;
    
    // Actions
    fetchCredentials: () => Promise<void>;
    getCredentialById: (id: string) => Credential | null;
    refreshCredential: (id: string) => Promise<void>;
    verifyCredential: (credentialId: string) => Promise<boolean>;
    revokeCredential: (credentialId: string) => Promise<void>;
    filterByType: (type: CredentialType) => Credential[];
    filterByStatus: (status: CredentialStatus) => Credential[];
    getActiveCredentials: () => Credential[];
    getExpiredCredentials: () => Credential[];
    getRevoked Credentials: () => Credential[];
    clearCache: () => Promise<void>;
    setError: (error: string | null) => void;
}

export const useCredentialStore = create<CredentialState>((set, get) => ({
    credentials: [],
    isLoading: false,
    error: null,
    lastSync: null,

    fetchCredentials: async () => {
        set({ isLoading: true, error: null });
        
        try {
            if (!substrateAPI.isConnected()) {
                throw new Error(ERROR_MESSAGES.API_CONNECTION_FAILED);
            }

            const identityStore = useIdentityStore.getState();
            const { didHash, address } = identityStore;

            if (!didHash && !address) {
                throw new Error(ERROR_MESSAGES.IDENTITY_NOT_FOUND);
            }

            let credentials: Credential[] = [];

            if (didHash) {
                const credentialIds = await substrateQueries.getCredentialsForSubject(didHash);
                
                if (credentialIds.length > 0) {
                    credentials = await substrateQueries.batchGetCredentials(credentialIds);
                }
            } else if (address) {
                credentials = await substrateQueries.getCredentialsForAccount(address);
            }

            const now = Date.now();
            
            await AsyncStorage.setItem(
                STORAGE_KEYS.CACHED_CREDENTIALS,
                JSON.stringify(credentials)
            );
            
            await AsyncStorage.setItem(
                STORAGE_KEYS.LAST_SYNC,
                now.toString()
            );

            set({
                credentials,
                lastSync: now,
                isLoading: false,
                error: null
            });

            console.log(`Fetched ${credentials.length} credentials`);
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            // Try to load from cache on error
            try {
                const cached = await AsyncStorage.getItem(STORAGE_KEYS.CACHED_CREDENTIALS);
                if (cached) {
                    const credentials = JSON.parse(cached);
                    set({ 
                        credentials, 
                        error: errorMessage, 
                        isLoading: false 
                    });
                    console.log('Loaded credentials from cache');
                    return;
                }
            } catch (cacheError) {
                console.error('Failed to load from cache:', cacheError);
            }

            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Fetch credentials failed:', error);
            throw error;
        }
    },

    getCredentialById: (id: string): Credential | null => {
        const { credentials } = get();
        return credentials.find(cred => 
            JSON.stringify(cred) === id || 
            cred.subject === id
        ) || null;
    },

    refreshCredential: async (id: string) => {
        set({ isLoading: true, error: null });
        
        try {
            if (!substrateAPI.isConnected()) {
                throw new Error(ERROR_MESSAGES.API_CONNECTION_FAILED);
            }

            const result = await substrateQueries.getCredential(id);

            if (!result.exists || !result.credential) {
                throw new Error(ERROR_MESSAGES.CREDENTIAL_NOT_FOUND);
            }

            const { credentials } = get();
            const updatedCredentials = credentials.map(cred => 
                JSON.stringify(cred) === id ? result.credential! : cred
            );

            await AsyncStorage.setItem(
                STORAGE_KEYS.CACHED_CREDENTIALS,
                JSON.stringify(updatedCredentials)
            );

            set({
                credentials: updatedCredentials,
                isLoading: false,
                error: null
            });

            console.log('Credential refreshed:', id);
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Refresh credential failed:', error);
            throw error;
        }
    },

    verifyCredential: async (credentialId: string): Promise<boolean> => {
        set({ isLoading: true, error: null });
        
        try {
            if (!substrateAPI.isConnected()) {
                throw new Error(ERROR_MESSAGES.API_CONNECTION_FAILED);
            }

            const identityStore = useIdentityStore.getState();
            const { keyPair } = identityStore;

            if (!keyPair) {
                throw new Error(ERROR_MESSAGES.KEYPAIR_NOT_FOUND);
            }

            const result = await substrateCalls.verifyCredential(keyPair, credentialId);

            set({ isLoading: false, error: null });

            if (result.success) {
                console.log('Credential verified successfully:', credentialId);
                return true;
            } else {
                console.log('Credential verification failed:', result.error);
                return false;
            }
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Verify credential error:', error);
            return false;
        }
    },

    revokeCredential: async (credentialId: string) => {
        set({ isLoading: true, error: null });
        
        try {
            if (!substrateAPI.isConnected()) {
                throw new Error(ERROR_MESSAGES.API_CONNECTION_FAILED);
            }

            const identityStore = useIdentityStore.getState();
            const { keyPair } = identityStore;

            if (!keyPair) {
                throw new Error(ERROR_MESSAGES.KEYPAIR_NOT_FOUND);
            }

            const result = await substrateCalls.revokeCredential(keyPair, credentialId);

            if (!result.success) {
                throw new Error(result.error || 'Failed to revoke credential');
            }

            await get().refreshCredential(credentialId);

            set({ isLoading: false, error: null });

            console.log('Credential revoked successfully:', credentialId);
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Revoke credential failed:', error);
            throw error;
        }
    },

    filterByType: (type: CredentialType): Credential[] => {
        const { credentials } = get();
        return credentials.filter(cred => cred.credentialType === type);
    },

    filterByStatus: (status: CredentialStatus): Credential[] => {
        const { credentials } = get();
        return credentials.filter(cred => cred.status === status);
    },

    getActiveCredentials: (): Credential[] => {
        return get().filterByStatus('Active' as CredentialStatus);
    },

    getExpiredCredentials: (): Credential[] => {
        return get().filterByStatus('Expired' as CredentialStatus);
    },

    getRevokedCredentials: (): Credential[] => {
        return get().filterByStatus('Revoked' as CredentialStatus);
    },

    clearCache: async () => {
        try {
            await AsyncStorage.removeItem(STORAGE_KEYS.CACHED_CREDENTIALS);
            await AsyncStorage.removeItem(STORAGE_KEYS.LAST_SYNC);
            
            set({
                credentials: [],
                lastSync: null,
                error: null
            });
            
            console.log('Credential cache cleared');
        } catch (error) {
            console.error('Clear cache failed:', error);
            throw error;
        }
    },

    setError: (error: string | null) => {
        set({ error });
    }
}));