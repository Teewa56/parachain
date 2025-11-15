import { useAuthStore } from '../store';

export const useAuth = () => {
    const store = useAuthStore();

    return {
        isAuthenticated: store.isAuthenticated,
        biometricEnabled: store.biometricEnabled,
        login: store.login,
        logout: store.logout,
        enableBiometric: store.enableBiometric,
        disableBiometric: store.disableBiometric,
        authenticateWithBiometric: store.authenticateWithBiometric,
    };
};