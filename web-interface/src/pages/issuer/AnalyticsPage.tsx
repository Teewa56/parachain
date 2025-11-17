export function AnalyticsPage() {
  return (
    <div className="p-8 space-y-6">
      <h2 className="text-3xl font-bold text-white mb-2">Analytics</h2>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatsCard title="Total Issued" value={1247} color="blue" />
        <StatsCard title="Active" value={1189} color="green" />
        <StatsCard title="Revoked" value={45} color="red" />
        <StatsCard title="Expired" value={13} color="yellow" />
      </div>

      <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Issuance Trend</h3>
        <div className="h-64 flex items-end justify-around gap-2">
          {[65, 59, 80, 81, 56, 72, 90].map((height, i) => (
            <div key={i} className="flex-1 bg-blue-500/30 rounded-t" style={{ height: `${height}%` }} />
          ))}
        </div>
      </div>
    </div>
  );
}