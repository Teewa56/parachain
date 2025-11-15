import { VALIDATION, REGEX } from './constants';

export const validatePin = (pin: string): { valid: boolean; error?: string } => {
    if (pin.length < VALIDATION.MIN_PIN_LENGTH) {
        return { valid: false, error: `PIN must be at least ${VALIDATION.MIN_PIN_LENGTH} digits` };
    }
    if (pin.length > VALIDATION.MAX_PIN_LENGTH) {
        return { valid: false, error: `PIN must not exceed ${VALIDATION.MAX_PIN_LENGTH} digits` };
    }
    if (!REGEX.NUMERIC.test(pin)) {
        return { valid: false, error: 'PIN must contain only numbers' };
    }
    return { valid: true };
};

export const validateDid = (did: string): boolean => {
    return REGEX.DID.test(did) && did.length >= VALIDATION.MIN_DID_LENGTH && did.length <= VALIDATION.MAX_DID_LENGTH;
};

export const validateHex = (hex: string): boolean => {
    return REGEX.HEX.test(hex);
};