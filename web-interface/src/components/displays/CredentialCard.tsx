export function CredentialCard({ credential, onRevoke }: any) {
  return (
    <div className="flex items-center justify-between bg-slate-700/30 p-4 rounded-lg border border-slate-600/50 hover:border-slate-500/70 transition">
      <div>
        <div className="font-medium text-white">{credential.type}</div>
        <div className="text-sm text-slate-400">{credential.subject}</div>
      </div>
      <div className="text-right">
        <div className="text-sm text-slate-300">{credential.issued}</div>
        <div className="flex gap-2 mt-2">
          <span
            className={`px-2 py-1 rounded text-xs font-medium ${
              credential.status === 'Active'
                ? 'bg-green-600/30 text-green-300'
                : 'bg-red-600/30 text-red-300'
            }`}
          >
            {credential.status}
          </span>
          {credential.status === 'Active' && (
            <button
              onClick={() => onRevoke(credential.id)}
              className="px-2 py-1 bg-red-600/20 hover:bg-red-600/30 text-red-400 text-xs font-medium rounded transition"
            >
              Revoke
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
