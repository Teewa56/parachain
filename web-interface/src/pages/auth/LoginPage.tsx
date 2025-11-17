import { useState } from 'react';
import { useAuth } from '../../hooks/useAuth';
import { useNavigate } from 'react-router-dom';

export function LoginPage() {
    const { initialize, accounts, selectAccount, selectedAccount } = useAuth();
    const [loading, setLoading] = useState(false);
    const navigate = useNavigate();

    const handleConnect = async () => {
        setLoading(true);
        try {
        await initialize();
        } catch (error) {
        console.error('Connection failed:', error);
        } finally {
        setLoading(false);
        }
    };

    const handleSelectAccount = (account: any) => {
        selectAccount(account);
        navigate('/dashboard');
    };

    return (
        <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-4">
        <div className="max-w-md w-full space-y-8 bg-slate-800/50 backdrop-blur border border-slate-700 rounded-2xl p-8">
            <div className="text-center">
            <div className="w-20 h-20 bg-gradient-to-br from-blue-400 to-purple-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
                <span className="text-3xl font-bold text-white">P</span>
            </div>
            <h2 className="text-3xl font-bold text-white mb-2">Welcome to PortableID</h2>
            <p className="text-slate-400">Connect your wallet to continue</p>
            </div>

            {!accounts.length ? (
            <button
                onClick={handleConnect}
                disabled={loading}
                className="w-full px-6 py-4 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg font-semibold transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
                {loading ? 'Connecting...' : 'Connect Wallet'}
            </button>
            ) : (
            <div className="space-y-3">
                <p className="text-sm text-slate-400 mb-3">Select an account:</p>
                {accounts.map((account) => (
                <button
                    key={account.address}
                    onClick={() => handleSelectAccount(account)}
                    className="w-full p-4 bg-slate-700/50 hover:bg-slate-700 border border-slate-600 rounded-lg text-left transition"
                >
                    <div className="font-medium text-white mb-1">{account.meta.name || 'Account'}</div>
                    <div className="text-xs font-mono text-slate-400">{account.address}</div>
                </button>
                ))}
            </div>
            )}

            <div className="text-center text-sm text-slate-500">
            Don't have a wallet?{' '}
            <a
                href="https://polkadot.js.org/extension/"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-400 hover:text-blue-300"
            >
                Install Polkadot.js
            </a>
            </div>
        </div>
        </div>
    );
}