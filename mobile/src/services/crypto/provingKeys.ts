import * as FileSystem from 'expo-file-system';
import { Asset } from 'expo-asset';

class ProvingKeyManager {
    private keyCache: Map<string, string> = new Map();

    async getProvingKey(circuitId: string): Promise<string> {
        // Check cache
        if (this.keyCache.has(circuitId)) {
            return this.keyCache.get(circuitId)!;
        }

        // Load from assets
        const asset = Asset.fromModule(
            require(`../../assets/proving-keys/${circuitId}.pk`)
        );
        
        await asset.downloadAsync();
        
        if (!asset.localUri) {
            throw new Error(`Failed to load proving key for ${circuitId}`);
        }

        // Read file as base64
        const base64Key = await FileSystem.readAsStringAsync(
            asset.localUri,
            { encoding: FileSystem.EncodingType.Base64 }
        );

        // Cache it
        this.keyCache.set(circuitId, base64Key);
        
        return base64Key;
    }

    clearCache() {
        this.keyCache.clear();
    }
}

export const provingKeyManager = new ProvingKeyManager();