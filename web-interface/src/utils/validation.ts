export const validation = {
    isDID(value: string): boolean {
        return /^did:[a-z]+:[a-zA-Z0-9._-]+$/.test(value);
    },

    isHex(value: string): boolean {
        return /^0x[a-fA-F0-9]+$/.test(value);
    },

    isHash256(value: string): boolean {
        return /^0x[a-fA-F0-9]{64}$/.test(value);
    },

    isAddress(value: string): boolean {
        return /^[0-9a-fA-F]{48}$/.test(value) || /^0x[0-9a-fA-F]{40}$/.test(value);
    },

    validateCredentialData(data: any): { valid: boolean; errors: string[] } {
        const errors: string[] = [];

        if (!data.subjectDid || !this.isDID(data.subjectDid)) {
        errors.push('Invalid subject DID format');
        }

        if (!data.dataHash || !this.isHash256(data.dataHash)) {
        errors.push('Invalid data hash format');
        }

        if (data.expiresAt && data.expiresAt <= Date.now()) {
        errors.push('Expiration date must be in the future');
        }

        return {
            valid: errors.length === 0,
            errors,
        };
    },
};