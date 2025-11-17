import * as LocalAuthentication from 'expo-local-authentication';
import * as SecureStore from 'expo-secure-store';
import { Platform } from 'react-native';

interface BiometricAuthResult {
    success: boolean;
    error?: string;
    biometricType?: string;
}

interface BiometricCapabilities {
    isAvailable: boolean;
    isEnrolled: boolean;
    supportedTypes: string[];
    securityLevel: 'none' | 'weak' | 'strong';
}

const STORAGE_KEYS = {
    BIOMETRIC_ENABLED: '@identity_wallet/biometric_enabled',
    FALLBACK_PIN_HASH: '@identity_wallet/fallback_pin_hash',
    FAILED_ATTEMPTS: '@identity_wallet/failed_attempts',
    LAST_AUTH_TIME: '@identity_wallet/last_auth_time',
};

const MAX_FAILED_ATTEMPTS = 5;
const LOCKOUT_DURATION = 300000; // 5 minutes in milliseconds

class BiometricAuthService {
    private failedAttempts: number = 0;
    private lockoutUntil: number | null = null;

    /**
     * Check if device supports biometric authentication
     */
    async checkCapabilities(): Promise<BiometricCapabilities> {
        try {
            const hasHardware = await LocalAuthentication.hasHardwareAsync();
            if (!hasHardware) {
                return {
                    isAvailable: false,
                    isEnrolled: false,
                    supportedTypes: [],
                    securityLevel: 'none',
                };
            }

            const isEnrolled = await LocalAuthentication.isEnrolledAsync();
            const supportedTypes = await LocalAuthentication.supportedAuthenticationTypesAsync();

            const securityLevel = await LocalAuthentication.getEnrolledLevelAsync();

            return {
                isAvailable: hasHardware,
                isEnrolled,
                supportedTypes: this.mapAuthenticationTypes(supportedTypes),
                securityLevel: this.mapSecurityLevel(securityLevel),
            };
        } catch (error) {
            console.error('Error checking biometric capabilities:', error);
            return {
                isAvailable: false,
                isEnrolled: false,
                supportedTypes: [],
                securityLevel: 'none',
            };
        }
    }

    /**
     * Map authentication types to readable strings
     */
    private mapAuthenticationTypes(
        types: LocalAuthentication.AuthenticationType[]
    ): string[] {
        return types.map(type => {
            switch (type) {
                case LocalAuthentication.AuthenticationType.FINGERPRINT:
                    return 'Fingerprint';
                case LocalAuthentication.AuthenticationType.FACIAL_RECOGNITION:
                    return Platform.OS === 'ios' ? 'Face ID' : 'Face Recognition';
                case LocalAuthentication.AuthenticationType.IRIS:
                    return 'Iris';
                default:
                    return 'Biometric';
            }
        });
    }

    /**
     * Map security level
     */
    private mapSecurityLevel(level: LocalAuthentication.SecurityLevel): 'none' | 'weak' | 'strong' {
        switch (level) {
            case LocalAuthentication.SecurityLevel.NONE:
                return 'none';
            case LocalAuthentication.SecurityLevel.SECRET:
                return 'weak';
            case LocalAuthentication.SecurityLevel.BIOMETRIC_WEAK:
                return 'weak';
            case LocalAuthentication.SecurityLevel.BIOMETRIC_STRONG:
                return 'strong';
            default:
                return 'none';
        }
    }

    /**
     * Authenticate user with biometrics
     */
    async authenticate(reason: string = 'Authenticate to continue'): Promise<BiometricAuthResult> {
        // Check if locked out
        if (this.isLockedOut()) {
            return {
                success: false,
                error: `Too many failed attempts. Try again in ${this.getLockoutTimeRemaining()} seconds.`,
            };
        }

        try {
            // Check if biometric is enabled
            const enabled = await this.isBiometricEnabled();
            if (!enabled) {
                return {
                    success: false,
                    error: 'Biometric authentication is not enabled',
                };
            }

            // Check capabilities
            const capabilities = await this.checkCapabilities();
            if (!capabilities.isAvailable || !capabilities.isEnrolled) {
                return {
                    success: false,
                    error: 'Biometric authentication is not available',
                };
            }

            // Perform authentication
            const result = await LocalAuthentication.authenticateAsync({
                promptMessage: reason,
                cancelLabel: 'Cancel',
                disableDeviceFallback: false,
                fallbackLabel: 'Use PIN',
            });

            if (result.success) {
                // Reset failed attempts
                this.failedAttempts = 0;
                await this.saveFailedAttempts(0);
                await this.saveLastAuthTime();

                return {
                    success: true,
                    biometricType: capabilities.supportedTypes[0] || 'Biometric',
                };
            } else {
                // Increment failed attempts
                await this.incrementFailedAttempts();

                return {
                    success: false,
                    error: result.error || 'Authentication failed',
                };
            }
        } catch (error) {
            console.error('Biometric authentication error:', error);
            await this.incrementFailedAttempts();

            return {
                success: false,
                error: error instanceof Error ? error.message : 'Authentication failed',
            };
        }
    }

    /**
     * Enable biometric authentication
     */
    async enableBiometric(): Promise<BiometricAuthResult> {
        try {
            const capabilities = await this.checkCapabilities();
            
            if (!capabilities.isAvailable) {
                return {
                    success: false,
                    error: 'Biometric hardware not available',
                };
            }

            if (!capabilities.isEnrolled) {
                return {
                    success: false,
                    error: 'No biometrics enrolled. Please set up biometrics in device settings.',
                };
            }

            // Test biometric authentication
            const testResult = await this.authenticate('Enable biometric authentication');
            
            if (testResult.success) {
                await SecureStore.setItemAsync(STORAGE_KEYS.BIOMETRIC_ENABLED, 'true');
                return {
                    success: true,
                    biometricType: testResult.biometricType,
                };
            }

            return testResult;
        } catch (error) {
            console.error('Error enabling biometric:', error);
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Failed to enable biometric',
            };
        }
    }

    /**
     * Disable biometric authentication
     */
    async disableBiometric(): Promise<boolean> {
        try {
            await SecureStore.deleteItemAsync(STORAGE_KEYS.BIOMETRIC_ENABLED);
            return true;
        } catch (error) {
            console.error('Error disabling biometric:', error);
            return false;
        }
    }

    /**
     * Check if biometric is enabled
     */
    async isBiometricEnabled(): Promise<boolean> {
        try {
            const enabled = await SecureStore.getItemAsync(STORAGE_KEYS.BIOMETRIC_ENABLED);
            return enabled === 'true';
        } catch (error) {
            console.error('Error checking biometric status:', error);
            return false;
        }
    }

    /**
     * Set fallback PIN
     */
    async setFallbackPin(pin: string): Promise<boolean> {
        try {
            // Hash the PIN before storing
            const pinHash = await this.hashPin(pin);
            await SecureStore.setItemAsync(
                STORAGE_KEYS.FALLBACK_PIN_HASH,
                pinHash,
                {
                    requireAuthentication: false, // PIN is fallback, no auth needed
                }
            );
            return true;
        } catch (error) {
            console.error('Error setting fallback PIN:', error);
            return false;
        }
    }

    /**
     * Verify fallback PIN
     */
    async verifyFallbackPin(pin: string): Promise<boolean> {
        // Check if locked out
        if (this.isLockedOut()) {
            return false;
        }

        try {
            const storedHash = await SecureStore.getItemAsync(STORAGE_KEYS.FALLBACK_PIN_HASH);
            if (!storedHash) {
                return false;
            }

            const pinHash = await this.hashPin(pin);
            const isValid = pinHash === storedHash;

            if (isValid) {
                // Reset failed attempts
                this.failedAttempts = 0;
                await this.saveFailedAttempts(0);
                await this.saveLastAuthTime();
            } else {
                // Increment failed attempts
                await this.incrementFailedAttempts();
            }

            return isValid;
        } catch (error) {
            console.error('Error verifying PIN:', error);
            await this.incrementFailedAttempts();
            return false;
        }
    }

    /**
     * Hash PIN using simple algorithm
     */
    private async hashPin(pin: string): Promise<string> {
        // use PBKDF2
        let hash = 0;
        for (let i = 0; i < pin.length; i++) {
            const char = pin.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32bit integer
        }
        return hash.toString(36);
    }

    /**
     * Check if PIN exists
     */
    async hasFallbackPin(): Promise<boolean> {
        try {
            const pinHash = await SecureStore.getItemAsync(STORAGE_KEYS.FALLBACK_PIN_HASH);
            return !!pinHash;
        } catch (error) {
            console.error('Error checking PIN:', error);
            return false;
        }
    }

    /**
     * Increment failed authentication attempts
     */
    private async incrementFailedAttempts(): Promise<void> {
        this.failedAttempts++;
        await this.saveFailedAttempts(this.failedAttempts);

        if (this.failedAttempts >= MAX_FAILED_ATTEMPTS) {
            this.lockoutUntil = Date.now() + LOCKOUT_DURATION;
        }
    }

    /**
     * Save failed attempts to storage
     */
    private async saveFailedAttempts(count: number): Promise<void> {
        try {
            await SecureStore.setItemAsync(
                STORAGE_KEYS.FAILED_ATTEMPTS,
                count.toString()
            );
        } catch (error) {
            console.error('Error saving failed attempts:', error);
        }
    }

    /**
     * Load failed attempts from storage
     */
    async loadFailedAttempts(): Promise<void> {
        try {
            const attemptsStr = await SecureStore.getItemAsync(STORAGE_KEYS.FAILED_ATTEMPTS);
            this.failedAttempts = attemptsStr ? parseInt(attemptsStr, 10) : 0;
        } catch (error) {
            console.error('Error loading failed attempts:', error);
            this.failedAttempts = 0;
        }
    }

    /**
     * Check if user is locked out
     */
    private isLockedOut(): boolean {
        if (!this.lockoutUntil) return false;
        
        if (Date.now() < this.lockoutUntil) {
            return true;
        }

        // Lockout expired
        this.lockoutUntil = null;
        return false;
    }

    /**
     * Get remaining lockout time in seconds
     */
    private getLockoutTimeRemaining(): number {
        if (!this.lockoutUntil) return 0;
        
        const remaining = Math.ceil((this.lockoutUntil - Date.now()) / 1000);
        return Math.max(0, remaining);
    }

    /**
     * Save last successful authentication time
     */
    private async saveLastAuthTime(): Promise<void> {
        try {
            await SecureStore.setItemAsync(
                STORAGE_KEYS.LAST_AUTH_TIME,
                Date.now().toString()
            );
        } catch (error) {
            console.error('Error saving auth time:', error);
        }
    }

    /**
     * Get time since last authentication in seconds
     */
    async getTimeSinceLastAuth(): Promise<number> {
        try {
            const lastAuthStr = await SecureStore.getItemAsync(STORAGE_KEYS.LAST_AUTH_TIME);
            if (!lastAuthStr) return Infinity;

            const lastAuth = parseInt(lastAuthStr, 10);
            return Math.floor((Date.now() - lastAuth) / 1000);
        } catch (error) {
            console.error('Error getting last auth time:', error);
            return Infinity;
        }
    }

    /**
     * Check if re-authentication is required
     * @param maxAgeSeconds Maximum age in seconds before re-auth required
     */
    async requiresReauth(maxAgeSeconds: number = 300): Promise<boolean> {
        const timeSinceAuth = await this.getTimeSinceLastAuth();
        return timeSinceAuth >= maxAgeSeconds;
    }

    /**
     * Clear all authentication data
     */
    async clearAuthData(): Promise<void> {
        try {
            await SecureStore.deleteItemAsync(STORAGE_KEYS.BIOMETRIC_ENABLED);
            await SecureStore.deleteItemAsync(STORAGE_KEYS.FALLBACK_PIN_HASH);
            await SecureStore.deleteItemAsync(STORAGE_KEYS.FAILED_ATTEMPTS);
            await SecureStore.deleteItemAsync(STORAGE_KEYS.LAST_AUTH_TIME);
            
            this.failedAttempts = 0;
            this.lockoutUntil = null;
        } catch (error) {
            console.error('Error clearing auth data:', error);
        }
    }

    /**
     * Validate PIN format
     */
    validatePinFormat(pin: string): { valid: boolean; error?: string } {
        if (!pin || pin.length < 6) {
            return {
                valid: false,
                error: 'PIN must be at least 6 digits',
            };
        }

        if (pin.length > 8) {
            return {
                valid: false,
                error: 'PIN must not exceed 8 digits',
            };
        }

        if (!/^\d+$/.test(pin)) {
            return {
                valid: false,
                error: 'PIN must contain only numbers',
            };
        }

        // Check for weak PINs
        if (this.isWeakPin(pin)) {
            return {
                valid: false,
                error: 'PIN is too weak. Avoid sequential or repeated digits',
            };
        }

        return { valid: true };
    }

    /**
     * Check for weak PIN patterns
     */
    private isWeakPin(pin: string): boolean {
        // Check for all same digits
        if (/^(\d)\1+$/.test(pin)) {
            return true;
        }

        // Check for sequential ascending
        const isAscending = pin.split('').every((digit, i, arr) => {
            if (i === 0) return true;
            return parseInt(digit) === parseInt(arr[i - 1]) + 1;
        });
        if (isAscending) return true;

        // Check for sequential descending
        const isDescending = pin.split('').every((digit, i, arr) => {
            if (i === 0) return true;
            return parseInt(digit) === parseInt(arr[i - 1]) - 1;
        });
        if (isDescending) return true;

        // Check for common patterns
        const commonPatterns = ['123456', '654321', '111111', '000000', '123123'];
        if (commonPatterns.includes(pin)) {
            return true;
        }

        return false;
    }

    /**
     * Get biometric type name for display
     */
    async getBiometricTypeName(): Promise<string> {
        const capabilities = await this.checkCapabilities();
        
        if (capabilities.supportedTypes.length > 0) {
            return capabilities.supportedTypes[0];
        }

        return 'Biometric';
    }

    /**
     * Test biometric authentication (for setup)
     */
    async testBiometric(): Promise<BiometricAuthResult> {
        return this.authenticate('Test biometric authentication');
    }
}

export const biometricAuth = new BiometricAuthService();