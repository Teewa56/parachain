import { useState, useEffect } from 'react';
import { biometricAuth } from '../services/storage/biometric';
import { useAuthStore } from '../store';

interface BiometricHook {
    isAvailable: boolean;
    isEnrolled: boolean;
    isEnabled: boolean;
    supportedTypes: string[];
    securityLevel: string;
    isLoading: boolean;
    error: string | null;
    authenticate: (reason?: string) => Promise<boolean>;
    enableBiometric: () => Promise<boolean>;
    disableBiometric: () => Promise<boolean>;
    checkCapabilities: () => Promise<void>;
}

export const useBiometric = (): BiometricHook => {
    const [isAvailable, setIsAvailable] = useState(false);
    const [isEnrolled, setIsEnrolled] = useState(false);
    const [isEnabled, setIsEnabled] = useState(false);
    const [supportedTypes, setSupportedTypes] = useState<string[]>([]);
    const [securityLevel, setSecurityLevel] = useState<string>('none');
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const authStore = useAuthStore();

    useEffect(() => {
        checkCapabilities();
    }, []);

    const checkCapabilities = async () => {
        try {
            setIsLoading(true);
            setError(null);

            const capabilities = await biometricAuth.checkCapabilities();
            setIsAvailable(capabilities.isAvailable);
            setIsEnrolled(capabilities.isEnrolled);
            setSupportedTypes(capabilities.supportedTypes);
            setSecurityLevel(capabilities.securityLevel);

            const enabled = await biometricAuth.isBiometricEnabled();
            setIsEnabled(enabled);
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to check biometric capabilities';
            setError(errorMessage);
        } finally {
            setIsLoading(false);
        }
    };

    const authenticate = async (reason: string = 'Authenticate to continue'): Promise<boolean> => {
        try {
            setError(null);
            const result = await biometricAuth.authenticate(reason);
            
            if (!result.success) {
                setError(result.error || 'Authentication failed');
                return false;
            }

            return true;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Authentication failed';
            setError(errorMessage);
            return false;
        }
    };

    const enableBiometric = async (): Promise<boolean> => {
        try {
            setError(null);
            const result = await biometricAuth.enableBiometric();
            
            if (!result.success) {
                setError(result.error || 'Failed to enable biometric');
                return false;
            }

            setIsEnabled(true);
            authStore.enableBiometric();
            return true;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to enable biometric';
            setError(errorMessage);
            return false;
        }
    };

    const disableBiometric = async (): Promise<boolean> => {
        try {
            setError(null);
            const success = await biometricAuth.disableBiometric();
            
            if (!success) {
                setError('Failed to disable biometric');
                return false;
            }

            setIsEnabled(false);
            authStore.disableBiometric();
            return true;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to disable biometric';
            setError(errorMessage);
            return false;
        }
    };

    return {
        isAvailable,
        isEnrolled,
        isEnabled,
        supportedTypes,
        securityLevel,
        isLoading,
        error,
        authenticate,
        enableBiometric,
        disableBiometric,
        checkCapabilities,
    };
};