import { useState } from "react";

interface AuditLogEntry {
  id: string;
  timestamp: number;
  action: string;
  actor: string;
  target: string;
  details: string;
  status: 'success' | 'failed';
}

export function AuditLog() {
  const [logs, setLogs] = useState<AuditLogEntry[]>([ ]);

  const [filter, setFilter] = useState('all');

  return (
    <div className="p-8 space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-white mb-2">Audit Log</h2>
        <p className="text-slate-400">System activity and security log</p>
      </div>

      <div className="flex gap-3">
        {['all', 'success', 'failed'].map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-4 py-2 rounded-lg transition ${
              filter === f
                ? 'bg-blue-600 text-white'
                : 'bg-slate-700/50 text-slate-300 hover:bg-slate-700'
            }`}
          >
            {f.charAt(0).toUpperCase() + f.slice(1)}
          </button>
        ))}
      </div>

      <div className="bg-slate-800/50 border border-slate-700 rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-slate-700/50 border-b border-slate-600">
            <tr>
              <th className="px-4 py-3 text-left font-semibold text-slate-300">Timestamp</th>
              <th className="px-4 py-3 text-left font-semibold text-slate-300">Action</th>
              <th className="px-4 py-3 text-left font-semibold text-slate-300">Actor</th>
              <th className="px-4 py-3 text-left font-semibold text-slate-300">Target</th>
              <th className="px-4 py-3 text-left font-semibold text-slate-300">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-700">
            {logs.map((log) => (
              <tr key={log.id} className="hover:bg-slate-700/20">
                <td className="px-4 py-3 text-slate-400">
                  {new Date(log.timestamp).toLocaleString()}
                </td>
                <td className="px-4 py-3 text-white font-medium">{log.action}</td>
                <td className="px-4 py-3 font-mono text-slate-300">{log.actor}</td>
                <td className="px-4 py-3 font-mono text-slate-300">{log.target}</td>
                <td className="px-4 py-3">
                  <span
                    className={`px-2 py-1 rounded-full text-xs ${
                      log.status === 'success'
                        ? 'bg-green-600/30 text-green-300'
                        : 'bg-red-600/30 text-red-300'
                    }`}
                  >
                    {log.status}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
