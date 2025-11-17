import { useEffect } from 'react';

interface ConfirmModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    type?: 'danger' | 'warning' | 'info';
    loading?: boolean;
}

export function ConfirmModal({
    isOpen,
    onClose,
    onConfirm,
    title,
    message,
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    type = 'info',
    loading = false,
}: ConfirmModalProps) {
    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && !loading) onClose();
        };
        if (isOpen) {
            document.addEventListener('keydown', handleEscape);
            document.body.style.overflow = 'hidden';
        }
        return () => {
            document.removeEventListener('keydown', handleEscape);
            document.body.style.overflow = 'unset';
        };
    }, [isOpen, loading, onClose]);

    if (!isOpen) return null;

    const typeColors = {
        danger: 'from-red-500 to-red-600',
        warning: 'from-yellow-500 to-yellow-600',
        info: 'from-blue-500 to-purple-600',
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" onClick={loading ? undefined : onClose} />
        <div className="relative bg-slate-800 border border-slate-700 rounded-xl shadow-2xl max-w-md w-full p-6 animate-scale-in">
            <div className="mb-4">
            <h3 className="text-xl font-bold text-white mb-2">{title}</h3>
            <p className="text-slate-400">{message}</p>
            </div>
            <div className="flex gap-3">
            <button
                onClick={onClose}
                disabled={loading}
                className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition disabled:opacity-50"
            >
                {cancelText}
            </button>
            <button
                onClick={onConfirm}
                disabled={loading}
                className={`flex-1 px-4 py-2 bg-gradient-to-r ${typeColors[type]} text-white rounded-lg font-semibold transition disabled:opacity-50`}
            >
                {loading ? 'Processing...' : confirmText}
            </button>
            </div>
        </div>
        </div>
    );
}
