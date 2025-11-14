import { create } from 'zustand';

export interface Toast {
    id: string;
    message: string;
    type: 'success' | 'error' | 'warning' | 'info';
    duration?: number;
}

interface UIState {
    isLoading: boolean;
    loadingMessage: string | null;
    error: string | null;
    toast: Toast | null;
    
    // Actions
    setLoading: (loading: boolean, message?: string) => void;
    showToast: (message: string, type?: Toast['type'], duration?: number) => void;
    hideToast: () => void;
    setError: (error: string | null) => void;
    clearError: () => void;
}

export const useUIStore = create<UIState>((set, get) => ({
    isLoading: false,
    loadingMessage: null,
    error: null,
    toast: null,

    setLoading: (loading: boolean, message?: string) => {
        set({ 
            isLoading: loading, 
            loadingMessage: loading ? (message || null) : null 
        });
    },

    showToast: (message: string, type: Toast['type'] = 'info', duration: number = 3000) => {
        const toast: Toast = {
            id: Date.now().toString(),
            message,
            type,
            duration
        };

        set({ toast });

        if (duration > 0) {
            setTimeout(() => {
                const currentToast = get().toast;
                if (currentToast?.id === toast.id) {
                    set({ toast: null });
                }
            }, duration);
        }
    },

    hideToast: () => {
        set({ toast: null });
    },

    setError: (error: string | null) => {
        set({ error });
        
        if (error) {
            get().showToast(error, 'error', 5000);
        }
    },

    clearError: () => {
        set({ error: null });
    }
}));