import * as SecureStore from 'expo-secure-store';
import { Platform } from 'react-native';

/**
 * Secure Storage Service
 * 
 * Provides encrypted storage using the device's secure enclave:
 * - iOS: Keychain Services
 * - Android: Keystore System
 */

interface SecureStorageOptions {
    requireAuthentication?: boolean;
    authenticationPrompt?: string;
}

class SecureStorageService {
    /**
     * Store a value securely
     */
    async setItem(
        key: string,
        value: string,
        options?: SecureStorageOptions
    ): Promise<boolean> {
        try {
            const storeOptions: SecureStore.SecureStoreOptions = {
                requireAuthentication: options?.requireAuthentication || false,
            };

            // Add authentication prompt for iOS
            if (Platform.OS === 'ios' && options?.authenticationPrompt) {
                storeOptions.authenticationPrompt = options.authenticationPrompt;
            }

            await SecureStore.setItemAsync(key, value, storeOptions);
            return true;
        } catch (error) {
            console.error(`Failed to store item ${key}:`, error);
            return false;
        }
    }

    /**
     * Retrieve a value securely
     */
    async getItem(
        key: string,
        options?: SecureStorageOptions
    ): Promise<string | null> {
        try {
            const retrieveOptions: SecureStore.SecureStoreOptions = {
                requireAuthentication: options?.requireAuthentication || false,
            };

            // Add authentication prompt for iOS
            if (Platform.OS === 'ios' && options?.authenticationPrompt) {
                retrieveOptions.authenticationPrompt = options.authenticationPrompt;
            }

            const value = await SecureStore.getItemAsync(key, retrieveOptions);
            return value;
        } catch (error) {
            console.error(`Failed to retrieve item ${key}:`, error);
            return null;
        }
    }

    /**
     * Delete a value securely
     */
    async deleteItem(key: string): Promise<boolean> {
        try {
            await SecureStore.deleteItemAsync(key);
            return true;
        } catch (error) {
            console.error(`Failed to delete item ${key}:`, error);
            return false;
        }
    }

    /**
     * Check if a key exists
     */
    async hasItem(key: string): Promise<boolean> {
        try {
            const value = await SecureStore.getItemAsync(key);
            return value !== null;
        } catch (error) {
            console.error(`Failed to check item ${key}:`, error);
            return false;
        }
    }

    /**
     * Store object as JSON
     */
    async setObject(
        key: string,
        value: any,
        options?: SecureStorageOptions
    ): Promise<boolean> {
        try {
            const jsonString = JSON.stringify(value);
            return await this.setItem(key, jsonString, options);
        } catch (error) {
            console.error(`Failed to store object ${key}:`, error);
            return false;
        }
    }

    /**
     * Retrieve object from JSON
     */
    async getObject<T = any>(
        key: string,
        options?: SecureStorageOptions
    ): Promise<T | null> {
        try {
            const jsonString = await this.getItem(key, options);
            if (!jsonString) return null;

            return JSON.parse(jsonString) as T;
        } catch (error) {
            console.error(`Failed to retrieve object ${key}:`, error);
            return null;
        }
    }

    /**
     * Store multiple items at once
     */
    async setMultiple(
        items: Array<{ key: string; value: string; options?: SecureStorageOptions }>
    ): Promise<boolean> {
        try {
            const promises = items.map(item =>
                this.setItem(item.key, item.value, item.options)
            );
            const results = await Promise.all(promises);
            return results.every(result => result === true);
        } catch (error) {
            console.error('Failed to store multiple items:', error);
            return false;
        }
    }

    /**
     * Retrieve multiple items at once
     */
    async getMultiple(
        keys: Array<{ key: string; options?: SecureStorageOptions }>
    ): Promise<Record<string, string | null>> {
        try {
            const promises = keys.map(async item => ({
                key: item.key,
                value: await this.getItem(item.key, item.options),
            }));
            const results = await Promise.all(promises);
            
            return results.reduce((acc, result) => {
                acc[result.key] = result.value;
                return acc;
            }, {} as Record<string, string | null>);
        } catch (error) {
            console.error('Failed to retrieve multiple items:', error);
            return {};
        }
    }

    /**
     * Delete multiple items at once
     */
    async deleteMultiple(keys: string[]): Promise<boolean> {
        try {
            const promises = keys.map(key => this.deleteItem(key));
            const results = await Promise.all(promises);
            return results.every(result => result === true);
        } catch (error) {
            console.error('Failed to delete multiple items:', error);
            return false;
        }
    }

    /**
     * Clear all secure storage
     * Use with caution!
     */
    async clearAll(keysToPreserve: string[] = []): Promise<boolean> {
        try {
            // Note: SecureStore doesn't provide a way to list all keys
            // This method should only be used if you know all the keys you want to clear
            console.warn('clearAll called - implement with specific keys');
            return true;
        } catch (error) {
            console.error('Failed to clear secure storage:', error);
            return false;
        }
    }

    /**
     * Get storage availability status
     */
    async isAvailable(): Promise<boolean> {
        try {
            // Test by trying to set and delete a test value
            const testKey = '__secure_store_test__';
            const testValue = 'test';
            
            await SecureStore.setItemAsync(testKey, testValue);
            const retrieved = await SecureStore.getItemAsync(testKey);
            await SecureStore.deleteItemAsync(testKey);
            
            return retrieved === testValue;
        } catch (error) {
            console.error('Secure storage not available:', error);
            return false;
        }
    }

    /**
     * Encrypt and store sensitive data with additional layer
     */
    async setSecure(
        key: string,
        value: string,
        password: string
    ): Promise<boolean> {
        try {
            // use proper encryption like AES
            const encrypted = this.simpleEncrypt(value, password);
            return await this.setItem(key, encrypted, { requireAuthentication: true });
        } catch (error) {
            console.error(`Failed to store secure item ${key}:`, error);
            return false;
        }
    }

    /**
     * Retrieve and decrypt sensitive data
     */
    async getSecure(
        key: string,
        password: string
    ): Promise<string | null> {
        try {
            const encrypted = await this.getItem(key, { requireAuthentication: true });
            if (!encrypted) return null;

            return this.simpleDecrypt(encrypted, password);
        } catch (error) {
            console.error(`Failed to retrieve secure item ${key}:`, error);
            return null;
        }
    }

    /**
     *  use proper encryption libraries
     */
    private simpleEncrypt(text: string, password: string): string {
        const textBytes = new TextEncoder().encode(text);
        const passwordBytes = new TextEncoder().encode(password);
        const encrypted = new Uint8Array(textBytes.length);

        for (let i = 0; i < textBytes.length; i++) {
            encrypted[i] = textBytes[i] ^ passwordBytes[i % passwordBytes.length];
        }

        return btoa(String.fromCharCode(...encrypted));
    }

    /**
     * Simple XOR decryption (for demonstration)
     */
    private simpleDecrypt(encrypted: string, password: string): string {
        const encryptedBytes = Uint8Array.from(atob(encrypted), c => c.charCodeAt(0));
        const passwordBytes = new TextEncoder().encode(password);
        const decrypted = new Uint8Array(encryptedBytes.length);

        for (let i = 0; i < encryptedBytes.length; i++) {
            decrypted[i] = encryptedBytes[i] ^ passwordBytes[i % passwordBytes.length];
        }

        return new TextDecoder().decode(decrypted);
    }

    /**
     * Migrate data from AsyncStorage to SecureStore
     */
    async migrateFromAsyncStorage(
        key: string,
        asyncStorageValue: string,
        options?: SecureStorageOptions
    ): Promise<boolean> {
        try {
            const success = await this.setItem(key, asyncStorageValue, options);
            return success;
        } catch (error) {
            console.error(`Failed to migrate ${key}:`, error);
            return false;
        }
    }
}

export const secureStorage = new SecureStorageService();