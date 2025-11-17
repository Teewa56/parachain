import { create } from 'zustand';
import { Proposal } from '../types/governance';

interface GovernanceState {
    proposals: Proposal[];
    loading: boolean;
    error: string | null;
    setProposals: (proposals: Proposal[]) => void;
    addProposal: (proposal: Proposal) => void;
    updateProposal: (id: number, updates: Partial<Proposal>) => void;
    setLoading: (loading: boolean) => void;
    setError: (error: string | null) => void;
    clearError: () => void;
}

export const useGovernanceStore = create<GovernanceState>((set) => ({
    proposals: [],
    loading: false,
    error: null,

    setProposals: (proposals) => set({ proposals }),

    addProposal: (proposal) =>
        set((state) => ({
        proposals: [proposal, ...state.proposals],
        })),

    updateProposal: (id, updates) =>
        set((state) => ({
        proposals: state.proposals.map((p) =>
            p.id === id ? { ...p, ...updates } : p
        ),
        })),

    setLoading: (loading) => set({ loading }),

    setError: (error) => set({ error }),

    clearError: () => set({ error: null }),
}));