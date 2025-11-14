import { create } from 'zustand';
import { keyManagement } from '../services/crypto/keyManagement';
import { substrateCalls } from '../services/substrate/calls';
import { substrateQueries } from '../services/substrate/queries';
import { substrateAPI } from '../services/substrate/api';
import type { Identity, DidDocument } from '../types/substrate';
import type { KeyringPair } from '@polkadot/keyring/types';
import { hashDid, generateDid } from '../services/substrate/utils';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { STORAGE_KEYS } from '../utils/constants';

interface IdentityState {
    // State
    did: string | null;
    didHash: string | null;
    address: string | null;
    publicKey: string | null;
    identity: Identity | null;
    didDocument: DidDocument | null;
    isLoading: boolean;
    error: string | null;
    keyPair: KeyringPair | null;

    // Actions
    createIdentity: (mnemonic: string, name?: string) => Promise<void>;
    importIdentity: (mnemonic: string, name?: string) => Promise<void>;
    loadIdentity: () => Promise<void>;
    updateIdentity: (newPublicKey: string) => Promise<void>;
    deactivateIdentity: () => Promise<void>;
    reactivateIdentity: () => Promise<void>;
    refreshIdentity: () => Promise<void>;
    clearIdentity: () => Promise<void>;
    setError: (error: string | null) => void;
}

export const useIdentityStore = create<IdentityState>((set, get) => ({
    // Initial state
    did: null,
    didHash: null,
    address: null,
    publicKey: null,
    identity: null,
    didDocument: null,
    isLoading: false,
    error: null,
    keyPair: null,

    // Create new identity
    createIdentity: async (mnemonic: string, name?: string) => {
        set({ isLoading: true, error: null });

        try {
            // Check if API is connected
            if (!substrateAPI.isConnected()) {
                throw new Error('Not connected to blockchain');
            }

            // Create keypair from mnemonic
            const keyPairInfo = await keyManagement.createFromSeed(mnemonic, name);

            // Generate DID
            const did = generateDid(keyPairInfo.address);
            const didHashValue = hashDid(did);

            // Submit transaction to create identity on-chain
            const result = await substrateCalls.createIdentity(
                keyPairInfo.pair,
                did,
                keyPairInfo.publicKey
            );

            if (!result.success) {
                throw new Error(result.error || 'Failed to create identity');
            }

            // Store DID and address
            await AsyncStorage.setItem(STORAGE_KEYS.CURRENT_DID, did);
            await AsyncStorage.setItem(STORAGE_KEYS.DID_HASH, didHashValue);

            // Load identity from chain
            const identityResult = await substrateQueries.getIdentity(didHashValue);

            set({
                did,
                didHash: didHashValue,
                address: keyPairInfo.address,
                publicKey: keyPairInfo.publicKey,
                identity: identityResult.identity || null,
                didDocument: identityResult.didDocument || null,
                keyPair: keyPairInfo.pair,
                isLoading: false,
                error: null,
            });

            console.log(' Identity created successfully');
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            set({ error: errorMessage, isLoading: false });
            console.error(' Failed to create identity:', error);
            throw error;
        }
    },

    // Import existing identity
    importIdentity: async (mnemonic: string, name?: string) => {
        set({ isLoading: true, error: null });

        try {
            // Check if API is connected
            if (!substrateAPI.isConnected()) {
                throw new Error('Not connected to blockchain');
            }

            // Import keypair
            const keyPairInfo = await keyManagement.importFromMnemonic(mnemonic, name);

            // Get DID hash from account
            const didHashValue = await substrateQueries.getAccountDid(keyPairInfo.address);

            if (!didHashValue) {
                throw new Error('No identity found for this account on-chain');
            }

            // Load identity from chain
            const identityResult = await substrateQueries.getIdentity(didHashValue);

            if (!identityResult.exists || !identityResult.identity) {
                throw new Error('Identity not found on-chain');
            }

            // Get DID from document
            const did = identityResult.didDocument?.did || generateDid(keyPairInfo.address);

            // Store DID and address
            await AsyncStorage.setItem(STORAGE_KEYS.CURRENT_DID, did);
            await AsyncStorage.setItem(STORAGE_KEYS.DID_HASH, didHashValue);

            set({
                did,
                didHash: didHashValue,
                address: keyPairInfo.address,
                publicKey: keyPairInfo.publicKey,
                identity: identityResult.identity,
                didDocument: identityResult.didDocument || null,
                keyPair: keyPairInfo.pair,
                isLoading: false,
                error: null,
            });

            console.log(' Identity imported successfully');
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            set({ error: errorMessage, isLoading: false });
            console.error(' Failed to import identity:', error);
            throw error;
        }
    },

    // Load existing identity from storage
    loadIdentity: async () => {
        set({ isLoading: true, error: null });

        try {
            // Get stored DID
            const storedDid = await AsyncStorage.getItem(STORAGE_KEYS.CURRENT_DID);
            const storedDidHash = await AsyncStorage.getItem(STORAGE_KEYS.DID_HASH);

            if (!storedDid || !storedDidHash) {
                set({ isLoading: false });
                return;
            }

            // Get keypair
            const keyPair = await keyManagement.getKeyPair();
            if (!keyPair) {
                throw new Error('Keypair not found');
            }

            // Get address and public key
            const address = await keyManagement.getAddress();
            const publicKey = await keyManagement.getPublicKey();

            // Load identity from chain if connected
            let identity: Identity | null = null;
            let didDocument: DidDocument | null = null;

            if (substrateAPI.isConnected()) {
                const identityResult = await substrateQueries.getIdentity(storedDidHash);
                identity = identityResult.identity || null;
                didDocument = identityResult.didDocument || null;
            }

            set({
                did: storedDid,
                didHash: storedDidHash,
                address: address || keyPair.address,
                publicKey: publicKey || null,
                identity,
                didDocument,
                keyPair,
                isLoading: false,
                error: null,
            });

            console.log(' Identity loaded from storage');
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            set({ error: errorMessage, isLoading: false });
            console.error(' Failed to load identity:', error);
        }
    },

    // Update identity public key
    updateIdentity: async (newPublicKey: string) => {
        const { keyPair } = get();
        
        if (!keyPair) {
            throw new Error('No keypair available');
        }

        set({ isLoading: true, error: null });

        try {
            const result = await substrateCalls.updateIdentity(keyPair, newPublicKey);

            if (!result.success) {
                throw new Error(result.error || 'Failed to update identity');
            }

            // Refresh identity
            await get().refreshIdentity();

            console.log(' Identity updated successfully');
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            set({ error: errorMessage, isLoading: false });
            console.error(' Failed to update identity:', error);
            throw error;
        }
    },

    // Deactivate identity
    deactivateIdentity: async () => {
        const { keyPair } = get();
        
        if (!keyPair) {
            throw new Error('No keypair available');
        }

        set({ isLoading: true, error: null });

        try {
            const result = await substrateCalls.deactivateIdentity(keyPair);

            if (!result.success) {
                throw new Error(result.error || 'Failed to deactivate identity');
            }

            // Refresh identity
            await get().refreshIdentity();

            console.log(' Identity deactivated successfully');
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            set({ error: errorMessage, isLoading: false });
            console.error(' Failed to deactivate identity:', error);
            throw error;
        }
    },

    // Reactivate identity
    reactivateIdentity: async () => {
        const { keyPair } = get();
        
        if (!keyPair) {
            throw new Error('No keypair available');
        }

        set({ isLoading: true, error: null });

        try {
            const result = await substrateCalls.reactivateIdentity(keyPair);

            if (!result.success) {
                throw new Error(result.error || 'Failed to reactivate identity');
            }

            // Refresh identity
            await get().refreshIdentity();

            console.log(' Identity reactivated successfully');
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            set({ error: errorMessage, isLoading: false });
            console.error(' Failed to reactivate identity:', error);
            throw error;
        }
    },

    // Refresh identity from chain
    refreshIdentity: async () => {
        const { didHash } = get();

        if (!didHash) {
            return;
        }

        set({ isLoading: true, error: null });

        try {
            if (!substrateAPI.isConnected()) {
                throw new Error('Not connected to blockchain');
            }

            const identityResult = await substrateQueries.getIdentity(didHash);

            set({
                identity: identityResult.identity || null,
                didDocument: identityResult.didDocument || null,
                isLoading: false,
                error: null,
            });

            console.log(' Identity refreshed');
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            set({ error: errorMessage, isLoading: false });
            console.error(' Failed to refresh identity:', error);
        }
    },

    // Clear identity (logout)
    clearIdentity: async () => {
        try {
            await AsyncStorage.removeItem(STORAGE_KEYS.CURRENT_DID);
            await AsyncStorage.removeItem(STORAGE_KEYS.DID_HASH);
            
            keyManagement.clearCache();

            set({
                did: null,
                didHash: null,
                address: null,
                publicKey: null,
                identity: null,
                didDocument: null,
                keyPair: null,
                error: null,
                isLoading: false,
            });

            console.log('Identity cleared');
        } catch (error) {
            console.error(' Failed to clear identity:', error);
        }
    },

    // Set error
    setError: (error: string | null) => {
        set({ error });
    },
}));