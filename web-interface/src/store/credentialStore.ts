import { create } from 'zustand';
import { Credential } from '../types/credential';

interface CredentialState {
    credentials: Credential[];
    loading: boolean;
    error: string | null;
    setCredentials: (credentials: Credential[]) => void;
    addCredential: (credential: Credential) => void;
    updateCredential: (id: string, updates: Partial<Credential>) => void;
    removeCredential: (id: string) => void;
    setLoading: (loading: boolean) => void;
    setError: (error: string | null) => void;
    clearError: () => void;
}

export const useCredentialStore = create<CredentialState>((set) => ({
    credentials: [],
    loading: false,
    error: null,

    setCredentials: (credentials) => set({ credentials }),

    addCredential: (credential) =>
        set((state) => ({
        credentials: [credential, ...state.credentials],
        })),

    updateCredential: (id, updates) =>
        set((state) => ({
        credentials: state.credentials.map((c) =>
            c.id === id ? { ...c, ...updates } : c
        ),
        })),

    removeCredential: (id) =>
        set((state) => ({
        credentials: state.credentials.filter((c) => c.id !== id),
        })),

    setLoading: (loading) => set({ loading }),

    setError: (error) => set({ error }),

    clearError: () => set({ error: null }),
}));