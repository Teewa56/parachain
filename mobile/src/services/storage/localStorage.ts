import AsyncStorage from '@react-native-async-storage/async-storage';

export const localStorage = {
    async setItem(key: string, value: string): Promise<void> {
        await AsyncStorage.setItem(key, value);
    },

    async getItem(key: string): Promise<string | null> {
        return await AsyncStorage.getItem(key);
    },

    async removeItem(key: string): Promise<void> {
        await AsyncStorage.removeItem(key);
    },

    async clear(): Promise<void> {
        await AsyncStorage.clear();
    },

    async getAllKeys(): Promise<string[]> {
        return await AsyncStorage.getAllKeys();
    },

    async multiGet(keys: string[]): Promise<[string, string | null][]> {
        return await AsyncStorage.multiGet(keys);
    },

    async multiSet(keyValuePairs: [string, string][]): Promise<void> {
        await AsyncStorage.multiSet(keyValuePairs);
    },

    async multiRemove(keys: string[]): Promise<void> {
        await AsyncStorage.multiRemove(keys);
    },
};