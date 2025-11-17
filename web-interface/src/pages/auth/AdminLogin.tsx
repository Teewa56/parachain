import { useState } from 'react';
import { useAuth } from '../../hooks/useAuth';
import { useNavigate } from 'react-router-dom';

export function AdminLogin() {
    const { initialize, accounts, selectAccount, signMessage } = useAuth();
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [step, setStep] = useState<'connect' | 'verify' | 'authenticate'>('connect');
    const [selectedAccountAddress, setSelectedAccountAddress] = useState<string | null>(null);
    const navigate = useNavigate();

    // Add function to check onchain for address 
    const ADMIN_ADDRESSES = [
        '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY', // Alice
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
        // Add more admin addresses
    ];

    const handleConnect = async () => {
        setLoading(true);
        setError(null);
        
        try {
        await initialize();
        setStep('verify');
        } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to connect wallet');
        } finally {
        setLoading(false);
        }
    };

    const handleSelectAccount = async (address: string) => {
        setSelectedAccountAddress(address);
        
        // Check if account is admin
        if (!ADMIN_ADDRESSES.includes(address)) {
        setError('This account is not authorized as an administrator');
        return;
        }

        const account = accounts.find((a) => a.address === address);
        if (account) {
        selectAccount(account);
        setStep('authenticate');
        }
    };

    const handleAuthenticate = async () => {
        if (!selectedAccountAddress) return;

        setLoading(true);
        setError(null);

        try {
        // Generate challenge message
        const challenge = `PortableID Admin Login\nTimestamp: ${Date.now()}\nNonce: ${Math.random().toString(36).substring(7)}`;
        
        // Sign the challenge
        const signature = await signMessage(challenge);
        
        // add functionality to verify the signature on-chain
        console.log('Admin authentication signature:', signature);
        
        // Store admin session
        localStorage.setItem('admin_token', signature);
        localStorage.setItem('admin_address', selectedAccountAddress);
        localStorage.setItem('admin_login_time', Date.now().toString());
        
        // Redirect to admin dashboard
        navigate('/admin/dashboard');
        } catch (err) {
        setError(err instanceof Error ? err.message : 'Authentication failed');
        } finally {
        setLoading(false);
        }
    };

    return (
        <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-4">
        <div className="max-w-md w-full space-y-8">
            {/* Header */}
            <div className="text-center">
            <div className="w-20 h-20 bg-gradient-to-br from-purple-500 to-pink-600 rounded-2xl flex items-center justify-center mx-auto mb-4 shadow-lg shadow-purple-500/50">
                <svg
                className="w-10 h-10 text-white"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                >
                <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                />
                </svg>
            </div>
            <h2 className="text-3xl font-bold text-white mb-2">Admin Portal</h2>
            <p className="text-slate-400">Secure administrator access</p>
            </div>

            {/* Main Content */}
            <div className="bg-slate-800/50 backdrop-blur border border-slate-700 rounded-2xl p-8 shadow-xl">
            {/* Error Message */}
            {error && (
                <div className="mb-6 p-4 bg-red-600/10 border border-red-500/50 rounded-lg">
                <div className="flex items-start gap-3">
                    <svg
                    className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    >
                    <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                    </svg>
                    <div className="flex-1">
                    <p className="text-sm text-red-300 font-medium">Authentication Error</p>
                    <p className="text-xs text-red-200/80 mt-1">{error}</p>
                    </div>
                </div>
                </div>
            )}

            {/* Step 1: Connect Wallet */}
            {step === 'connect' && (
                <div className="space-y-4">
                <button
                    onClick={handleConnect}
                    disabled={loading}
                    className="w-full px-6 py-4 bg-gradient-to-r from-purple-500 to-pink-600 hover:from-purple-600 hover:to-pink-700 text-white rounded-lg font-semibold transition disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-purple-500/30"
                >
                    {loading ? (
                    <span className="flex items-center justify-center gap-2">
                        <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                        Connecting...
                    </span>
                    ) : (
                    'Connect Admin Wallet'
                    )}
                </button>

                <div className="relative">
                    <div className="absolute inset-0 flex items-center">
                    <div className="w-full border-t border-slate-700"></div>
                    </div>
                    <div className="relative flex justify-center text-sm">
                    <span className="px-2 bg-slate-800/50 text-slate-500">Admin Access Only</span>
                    </div>
                </div>

                <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <h3 className="text-sm font-medium text-white mb-2">Requirements</h3>
                    <ul className="space-y-2 text-xs text-slate-400">
                    <li className="flex items-center gap-2">
                        <svg className="w-4 h-4 text-purple-400" fill="currentColor" viewBox="0 0 20 20">
                        <path
                            fillRule="evenodd"
                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clipRule="evenodd"
                        />
                        </svg>
                        Polkadot.js extension installed
                    </li>
                    <li className="flex items-center gap-2">
                        <svg className="w-4 h-4 text-purple-400" fill="currentColor" viewBox="0 0 20 20">
                        <path
                            fillRule="evenodd"
                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clipRule="evenodd"
                        />
                        </svg>
                        Authorized admin account
                    </li>
                    <li className="flex items-center gap-2">
                        <svg className="w-4 h-4 text-purple-400" fill="currentColor" viewBox="0 0 20 20">
                        <path
                            fillRule="evenodd"
                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clipRule="evenodd"
                        />
                        </svg>
                        Message signing capability
                    </li>
                    </ul>
                </div>
                </div>
            )}

            {/* Step 2: Select Account */}
            {step === 'verify' && accounts.length > 0 && (
                <div className="space-y-4">
                <div className="text-center mb-4">
                    <h3 className="text-lg font-semibold text-white mb-1">Select Admin Account</h3>
                    <p className="text-sm text-slate-400">Choose your authorized administrator account</p>
                </div>

                <div className="space-y-3 max-h-64 overflow-y-auto">
                    {accounts.map((account) => {
                    const isAdmin = ADMIN_ADDRESSES.includes(account.address);
                    return (
                        <button
                        key={account.address}
                        onClick={() => handleSelectAccount(account.address)}
                        disabled={!isAdmin}
                        className={`w-full p-4 rounded-lg text-left transition ${
                            isAdmin
                            ? 'bg-slate-700/50 hover:bg-slate-700 border border-slate-600 cursor-pointer'
                            : 'bg-slate-900/50 border border-slate-800 opacity-50 cursor-not-allowed'
                        }`}
                        >
                        <div className="flex items-center gap-3">
                            <div className={`w-10 h-10 rounded-full flex items-center justify-center ${
                            isAdmin ? 'bg-purple-600/20' : 'bg-slate-700'
                            }`}>
                            {isAdmin ? (
                                <svg className="w-5 h-5 text-purple-400" fill="currentColor" viewBox="0 0 20 20">
                                <path
                                    fillRule="evenodd"
                                    d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                                    clipRule="evenodd"
                                />
                                </svg>
                            ) : (
                                <svg className="w-5 h-5 text-slate-500" fill="currentColor" viewBox="0 0 20 20">
                                <path
                                    fillRule="evenodd"
                                    d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z"
                                    clipRule="evenodd"
                                />
                                </svg>
                            )}
                            </div>
                            <div className="flex-1 min-w-0">
                            <div className="font-medium text-white mb-1">
                                {account.meta.name || 'Account'}
                                {isAdmin && (
                                <span className="ml-2 px-2 py-0.5 bg-purple-600/30 text-purple-300 rounded text-xs font-medium">
                                    Admin
                                </span>
                                )}
                            </div>
                            <div className="text-xs font-mono text-slate-400 truncate">
                                {account.address}
                            </div>
                            </div>
                        </div>
                        </button>
                    );
                    })}
                </div>

                <button
                    onClick={() => setStep('connect')}
                    className="w-full px-4 py-2 bg-slate-700/50 hover:bg-slate-700 text-slate-300 rounded-lg text-sm transition"
                >
                    Back
                </button>
                </div>
            )}

            {/* Step 3: Authenticate */}
            {step === 'authenticate' && (
                <div className="space-y-4">
                <div className="text-center mb-4">
                    <div className="w-16 h-16 bg-purple-600/20 rounded-full flex items-center justify-center mx-auto mb-3">
                    <svg className="w-8 h-8 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                        />
                    </svg>
                    </div>
                    <h3 className="text-lg font-semibold text-white mb-1">Authenticate Access</h3>
                    <p className="text-sm text-slate-400">Sign the challenge to verify your identity</p>
                </div>

                <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <p className="text-xs text-slate-400 mb-2">Selected Account:</p>
                    <p className="text-sm font-mono text-white break-all">{selectedAccountAddress}</p>
                </div>

                <div className="flex gap-3">
                    <button
                    onClick={() => setStep('verify')}
                    disabled={loading}
                    className="flex-1 px-4 py-3 bg-slate-700/50 hover:bg-slate-700 text-white rounded-lg transition disabled:opacity-50"
                    >
                    Back
                    </button>
                    <button
                    onClick={handleAuthenticate}
                    disabled={loading}
                    className="flex-1 px-4 py-3 bg-gradient-to-r from-purple-500 to-pink-600 hover:from-purple-600 hover:to-pink-700 text-white rounded-lg font-semibold transition disabled:opacity-50 shadow-lg shadow-purple-500/30"
                    >
                    {loading ? (
                        <span className="flex items-center justify-center gap-2">
                        <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                        Signing...
                        </span>
                    ) : (
                        'Authenticate'
                    )}
                    </button>
                </div>
                </div>
            )}
            </div>

            {/* Footer */}
            <div className="text-center">
            <p className="text-sm text-slate-500 mb-2">
                Need access?{' '}
                <a href="#" className="text-purple-400 hover:text-purple-300 transition">
                Contact system administrator
                </a>
            </p>
            <p className="text-xs text-slate-600">
                Protected by cryptographic signatures • Zero-knowledge authentication
            </p>
            </div>
        </div>
        </div>
    );
}