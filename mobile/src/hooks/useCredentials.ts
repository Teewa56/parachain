import { useCredentialStore } from '../store';

export const useCredentials = () => {
    const store = useCredentialStore();

    return {
        credentials: store.credentials,
        isLoading: store.isLoading,
        fetchCredentials: store.fetchCredentials,
        getCredentialById: store.getCredentialById,
        refreshCredential: store.refreshCredential,
        getActiveCredentials: store.getActiveCredentials,
        filterByType: store.filterByType,
    };
};