export function GovernancePanel({ proposals, onVote, loading }: any) {
  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-white mb-8">Governance Proposals</h2>
      <div className="space-y-4">
        {proposals.map((prop: any) => (
          <div
            key={prop.id}
            className="bg-slate-800/50 border border-slate-700 rounded-lg p-6"
          >
            <div className="flex items-center justify-between">
              <div>
                <div className="font-semibold text-white">
                  Proposal #{prop.id}: Add Trusted Issuer
                </div>
                <div className="text-sm text-slate-400 mt-1">
                  Votes: {prop.votes}
                </div>
              </div>
              <div className="text-right">
                <div
                  className={`inline-block px-3 py-1 rounded-full text-sm font-medium mb-2 ${
                    prop.status === 'Active'
                      ? 'bg-blue-600/30 text-blue-300'
                      : 'bg-green-600/30 text-green-300'
                  }`}
                >
                  {prop.status}
                </div>
              </div>
            </div>
            {prop.status === 'Active' && (
              <div className="flex gap-2 mt-4">
                <button
                  onClick={() => onVote(prop.id, 'Yes')}
                  disabled={loading}
                  className="px-4 py-2 bg-green-600/30 hover:bg-green-600/40 text-green-300 rounded-lg text-sm font-medium transition disabled:opacity-50"
                >
                  Vote Yes
                </button>
                <button
                  onClick={() => onVote(prop.id, 'No')}
                  disabled={loading}
                  className="px-4 py-2 bg-red-600/30 hover:bg-red-600/40 text-red-300 rounded-lg text-sm font-medium transition disabled:opacity-50"
                >
                  Vote No
                </button>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
