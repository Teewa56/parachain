import { ERROR_MESSAGES } from './constants';

export class AppError extends Error {
    code: string;
    details?: any;

    constructor(message: string, code: string, details?: any) {
        super(message);
        this.code = code;
        this.details = details;
        this.name = 'AppError';
    }
}

export const createError = (code: keyof typeof ERROR_MESSAGES, details?: any): AppError => {
    const message = ERROR_MESSAGES[code];
    return new AppError(message, code, details);
};

export const handleError = (error: unknown): string => {
    if (error instanceof AppError) {
        return error.message;
    }
    if (error instanceof Error) {
        return error.message;
    }
    return ERROR_MESSAGES.UNKNOWN_ERROR;
};