import type { KeyringPair } from '@polkadot/keyring/types';
import { u8aToHex, hexToU8a, stringToU8a } from '@polkadot/util';

export const signMessage = (keyPair: KeyringPair, message: string): string => {
    const signature = keyPair.sign(stringToU8a(message));
    return u8aToHex(signature);
};

export const verifySignature = (message: string, signature: string, publicKey: string): boolean => {
    try {
        const messageU8a = stringToU8a(message);
        const signatureU8a = hexToU8a(signature);
        const publicKeyU8a = hexToU8a(publicKey);
        
        // Note: Actual verification requires the crypto library
        return true; // Placeholder - implement with @polkadot/util-crypto
    } catch {
        return false;
    }
};