import { create } from 'zustand';

interface Modal {
    type: 'confirm' | 'error' | 'success';
    title: string;
    message: string;
    onConfirm?: () => void;
    onClose?: () => void;
}

interface Toast {
    id: string;
    type: 'success' | 'error' | 'info' | 'warning';
    message: string;
    duration?: number;
}

interface UIState {
    sidebarOpen: boolean;
    modal: Modal | null;
    toasts: Toast[];
    loading: boolean;
    toggleSidebar: () => void;
    setSidebarOpen: (open: boolean) => void;
    showModal: (modal: Modal) => void;
    hideModal: () => void;
    addToast: (toast: Omit<Toast, 'id'>) => void;
    removeToast: (id: string) => void;
    setLoading: (loading: boolean) => void;
}

export const useUIStore = create<UIState>((set) => ({
    sidebarOpen: true,
    modal: null,
    toasts: [],
    loading: false,

    toggleSidebar: () =>
        set((state) => ({ sidebarOpen: !state.sidebarOpen })),

    setSidebarOpen: (open) =>
        set({ sidebarOpen: open }),

    showModal: (modal) =>
        set({ modal }),

    hideModal: () =>
        set({ modal: null }),

    addToast: (toast) =>
        set((state) => ({
        toasts: [
            ...state.toasts,
            { ...toast, id: Math.random().toString(36).substring(7) },
        ],
        })),

    removeToast: (id) =>
        set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
        })),

    setLoading: (loading) =>
        set({ loading }),
}));