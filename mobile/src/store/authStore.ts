import { create } from 'zustand';
import * as LocalAuthentication from 'expo-local-authentication';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { STORAGE_KEYS, ERROR_MESSAGES } from '../utils/constants';

interface AuthState {
    isAuthenticated: boolean;
    biometricEnabled: boolean;
    hasSeedPhrase: boolean;
    hasCompletedOnboarding: boolean;
    isLoading: boolean;
    error: string | null;
    
    // Actions
    login: () => Promise<void>;
    logout: () => Promise<void>;
    enableBiometric: () => Promise<void>;
    disableBiometric: () => Promise<void>;
    authenticateWithBiometric: () => Promise<boolean>;
    checkBiometricAvailability: () => Promise<boolean>;
    setHasSeedPhrase: (value: boolean) => void;
    completeOnboarding: () => Promise<void>;
    loadAuthState: () => Promise<void>;
    setError: (error: string | null) => void;
}

export const useAuthStore = create<AuthState>((set, get) => ({
    isAuthenticated: false,
    biometricEnabled: false,
    hasSeedPhrase: false,
    hasCompletedOnboarding: false,
    isLoading: false,
    error: null,

    login: async () => {
        set({ isLoading: true, error: null });
        
        try {
            await AsyncStorage.setItem(STORAGE_KEYS.IS_AUTHENTICATED, 'true');
            await AsyncStorage.setItem(
                STORAGE_KEYS.LAST_LOGIN, 
                Date.now().toString()
            );
            
            set({ 
                isAuthenticated: true, 
                isLoading: false 
            });
            
            console.log('User logged in successfully');
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Login failed:', error);
            throw error;
        }
    },

    logout: async () => {
        set({ isLoading: true, error: null });
        
        try {
            await AsyncStorage.removeItem(STORAGE_KEYS.IS_AUTHENTICATED);
            
            set({ 
                isAuthenticated: false,
                isLoading: false 
            });
            
            console.log('User logged out successfully');
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Logout failed:', error);
            throw error;
        }
    },

    enableBiometric: async () => {
        set({ isLoading: true, error: null });
        
        try {
            const isAvailable = await get().checkBiometricAvailability();
            
            if (!isAvailable) {
                throw new Error(ERROR_MESSAGES.BIOMETRIC_NOT_AVAILABLE);
            }

            const result = await LocalAuthentication.authenticateAsync({
                promptMessage: 'Enable biometric authentication',
                disableDeviceFallback: false,
            });

            if (!result.success) {
                throw new Error(ERROR_MESSAGES.BIOMETRIC_FAILED);
            }

            await AsyncStorage.setItem(STORAGE_KEYS.BIOMETRIC_ENABLED, 'true');
            
            set({ 
                biometricEnabled: true, 
                isLoading: false 
            });
            
            console.log('Biometric authentication enabled');
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Enable biometric failed:', error);
            throw error;
        }
    },

    disableBiometric: async () => {
        set({ isLoading: true, error: null });
        
        try {
            await AsyncStorage.removeItem(STORAGE_KEYS.BIOMETRIC_ENABLED);
            
            set({ 
                biometricEnabled: false, 
                isLoading: false 
            });
            
            console.log('Biometric authentication disabled');
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Disable biometric failed:', error);
            throw error;
        }
    },

    authenticateWithBiometric: async (): Promise<boolean> => {
        set({ isLoading: true, error: null });
        
        try {
            const isAvailable = await get().checkBiometricAvailability();
            
            if (!isAvailable) {
                throw new Error(ERROR_MESSAGES.BIOMETRIC_NOT_AVAILABLE);
            }

            const result = await LocalAuthentication.authenticateAsync({
                promptMessage: 'Authenticate to continue',
                disableDeviceFallback: false,
                cancelLabel: 'Cancel',
            });

            set({ isLoading: false });

            if (result.success) {
                console.log('Biometric authentication successful');
                return true;
            } else {
                console.log('Biometric authentication failed');
                return false;
            }
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.BIOMETRIC_FAILED;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Biometric authentication error:', error);
            return false;
        }
    },

    checkBiometricAvailability: async (): Promise<boolean> => {
        try {
            const compatible = await LocalAuthentication.hasHardwareAsync();
            const enrolled = await LocalAuthentication.isEnrolledAsync();
            
            return compatible && enrolled;
        } catch (error) {
            console.error('Check biometric availability error:', error);
            return false;
        }
    },

    setHasSeedPhrase: (value: boolean) => {
        set({ hasSeedPhrase: value });
    },

    completeOnboarding: async () => {
        set({ isLoading: true, error: null });
        
        try {
            await AsyncStorage.setItem(
                STORAGE_KEYS.HAS_COMPLETED_ONBOARDING, 
                'true'
            );
            
            set({ 
                hasCompletedOnboarding: true, 
                isLoading: false 
            });
            
            console.log('Onboarding completed');
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Complete onboarding failed:', error);
            throw error;
        }
    },

    loadAuthState: async () => {
        set({ isLoading: true, error: null });
        
        try {
            const [
                isAuthenticated,
                biometricEnabled,
                hasCompletedOnboarding
            ] = await Promise.all([
                AsyncStorage.getItem(STORAGE_KEYS.IS_AUTHENTICATED),
                AsyncStorage.getItem(STORAGE_KEYS.BIOMETRIC_ENABLED),
                AsyncStorage.getItem(STORAGE_KEYS.HAS_COMPLETED_ONBOARDING)
            ]);

            set({
                isAuthenticated: isAuthenticated === 'true',
                biometricEnabled: biometricEnabled === 'true',
                hasCompletedOnboarding: hasCompletedOnboarding === 'true',
                isLoading: false
            });
            
            console.log('Auth state loaded successfully');
        } catch (error) {
            const errorMessage = error instanceof Error 
                ? error.message 
                : ERROR_MESSAGES.UNKNOWN_ERROR;
            
            set({ 
                error: errorMessage, 
                isLoading: false 
            });
            
            console.error('Load auth state failed:', error);
        }
    },

    setError: (error: string | null) => {
        set({ error });
    }
}));