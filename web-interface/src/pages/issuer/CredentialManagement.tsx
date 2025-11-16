export function CredentialManagement({ credentials, onRevoke }: any) {
  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-white mb-8">Credentials</h2>
      <div className="bg-slate-800/50 border border-slate-700 rounded-lg overflow-hidden">
        <table className="w-full">
          <thead className="bg-slate-700/50 border-b border-slate-600">
            <tr>
              <th className="px-6 py-4 text-left text-sm font-semibold text-slate-300">
                Subject
              </th>
              <th className="px-6 py-4 text-left text-sm font-semibold text-slate-300">
                Type
              </th>
              <th className="px-6 py-4 text-left text-sm font-semibold text-slate-300">
                Issued
              </th>
              <th className="px-6 py-4 text-left text-sm font-semibold text-slate-300">
                Status
              </th>
              <th className="px-6 py-4 text-right text-sm font-semibold text-slate-300">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-700">
            {credentials.map((cred: any) => (
              <tr key={cred.id} className="hover:bg-slate-700/20 transition">
                <td className="px-6 py-4 text-white font-mono text-sm">{cred.subject}</td>
                <td className="px-6 py-4 text-slate-300">{cred.type}</td>
                <td className="px-6 py-4 text-slate-400">{cred.issued}</td>
                <td className="px-6 py-4">
                  <span
                    className={`px-3 py-1 rounded-full text-xs font-medium ${
                      cred.status === 'Active'
                        ? 'bg-green-600/30 text-green-300'
                        : 'bg-red-600/30 text-red-300'
                    }`}
                  >
                    {cred.status}
                  </span>
                </td>
                <td className="px-6 py-4 text-right">
                  {cred.status === 'Active' && (
                    <button
                      onClick={() => onRevoke(cred.id)}
                      className="text-red-400 hover:text-red-300 transition text-sm font-medium"
                    >
                      Revoke
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
