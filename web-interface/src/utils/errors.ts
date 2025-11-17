export class SubstrateError extends Error {
    constructor(
        message: string,
        public code?: string,
        public details?: any
    ) {
        super(message);
        this.name = 'SubstrateError';
    }
}

export class ValidationError extends Error {
    constructor(
        message: string,
        public field?: string
    ) {
        super(message);
        this.name = 'ValidationError';
    }
}

export const errorHandler = {
    parse(error: any): { message: string; code?: string; details?: any } {
        if (error instanceof SubstrateError) {
        return {
            message: error.message,
            code: error.code,
            details: error.details,
        };
        }

        if (error.message?.includes('1010')) {
        return {
            message: 'Insufficient balance to pay transaction fees',
            code: 'INSUFFICIENT_BALANCE',
        };
        }

        if (error.message?.includes('BadOrigin')) {
        return {
            message: 'Not authorized to perform this action',
            code: 'UNAUTHORIZED',
        };
        }

        return {
        message: error.message || 'An unknown error occurred',
        code: 'UNKNOWN',
        };
    },

    handle(error: any, onError?: (err: any) => void) {
        const parsed = this.parse(error);
        console.error(`[${parsed.code}]`, parsed.message, parsed.details);
        
        if (onError) {
        onError(parsed);
        }

        return parsed;
    },
};
