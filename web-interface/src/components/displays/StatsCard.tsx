export function StatsCard({ title, value, color }: any) {
  const colorClass = {
    blue: 'from-blue-600/20 to-blue-900/20 border-blue-500/30',
    green: 'from-green-600/20 to-green-900/20 border-green-500/30',
    purple: 'from-purple-600/20 to-purple-900/20 border-purple-500/30',
  }[color] || 'from-slate-600/20 to-slate-900/20 border-slate-500/30';

  return (
    <div className={`bg-gradient-to-br ${colorClass} border rounded-lg p-6`}>
      <div className="text-sm text-slate-400 mb-2">{title}</div>
      <div className="text-3xl font-bold text-white">{value}</div>
    </div>
  );
}
