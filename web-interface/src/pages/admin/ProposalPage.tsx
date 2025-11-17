import { useState, useEffect } from 'react';
import { ProposalCard } from '../../components/displays/ProposalCard';
import { ProposalForm } from '../../components/forms/ProposalForm';
import { VotingForm } from '../../components/forms/VotingForm';
import { Proposal, ProposalStatus, ProposalType } from '../../types/governance';
import { CredentialType } from '../../types/credential';

export function ProposalPage() {
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [selectedProposal, setSelectedProposal] = useState<Proposal | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showVotingForm, setShowVotingForm] = useState(false);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState<'all' | ProposalStatus>('all');

  useEffect(() => {
    fetchProposals();
  }, []);

  const fetchProposals = async () => {
    setLoading(true);
    try {
      // Fetch from blockchain
      const mockProposals: Proposal[] = [
        {
          id: 1,
          proposer: '0x1234567890abcdef',
          issuerDid: '0xabcdef1234567890',
          proposalType: ProposalType.AddTrustedIssuer,
          credentialTypes: [CredentialType.Health, CredentialType.Education],
          description: 'Add City Hospital as a trusted issuer for health credentials',
          deposit: '100000000000',
          createdAt: Date.now() - 86400000 * 3,
          votingEndsAt: Date.now() + 86400000 * 4,
          status: ProposalStatus.Active,
          yesVotes: 18,
          noVotes: 5,
        },
        {
          id: 2,
          proposer: '0xfedcba0987654321',
          issuerDid: '0x9876543210fedcba',
          proposalType: ProposalType.AddTrustedIssuer,
          credentialTypes: [CredentialType.Employment],
          description: 'Add TechCorp as trusted issuer for employment verification',
          deposit: '100000000000',
          createdAt: Date.now() - 86400000 * 7,
          votingEndsAt: Date.now() - 86400000,
          status: ProposalStatus.Approved,
          yesVotes: 22,
          noVotes: 3,
        },
        {
          id: 3,
          proposer: '0xaabbccddee112233',
          issuerDid: '0x33221100eeddccbb',
          proposalType: ProposalType.RemoveTrustedIssuer,
          credentialTypes: [CredentialType.Health],
          description: 'Remove compromised issuer from trusted list',
          deposit: '100000000000',
          createdAt: Date.now() - 86400000 * 2,
          votingEndsAt: Date.now() + 86400000 * 5,
          status: ProposalStatus.Active,
          yesVotes: 15,
          noVotes: 8,
        },
      ];
      setProposals(mockProposals);
    } catch (error) {
      console.error('Failed to fetch proposals:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateProposal = async (formData: any) => {
    setLoading(true);
    try {
      // Submit to blockchain
      console.log('Creating proposal:', formData);
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      // Refresh proposals
      await fetchProposals();
      setShowCreateForm(false);
    } catch (error) {
      console.error('Failed to create proposal:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleVote = async (proposalId: number, vote: string) => {
    setLoading(true);
    try {
      // Submit vote to blockchain
      console.log('Voting:', proposalId, vote);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Refresh proposals
      await fetchProposals();
      setShowVotingForm(false);
      setSelectedProposal(null);
    } catch (error) {
      console.error('Failed to vote:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleFinalizeProposal = async (proposalId: number) => {
    if (!confirm('Finalize this proposal? This action cannot be undone.')) {
      return;
    }

    setLoading(true);
    try {
      // Submit finalization to blockchain
      console.log('Finalizing proposal:', proposalId);
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      // Refresh proposals
      await fetchProposals();
    } catch (error) {
      console.error('Failed to finalize proposal:', error);
    } finally {
      setLoading(false);
    }
  };

  const filteredProposals = proposals.filter((p) =>
    filter === 'all' ? true : p.status === filter
  );

  const stats = {
    total: proposals.length,
    active: proposals.filter((p) => p.status === ProposalStatus.Active).length,
    approved: proposals.filter((p) => p.status === ProposalStatus.Approved).length,
    rejected: proposals.filter((p) => p.status === ProposalStatus.Rejected).length,
  };

  return (
    <div className="p-8 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-white mb-2">Governance Proposals</h2>
          <p className="text-slate-400">
            Manage and vote on issuer authorization proposals
          </p>
        </div>
        <button
          onClick={() => setShowCreateForm(true)}
          className="px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg font-semibold transition"
        >
          + New Proposal
        </button>
      </div>

      {/* Statistics */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-4">
          <div className="text-sm text-slate-400 mb-1">Total Proposals</div>
          <div className="text-2xl font-bold text-white">{stats.total}</div>
        </div>
        <div className="bg-slate-800/50 border border-blue-500/30 rounded-lg p-4">
          <div className="text-sm text-slate-400 mb-1">Active</div>
          <div className="text-2xl font-bold text-blue-400">{stats.active}</div>
        </div>
        <div className="bg-slate-800/50 border border-green-500/30 rounded-lg p-4">
          <div className="text-sm text-slate-400 mb-1">Approved</div>
          <div className="text-2xl font-bold text-green-400">{stats.approved}</div>
        </div>
        <div className="bg-slate-800/50 border border-red-500/30 rounded-lg p-4">
          <div className="text-sm text-slate-400 mb-1">Rejected</div>
          <div className="text-2xl font-bold text-red-400">{stats.rejected}</div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-3 flex-wrap">
        <button
          onClick={() => setFilter('all')}
          className={`px-4 py-2 rounded-lg transition ${
            filter === 'all'
              ? 'bg-blue-600 text-white'
              : 'bg-slate-700/50 text-slate-300 hover:bg-slate-700'
          }`}
        >
          All
        </button>
        {Object.values(ProposalStatus).map((status) => (
          <button
            key={status}
            onClick={() => setFilter(status)}
            className={`px-4 py-2 rounded-lg transition ${
              filter === status
                ? 'bg-blue-600 text-white'
                : 'bg-slate-700/50 text-slate-300 hover:bg-slate-700'
            }`}
          >
            {status}
          </button>
        ))}
      </div>

      {/* Create Proposal Modal */}
      {showCreateForm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="relative bg-slate-800 border border-slate-700 rounded-xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto p-6">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-2xl font-bold text-white">Create New Proposal</h3>
              <button
                onClick={() => setShowCreateForm(false)}
                className="text-slate-400 hover:text-white transition"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <ProposalForm onSubmit={handleCreateProposal} loading={loading} />
          </div>
        </div>
      )}

      {/* Voting Modal */}
      {showVotingForm && selectedProposal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="relative bg-slate-800 border border-slate-700 rounded-xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto p-6">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-2xl font-bold text-white">Cast Your Vote</h3>
              <button
                onClick={() => {
                  setShowVotingForm(false);
                  setSelectedProposal(null);
                }}
                className="text-slate-400 hover:text-white transition"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <VotingForm
              proposal={selectedProposal}
              onVote={handleVote}
              onCancel={() => {
                setShowVotingForm(false);
                setSelectedProposal(null);
              }}
              loading={loading}
            />
          </div>
        </div>
      )}

      {/* Proposals List */}
      {loading && !proposals.length ? (
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
        </div>
      ) : filteredProposals.length === 0 ? (
        <div className="text-center py-12 bg-slate-800/30 border border-slate-700 rounded-lg">
          <div className="text-4xl mb-3">📋</div>
          <p className="text-slate-400">No {filter !== 'all' ? filter : ''} proposals found</p>
        </div>
      ) : (
        <div className="space-y-4">
          {filteredProposals.map((proposal) => (
            <div key={proposal.id} className="relative">
              <ProposalCard
                proposal={proposal}
                onVote={(id, vote) => {
                  setSelectedProposal(proposal);
                  setShowVotingForm(true);
                }}
                canVote={proposal.status === ProposalStatus.Active}
                loading={loading}
              />
              
              {/* Admin Actions */}
              {proposal.status === ProposalStatus.Active && (
                <div className="mt-3 flex gap-3">
                  <button
                    onClick={() => handleFinalizeProposal(proposal.id)}
                    disabled={loading}
                    className="px-4 py-2 bg-purple-600/20 hover:bg-purple-600/30 border border-purple-500/50 text-purple-300 rounded-lg text-sm font-medium transition disabled:opacity-50"
                  >
                    Finalize Early
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}