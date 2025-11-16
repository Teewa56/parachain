import { StatsCard, CredentialCard } from '../../components/displays';

export function Dashboard({ credentials, proposals }: any) {
  const activeCount = credentials.filter((c: any) => c.status === 'Active').length;

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-white mb-8">Dashboard</h2>

      <div className="grid grid-cols-3 gap-6 mb-8">
        <StatsCard title="Total Issued" value={credentials.length} color="blue" />
        <StatsCard title="Active" value={activeCount} color="green" />
        <StatsCard title="Proposals" value={proposals.length} color="purple" />
      </div>

      <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Recent Credentials</h3>
        <div className="space-y-3">
          {credentials.slice(0, 5).map((cred: any) => (
            <CredentialCard key={cred.id} credential={cred} />
          ))}
        </div>
      </div>
    </div>
  );
}
