import { useState } from 'react';
import { Proposal, VoteType } from '../../types/governance';

interface VotingFormProps {
  proposal: Proposal;
  onVote: (proposalId: number, vote: VoteType) => Promise<void>;
  onCancel?: () => void;
  loading?: boolean;
}

export function VotingForm({ proposal, onVote, onCancel, loading = false }: VotingFormProps) {
  const [selectedVote, setSelectedVote] = useState<VoteType | null>(null);
  const [reasoning, setReasoning] = useState('');
  const [showConfirmation, setShowConfirmation] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedVote) return;
    
    setShowConfirmation(true);
  };

  const confirmVote = async () => {
    if (!selectedVote) return;
    
    try {
      await onVote(proposal.id, selectedVote);
      setShowConfirmation(false);
      setSelectedVote(null);
      setReasoning('');
    } catch (error) {
      console.error('Vote submission failed:', error);
    }
  };

  const totalVotes = proposal.yesVotes + proposal.noVotes;
  const yesPercentage = totalVotes > 0 ? (proposal.yesVotes / totalVotes) * 100 : 0;

  const voteOptions = [
    {
      type: VoteType.Yes,
      label: 'Vote Yes',
      icon: '✓',
      color: 'green',
      description: 'Support this proposal',
    },
    {
      type: VoteType.No,
      label: 'Vote No',
      icon: '✕',
      color: 'red',
      description: 'Reject this proposal',
    },
    {
      type: VoteType.Abstain,
      label: 'Abstain',
      icon: '⊝',
      color: 'gray',
      description: 'Neither support nor reject',
    },
  ];

  return (
    <div className="space-y-6">
      {/* Proposal Summary */}
      <div className="bg-slate-800/30 border border-slate-700 rounded-lg p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-lg font-semibold text-white">Proposal #{proposal.id}</h3>
          <span className="px-3 py-1 bg-blue-600/30 text-blue-300 rounded-full text-xs font-medium">
            {proposal.status}
          </span>
        </div>
        <p className="text-sm text-slate-400 mb-3">{proposal.description}</p>
        
        {/* Current Voting Status */}
        <div className="space-y-2">
          <div className="flex justify-between text-sm">
            <span className="text-slate-400">Current Status</span>
            <span className="text-white">{yesPercentage.toFixed(1)}% Yes</span>
          </div>
          <div className="h-2 bg-slate-700 rounded-full overflow-hidden flex">
            <div 
              className="bg-green-500 transition-all duration-300" 
              style={{ width: `${yesPercentage}%` }}
            />
            <div 
              className="bg-red-500 transition-all duration-300" 
              style={{ width: `${100 - yesPercentage}%` }}
            />
          </div>
          <div className="flex justify-between text-xs text-slate-500">
            <span>Yes: {proposal.yesVotes}</span>
            <span>No: {proposal.noVotes}</span>
          </div>
        </div>
      </div>

      {!showConfirmation ? (
        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Vote Options */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-3">
              Cast Your Vote
            </label>
            <div className="space-y-3">
              {voteOptions.map((option) => (
                <button
                  key={option.type}
                  type="button"
                  onClick={() => setSelectedVote(option.type)}
                  className={`w-full p-4 rounded-lg border-2 transition text-left ${
                    selectedVote === option.type
                      ? `border-${option.color}-500 bg-${option.color}-600/20`
                      : 'border-slate-600 bg-slate-700/30 hover:border-slate-500'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div className={`w-10 h-10 rounded-full flex items-center justify-center text-xl ${
                      selectedVote === option.type
                        ? `bg-${option.color}-500/30 text-${option.color}-300`
                        : 'bg-slate-700 text-slate-400'
                    }`}>
                      {option.icon}
                    </div>
                    <div className="flex-1">
                      <div className="font-semibold text-white mb-1">{option.label}</div>
                      <div className="text-sm text-slate-400">{option.description}</div>
                    </div>
                    {selectedVote === option.type && (
                      <div className="w-6 h-6 bg-blue-500 rounded-full flex items-center justify-center">
                        <span className="text-white text-sm">✓</span>
                      </div>
                    )}
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Reasoning (Optional) */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Reasoning (Optional)
            </label>
            <textarea
              value={reasoning}
              onChange={(e) => setReasoning(e.target.value)}
              placeholder="Explain your vote decision..."
              rows={3}
              className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none resize-none transition"
            />
            <p className="mt-1 text-xs text-slate-500">
              This will be recorded on-chain and visible to all council members
            </p>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-3">
            {onCancel && (
              <button
                type="button"
                onClick={onCancel}
                className="flex-1 px-6 py-3 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-semibold transition"
              >
                Cancel
              </button>
            )}
            <button
              type="submit"
              disabled={!selectedVote || loading}
              className="flex-1 px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg font-semibold transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Continue
            </button>
          </div>
        </form>
      ) : (
        <div className="space-y-6">
          {/* Confirmation Screen */}
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="text-center mb-6">
              <div className={`w-16 h-16 mx-auto mb-4 rounded-full flex items-center justify-center text-3xl ${
                selectedVote === VoteType.Yes ? 'bg-green-600/20 text-green-300' :
                selectedVote === VoteType.No ? 'bg-red-600/20 text-red-300' :
                'bg-gray-600/20 text-gray-300'
              }`}>
                {selectedVote === VoteType.Yes ? '✓' : selectedVote === VoteType.No ? '✕' : '⊝'}
              </div>
              <h3 className="text-xl font-bold text-white mb-2">Confirm Your Vote</h3>
              <p className="text-slate-400">
                You are about to vote <span className="font-semibold text-white">{selectedVote}</span> on Proposal #{proposal.id}
              </p>
            </div>

            {reasoning && (
              <div className="bg-slate-900/50 rounded-lg p-4 mb-4">
                <p className="text-sm text-slate-400 mb-1">Your reasoning:</p>
                <p className="text-white">{reasoning}</p>
              </div>
            )}

            <div className="bg-yellow-600/10 border border-yellow-600/30 rounded-lg p-4 mb-6">
              <div className="flex gap-3">
                <span className="text-yellow-400 text-xl">⚠️</span>
                <div className="flex-1">
                  <p className="text-sm text-yellow-300 font-medium mb-1">Important</p>
                  <p className="text-xs text-yellow-200/80">
                    This action cannot be undone. Your vote will be recorded on-chain and will be publicly visible.
                  </p>
                </div>
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => setShowConfirmation(false)}
                disabled={loading}
                className="flex-1 px-6 py-3 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-semibold transition disabled:opacity-50"
              >
                Go Back
              </button>
              <button
                onClick={confirmVote}
                disabled={loading}
                className={`flex-1 px-6 py-3 rounded-lg font-semibold transition disabled:opacity-50 disabled:cursor-not-allowed ${
                  selectedVote === VoteType.Yes
                    ? 'bg-green-600 hover:bg-green-700'
                    : selectedVote === VoteType.No
                    ? 'bg-red-600 hover:bg-red-700'
                    : 'bg-gray-600 hover:bg-gray-700'
                } text-white`}
              >
                {loading ? 'Submitting...' : `Confirm ${selectedVote}`}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Voting Power Info */}
      <div className="bg-slate-800/30 border border-slate-700 rounded-lg p-4">
        <div className="flex items-center justify-between text-sm">
          <span className="text-slate-400">Your Voting Power</span>
          <span className="text-white font-semibold">10 votes</span>
        </div>
      </div>
    </div>
  );
}