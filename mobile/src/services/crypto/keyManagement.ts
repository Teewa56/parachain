import { Keyring } from '@polkadot/keyring';
import { mnemonicGenerate, mnemonicValidate, mnemonicToMiniSecret } from '@polkadot/util-crypto';
import { u8aToHex, hexToU8a } from '@polkadot/util';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { KeyPairInfo } from '../../types/substrate';
import * as SecureStore from 'expo-secure-store';


const STORAGE_KEYS = {
    MNEMONIC: 'identity_wallet_mnemonic',
    KEYPAIR_JSON: 'identity_wallet_keypair',
    PUBLIC_KEY: 'identity_wallet_public_key',
    ADDRESS: 'identity_wallet_address',
};

class KeyManagementService {
    private keyring: Keyring;
    private cachedKeyPair: KeyringPair | null = null;

    constructor() {
        // Initialize keyring with sr25519 (Substrate default)
        this.keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
    }

    /**
     * Generate a new seed phrase (12 words)
     */
    generateSeedPhrase(): string {
        return mnemonicGenerate(12);
    }

    /**
     * Generate a new seed phrase (24 words)
     */
    generateSeedPhrase24(): string {
        return mnemonicGenerate(24);
    }

    /**
     * Validate a seed phrase
     */
    validateSeedPhrase(mnemonic: string): boolean {
        return mnemonicValidate(mnemonic);
    }

    /**
     * Create keypair from seed phrase
     */
    async createFromSeed(mnemonic: string, name?: string): Promise<KeyPairInfo> {
        if (!this.validateSeedPhrase(mnemonic)) {
        throw new Error('Invalid mnemonic phrase');
        }

        try {
        // Create keypair from mnemonic
        const keyPair = this.keyring.addFromMnemonic(mnemonic, { name }, 'sr25519');

        const keyPairInfo: KeyPairInfo = {
            pair: keyPair,
            mnemonic,
            address: keyPair.address,
            publicKey: u8aToHex(keyPair.publicKey),
        };

        // Store securely
        await this.storeKeyPair(keyPairInfo);

        // Cache keypair
        this.cachedKeyPair = keyPair;

        return keyPairInfo;
        } catch (error) {
        console.error('Error creating keypair from seed:', error);
        throw error;
        }
    }

    /**
     * Generate new keypair with mnemonic
     */
    async generateKeyPair(name?: string): Promise<KeyPairInfo> {
        const mnemonic = this.generateSeedPhrase();
        return await this.createFromSeed(mnemonic, name);
    }

    /**
     * Import keypair from existing mnemonic
     */
    async importFromMnemonic(mnemonic: string, name?: string): Promise<KeyPairInfo> {
        return await this.createFromSeed(mnemonic, name);
    }

    /**
     * Get stored keypair
     */
    async getKeyPair(): Promise<KeyringPair | null> {
        // Return cached if available
        if (this.cachedKeyPair) {
        return this.cachedKeyPair;
        }

        try {
        // Retrieve mnemonic from secure storage
        const mnemonic = await SecureStore.getItemAsync(STORAGE_KEYS.MNEMONIC);
        
        if (!mnemonic) {
            return null;
        }

        // Recreate keypair from mnemonic
        const keyPair = this.keyring.addFromMnemonic(mnemonic, {}, 'sr25519');
        this.cachedKeyPair = keyPair;

        return keyPair;
        } catch (error) {
        console.error('Error retrieving keypair:', error);
        return null;
        }
    }

    /**
     * Get address
     */
    async getAddress(): Promise<string | null> {
        try {
        return await SecureStore.getItemAsync(STORAGE_KEYS.ADDRESS);
        } catch (error) {
        console.error('Error getting address:', error);
        return null;
        }
    }

    /**
     * Get public key
     */
    async getPublicKey(): Promise<string | null> {
        try {
        return await SecureStore.getItemAsync(STORAGE_KEYS.PUBLIC_KEY);
        } catch (error) {
        console.error('Error getting public key:', error);
        return null;
        }
    }

    /**
     * Get mnemonic (requires authentication)
     */
    async getMnemonic(): Promise<string | null> {
        try {
        return await SecureStore.getItemAsync(STORAGE_KEYS.MNEMONIC, {
            requireAuthentication: true,
        });
        } catch (error) {
        console.error('Error getting mnemonic:', error);
        return null;
        }
    }

    /**
     * Check if keypair exists
     */
    async hasKeyPair(): Promise<boolean> {
        try {
        const mnemonic = await SecureStore.getItemAsync(STORAGE_KEYS.MNEMONIC);
        return !!mnemonic;
        } catch (error) {
        console.error('Error checking keypair:', error);
        return false;
        }
    }

    /**
     * Store keypair securely
     */
    private async storeKeyPair(keyPairInfo: KeyPairInfo): Promise<void> {
        try {
        // Store mnemonic with authentication requirement
        await SecureStore.setItemAsync(
            STORAGE_KEYS.MNEMONIC,
            keyPairInfo.mnemonic,
            {
            requireAuthentication: true,
            }
        );

        // Store address and public key (non-sensitive, faster access)
        await SecureStore.setItemAsync(STORAGE_KEYS.ADDRESS, keyPairInfo.address);
        await SecureStore.setItemAsync(STORAGE_KEYS.PUBLIC_KEY, keyPairInfo.publicKey);

        console.log('✅ Keypair stored securely');
        } catch (error) {
        console.error('Error storing keypair:', error);
        throw error;
        }
    }

    /**
     * Delete keypair
     */
    async deleteKeyPair(): Promise<void> {
        try {
        await SecureStore.deleteItemAsync(STORAGE_KEYS.MNEMONIC);
        await SecureStore.deleteItemAsync(STORAGE_KEYS.ADDRESS);
        await SecureStore.deleteItemAsync(STORAGE_KEYS.PUBLIC_KEY);
        
        this.cachedKeyPair = null;

        console.log('✅ Keypair deleted');
        } catch (error) {
        console.error('Error deleting keypair:', error);
        throw error;
        }
    }

    /**
     * Export keypair as JSON
     */
    async exportKeyPairJson(password: string): Promise<string> {
        const keyPair = await this.getKeyPair();
        if (!keyPair) {
        throw new Error('No keypair found');
        }

        try {
        const json = keyPair.toJson(password);
        return JSON.stringify(json);
        } catch (error) {
        console.error('Error exporting keypair:', error);
        throw error;
        }
    }

    /**
     * Import keypair from JSON
     */
    async importFromJson(json: string, password: string): Promise<KeyPairInfo> {
        try {
        const keyringJson = JSON.parse(json);
        const keyPair = this.keyring.addFromJson(keyringJson);
        
        // Unlock with password
        keyPair.unlock(password);

        // Note: JSON export doesn't include mnemonic
        // Store warning that backup won't include seed phrase
        const keyPairInfo: KeyPairInfo = {
            pair: keyPair,
            mnemonic: '', // JSON import doesn't have mnemonic
            address: keyPair.address,
            publicKey: u8aToHex(keyPair.publicKey),
        };

        // Store (without mnemonic)
        await SecureStore.setItemAsync(STORAGE_KEYS.ADDRESS, keyPairInfo.address);
        await SecureStore.setItemAsync(STORAGE_KEYS.PUBLIC_KEY, keyPairInfo.publicKey);

        this.cachedKeyPair = keyPair;

        return keyPairInfo;
        } catch (error) {
        console.error('Error importing from JSON:', error);
        throw error;
        }
    }

    /**
     * Clear cached keypair (e.g., on logout)
     */
    clearCache(): void {
        this.cachedKeyPair = null;
    }

    /**
     * Create additional keypair (for multi-identity support)
     */
    async createAdditionalKeyPair(name: string): Promise<KeyPairInfo> {
        const mnemonic = this.generateSeedPhrase();
        const keyPair = this.keyring.addFromMnemonic(mnemonic, { name }, 'sr25519');

        return {
        pair: keyPair,
        mnemonic,
        address: keyPair.address,
        publicKey: u8aToHex(keyPair.publicKey),
        };
    }

    /**
     * Derive child keypair from parent (HD wallet)
     */
    derivePath(keyPair: KeyringPair, derivationPath: string): KeyringPair {
        return keyPair.derive(derivationPath);
    }
}

export const keyManagement = new KeyManagementService();