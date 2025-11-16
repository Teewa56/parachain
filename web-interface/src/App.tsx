import { useState, useEffect } from 'react';
import { Header } from './components/layout/Header';
import { Sidebar } from './components/layout/Sidebar';
import { Dashboard } from './pages/issuer/Dashboard';
import { IssuePage } from './pages/issuer/IssuePage';
import { CredentialManagement } from './pages/issuer/CredentialManagement';
import { GovernancePanel } from './pages/issuer/GovernancePanel';
import { useCredentials } from './hooks/useCredentials';
import { useGovernance } from './hooks/useGovernance';

export default function App() {
  const [currentPage, setCurrentPage] = useState('dashboard');
  const [account, setAccount] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const { credentials, issue, revoke } = useCredentials();
  const { proposals, vote } = useGovernance();

  useEffect(() => {
    // Mock data
    const mockCredentials = [
      { id: '1', subject: '0xabcd...1234', type: 'Education', issued: '2024-01-15', status: 'Active' },
      { id: '2', subject: '0xef01...5678', type: 'Health', issued: '2024-01-14', status: 'Active' },
      { id: '3', subject: '0x9012...3456', type: 'Employment', issued: '2024-01-10', status: 'Revoked' },
    ];
    // Simulate initial load
  }, []);

  const handleConnect = async () => {
    setLoading(true);
    await new Promise((r) => setTimeout(r, 1000));
    setAccount('0x1234...5678');
    setLoading(false);
  };

  const handleDisconnect = () => {
    setAccount(null);
    setCurrentPage('dashboard');
  };
  const handleIssue = async (subjectDid: string, type: string, dataHash: string) => {
    setLoading(true);
    await issue(subjectDid, type, dataHash);
    setLoading(false);
  };

  const handleRevoke = async (credentialId: string) => {
    setLoading(true);
    await revoke(credentialId);
    setLoading(false);
  };

  const handleVote = async (proposalId: number, voteType: string) => {
    setLoading(true);
    await vote(proposalId, voteType);
    setLoading(false);
  };

  const mockCredentials = [
    { id: '1', subject: '0xabcd...1234', type: 'Education', issued: '2024-01-15', status: 'Active' },
    { id: '2', subject: '0xef01...5678', type: 'Health', issued: '2024-01-14', status: 'Active' },
  ];

  const mockProposals = [
    { id: 1, status: 'Active', votes: '18/25' },
    { id: 2, status: 'Approved', votes: '22/25' },
  ];

  if (!account) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900">
        <Header account={null} onConnect={handleConnect} />
        <div className="flex items-center justify-center min-h-[calc(100vh-80px)]">
          <div className="text-center">
            <h2 className="text-4xl font-bold text-white mb-4">PortableID Issuer</h2>
            <p className="text-xl text-slate-400 mb-8">
              Connect wallet to issue verifiable credentials
            </p>
            <button
              onClick={handleConnect}
              disabled={loading}
              className="px-8 py-3 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg font-semibold transition"
            >
              {loading ? 'Connecting...' : 'Get Started'}
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900">
      <Header account={account} onDisconnect={handleDisconnect} />
      <div className="flex h-[calc(100vh-80px)]">
        <Sidebar currentPage={currentPage} onPageChange={setCurrentPage} />
        <main className="flex-1 overflow-auto">
          {currentPage === 'dashboard' && (
            <Dashboard credentials={mockCredentials} proposals={mockProposals} />
          )}
          {currentPage === 'issue' && (
            <IssuePage onIssue={handleIssue} loading={loading} />
          )}
          {currentPage === 'credentials' && (
            <CredentialManagement
              credentials={mockCredentials}
              onRevoke={handleRevoke}
            />
          )}
          {currentPage === 'governance' && (
            <GovernancePanel
              proposals={mockProposals}
              onVote={handleVote}
              loading={loading}
            />
          )}
          {currentPage === 'explorer' && (
            <div className="p-8">
              <h2 className="text-3xl font-bold text-white mb-8">Explorer</h2>
              <div className="grid grid-cols-2 gap-6">
                <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
                  <h3 className="text-lg font-semibold text-white mb-4">
                    Search Identities
                  </h3>
                  <input
                    type="text"
                    placeholder="Enter DID..."
                    className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:border-blue-500 outline-none mb-4"
                  />
                  <button className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition">
                    Search
                  </button>
                </div>
                <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
                  <h3 className="text-lg font-semibold text-white mb-4">
                    Search Credentials
                  </h3>
                  <input
                    type="text"
                    placeholder="Enter credential ID..."
                    className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:border-blue-500 outline-none mb-4"
                  />
                  <button className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition">
                    Search
                  </button>
                </div>
              </div>
            </div>
          )}
         </main> 
      </div>
    </div>
  );
}
