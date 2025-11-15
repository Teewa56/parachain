import { blake2AsHex, blake2AsU8a } from '@polkadot/util-crypto';
import { u8aToHex, hexToU8a, stringToU8a, u8aToString } from '@polkadot/util';

/**
 * Hash a DID string to H256
 */
export function hashDid(did: string): string {
    return blake2AsHex(stringToU8a(did), 256);
}

/**
 * Hash any data to H256
 */
export function hashData(data: string | Uint8Array): string {
    if (typeof data === 'string') {
        return blake2AsHex(stringToU8a(data), 256);
    }
    return blake2AsHex(data, 256);
}

/**
 * Generate credential ID from components
 */
export function generateCredentialId(
    subjectDid: string,
    issuerDid: string,
    dataHash: string,
    issuedAt: number
): string {
    const data = new Uint8Array([
        ...hexToU8a(subjectDid),
        ...hexToU8a(issuerDid),
        ...hexToU8a(dataHash),
        ...new Uint8Array(new BigUint64Array([BigInt(issuedAt)]).buffer),
    ]);

    return blake2AsHex(data, 256);
}

/**
 * Format DID for display (shorten middle)
 */
export function formatDid(did: string, startChars: number = 10, endChars: number = 10): string {
    if (did.length <= startChars + endChars) {
        return did;
    }
    return `${did.slice(0, startChars)}...${did.slice(-endChars)}`;
}

/**
 * Format address for display
 */
export function formatAddress(address: string, startChars: number = 6, endChars: number = 4): string {
    if (address.length <= startChars + endChars) {
        return address;
    }
    return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
}

/**
 * Format balance (from planck to UNIT)
 */
export function formatBalance(balance: string, decimals: number = 12): string {
    const num = BigInt(balance);
    const divisor = BigInt(10 ** decimals);
    const whole = num / divisor;
    const fraction = num % divisor;

    if (fraction === BigInt(0)) {
        return whole.toString();
    }

    const fractionStr = fraction.toString().padStart(decimals, '0');
    const trimmed = fractionStr.replace(/0+$/, '');
    
    return `${whole}.${trimmed}`;
}

/**
 * Parse balance (from UNIT to planck)
 */
export function parseBalance(balance: string, decimals: number = 12): string {
    const [whole, fraction = ''] = balance.split('.');
    const fractionPadded = fraction.padEnd(decimals, '0').slice(0, decimals);
    return `${whole}${fractionPadded}`;
}

/**
 * Format timestamp to readable date
 */
export function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp * 1000); // Convert from seconds
    return date.toLocaleString();
}

/**
 * Format timestamp to relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(timestamp: number): string {
    const now = Date.now();
    const diff = now - timestamp * 1000; // Convert timestamp from seconds

    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);
    const months = Math.floor(days / 30);
    const years = Math.floor(days / 365);

    if (years > 0) return `${years} year${years > 1 ? 's' : ''} ago`;
    if (months > 0) return `${months} month${months > 1 ? 's' : ''} ago`;
    if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
    if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    return 'Just now';
}

/**
 * Check if credential is expired
 */
export function isCredentialExpired(expiresAt: number): boolean {
    if (expiresAt === 0) return false; // No expiration
    const now = Math.floor(Date.now() / 1000);
    return now > expiresAt;
}

/**
 * Get credential expiry status
 */
export function getExpiryStatus(expiresAt: number): {
    expired: boolean;
    expiringSoon: boolean;
    daysRemaining: number;
} {
    if (expiresAt === 0) {
        return { expired: false, expiringSoon: false, daysRemaining: -1 };
    }

    const now = Math.floor(Date.now() / 1000);
    const diff = expiresAt - now;
    const daysRemaining = Math.floor(diff / 86400);

    return {
        expired: diff <= 0,
        expiringSoon: diff > 0 && daysRemaining <= 30,
        daysRemaining: daysRemaining > 0 ? daysRemaining : 0,
    };
}

/**
 * Validate DID format
 */
export function validateDidFormat(did: string): boolean {
    // DID format: did:identity:identifier
    const didRegex = /^did:[a-z0-9]+:[a-z0-9\-_]+$/i;
    return didRegex.test(did) && did.length >= 7 && did.length <= 255;
}

/**
 * Generate DID from address
 */
export function generateDid(address: string, method: string = 'identity'): string {
    // Create unique identifier from address hash
    const identifier = blake2AsHex(address, 256).slice(2, 34); // Take first 32 chars
    return `did:${method}:${identifier}`;
}

/**
 * Convert hex string to bytes
 */
export function hexToBytes(hex: string): Uint8Array {
    return hexToU8a(hex);
}

/**
 * Convert bytes to hex string
 */
export function bytesToHex(bytes: Uint8Array): string {
    return u8aToHex(bytes);
}

/**
 * Parse error from dispatch error
 */
export function parseDispatchError(error: any): string {
    if (!error) return 'Unknown error';

    if (typeof error === 'string') return error;

    if (error.isModule) {
        try {
        const decoded = error.registry.findMetaError(error.asModule);
        return `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
        } catch (e) {
        return 'Module error (unable to decode)';
        }
    }

    if (error.isBadOrigin) return 'Bad origin';
    if (error.isCannotLookup) return 'Cannot lookup';
    if (error.isOther) return error.asOther.toString();

    return error.toString();
}

/**
 * Wait for finalization with timeout
 */
export async function waitForFinalization(
    promise: Promise<any>,
    timeoutMs: number = 30000
): Promise<any> {
    return Promise.race([
        promise,
        new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Transaction timeout')), timeoutMs)
        ),
    ]);
}

/**
 * Retry function with exponential backoff
 */
export async function retryWithBackoff<T>(
    fn: () => Promise<T>,
    maxRetries: number = 3,
    baseDelay: number = 1000
): Promise<T> {
    let lastError: Error | undefined;

    for (let i = 0; i < maxRetries; i++) {
        try {
        return await fn();
        } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        
        if (i < maxRetries - 1) {
            const delay = baseDelay * Math.pow(2, i);
            console.log(`Retry attempt ${i + 1}/${maxRetries} after ${delay}ms`);
            await new Promise((resolve) => setTimeout(resolve, delay));
        }
        }
    }

    throw lastError || new Error('Retry failed');
}

/**
 * Batch process with rate limiting
 */
export async function batchProcess<T, R>(
    items: T[],
    processFn: (item: T) => Promise<R>,
    batchSize: number = 10,
    delayMs: number = 100
): Promise<R[]> {
    const results: R[] = [];

    for (let i = 0; i < items.length; i += batchSize) {
        const batch = items.slice(i, i + batchSize);
        const batchResults = await Promise.all(batch.map(processFn));
        results.push(...batchResults);

        // Delay between batches
        if (i + batchSize < items.length) {
        await new Promise((resolve) => setTimeout(resolve, delayMs));
        }
    }

    return results;
}

/**
 * Calculate proof hash (for replay prevention)
 */
export function calculateProofHash(
    proofData: Uint8Array,
    publicInputs: Uint8Array[],
    credentialHash: string,
    nonce: string
): string {
    const data = new Uint8Array([
        ...proofData,
        ...publicInputs.flat(),
        ...hexToU8a(credentialHash),
        ...hexToU8a(nonce),
    ]);

    return blake2AsHex(data, 256);
}

/**
 * Check if transaction is finalized
 */
export function isFinalized(status: any): boolean {
    return status.isFinalized || status.isInBlock;
}

/**
 * Get block time estimate (in seconds)
 */
export function estimateBlockTime(blockCount: number, blockTimeSeconds: number = 6): number {
    return blockCount * blockTimeSeconds;
}

/**
 * Format block time
 */
export function formatBlockTime(seconds: number): string {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
    return `${Math.floor(seconds / 86400)}d`;
}