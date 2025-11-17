import { Proposal, ProposalStatus } from '../../types/governance';

interface ProposalCardProps {
    proposal: Proposal;
    onVote?: (proposalId: number, vote: 'Yes' | 'No' | 'Abstain') => void;
    canVote?: boolean;
    loading?: boolean;
}

export function ProposalCard({ proposal, onVote, canVote = false, loading = false }: ProposalCardProps) {
    const totalVotes = proposal.yesVotes + proposal.noVotes;
    const yesPercentage = totalVotes > 0 ? (proposal.yesVotes / totalVotes) * 100 : 0;
    const noPercentage = totalVotes > 0 ? (proposal.noVotes / totalVotes) * 100 : 0;

    const statusColors = {
        [ProposalStatus.Active]: 'bg-blue-600/30 text-blue-300 border-blue-500/50',
        [ProposalStatus.Approved]: 'bg-green-600/30 text-green-300 border-green-500/50',
        [ProposalStatus.Rejected]: 'bg-red-600/30 text-red-300 border-red-500/50',
        [ProposalStatus.Executed]: 'bg-purple-600/30 text-purple-300 border-purple-500/50',
        [ProposalStatus.Cancelled]: 'bg-gray-600/30 text-gray-300 border-gray-500/50',
    };

    const timeRemaining = proposal.votingEndsAt - Date.now();
    const daysRemaining = Math.max(0, Math.floor(timeRemaining / (1000 * 60 * 60 * 24)));
    const hoursRemaining = Math.max(0, Math.floor((timeRemaining % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)));

    return (
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6 hover:border-slate-600 transition">
        <div className="flex items-start justify-between mb-4">
            <div className="flex-1">
            <div className="flex items-center gap-3 mb-2">
                <h3 className="text-lg font-semibold text-white">
                Proposal #{proposal.id}
                </h3>
                <span className={`px-3 py-1 rounded-full text-xs font-medium border ${statusColors[proposal.status]}`}>
                {proposal.status}
                </span>
            </div>
            <p className="text-sm text-slate-400 mb-3">{proposal.description}</p>
            <div className="flex flex-wrap gap-2 mb-3">
                {proposal.credentialTypes.map((type, idx) => (
                <span key={idx} className="px-2 py-1 bg-slate-700/50 rounded text-xs text-slate-300">
                    {type}
                </span>
                ))}
            </div>
            <div className="text-xs text-slate-500">
                Proposer: <span className="font-mono text-slate-400">{proposal.proposer.slice(0, 10)}...{proposal.proposer.slice(-8)}</span>
            </div>
            </div>
        </div>

        {/* Voting Progress */}
        <div className="mb-4">
            <div className="flex justify-between text-sm mb-2">
            <span className="text-green-400">Yes: {proposal.yesVotes}</span>
            <span className="text-red-400">No: {proposal.noVotes}</span>
            </div>
            <div className="h-2 bg-slate-700 rounded-full overflow-hidden flex">
            <div 
                className="bg-green-500 transition-all duration-300" 
                style={{ width: `${yesPercentage}%` }}
            />
            <div 
                className="bg-red-500 transition-all duration-300" 
                style={{ width: `${noPercentage}%` }}
            />
            </div>
            <div className="flex justify-between text-xs text-slate-500 mt-1">
            <span>{yesPercentage.toFixed(1)}%</span>
            <span>{noPercentage.toFixed(1)}%</span>
            </div>
        </div>

        {/* Time Remaining */}
        {proposal.status === ProposalStatus.Active && timeRemaining > 0 && (
            <div className="text-xs text-slate-400 mb-4">
            ⏱️ {daysRemaining}d {hoursRemaining}h remaining
            </div>
        )}

        {/* Voting Buttons */}
        {canVote && proposal.status === ProposalStatus.Active && onVote && (
            <div className="flex gap-2">
            <button
                onClick={() => onVote(proposal.id, 'Yes')}
                disabled={loading}
                className="flex-1 px-4 py-2 bg-green-600/20 hover:bg-green-600/30 border border-green-500/50 text-green-300 rounded-lg text-sm font-medium transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
                Vote Yes
            </button>
            <button
                onClick={() => onVote(proposal.id, 'No')}
                disabled={loading}
                className="flex-1 px-4 py-2 bg-red-600/20 hover:bg-red-600/30 border border-red-500/50 text-red-300 rounded-lg text-sm font-medium transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
                Vote No
            </button>
            <button
                onClick={() => onVote(proposal.id, 'Abstain')}
                disabled={loading}
                className="px-4 py-2 bg-slate-700/30 hover:bg-slate-700/50 border border-slate-600 text-slate-400 rounded-lg text-sm font-medium transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
                Abstain
            </button>
            </div>
        )}
        </div>
    );
}