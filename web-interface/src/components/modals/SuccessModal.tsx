import { useEffect } from "react";

interface SuccessModalProps {
    isOpen: boolean;
    onClose: () => void;
    title?: string;
    message: string;
    transactionHash?: string;
    autoCloseMs?: number;
}

export function SuccessModal({
    isOpen,
    onClose,
    title = 'Success!',
    message,
    transactionHash,
    autoCloseMs,
}: SuccessModalProps) {
    useEffect(() => {
        if (isOpen && autoCloseMs) {
        const timer = setTimeout(onClose, autoCloseMs);
        return () => clearTimeout(timer);
        }
    }, [isOpen, autoCloseMs, onClose]);

    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
        if (e.key === 'Escape') onClose();
        };
        if (isOpen) {
        document.addEventListener('keydown', handleEscape);
        document.body.style.overflow = 'hidden';
        }
        return () => {
        document.removeEventListener('keydown', handleEscape);
        document.body.style.overflow = 'unset';
        };
    }, [isOpen, onClose]);

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" onClick={onClose} />
        <div className="relative bg-slate-800 border border-green-500/50 rounded-xl shadow-2xl max-w-md w-full p-6 animate-scale-in">
            <div className="flex items-start gap-4 mb-4">
            <div className="w-12 h-12 bg-green-500/20 rounded-full flex items-center justify-center text-2xl flex-shrink-0">
                ✅
            </div>
            <div className="flex-1">
                <h3 className="text-xl font-bold text-white mb-2">{title}</h3>
                <p className="text-slate-300">{message}</p>
                {transactionHash && (
                <div className="mt-3 p-3 bg-slate-900/50 rounded-lg border border-slate-700">
                    <p className="text-xs text-slate-500 mb-1">Transaction Hash:</p>
                    <p className="text-xs text-green-400 font-mono break-all">{transactionHash}</p>
                </div>
                )}
            </div>
            </div>
            <button
            onClick={onClose}
            className="w-full px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg font-semibold transition"
            >
            Close
            </button>
        </div>
        </div>
    );
}