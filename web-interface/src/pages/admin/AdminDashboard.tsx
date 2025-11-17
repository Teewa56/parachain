import { useState, useEffect } from 'react';
import { StatsCard } from '../../components/displays/StatsCard';
import { TimelineComponent } from '../../components/displays/TimelineComponent';

interface AdminStats {
  totalIdentities: number;
  totalCredentials: number;
  activeProposals: number;
  totalIssuers: number;
  councilMembers: number;
}

export function AdminDashboard() {
    const [stats, setStats] = useState<AdminStats>({
        totalIdentities: 0,
        totalCredentials: 0,
        activeProposals: 0,
        totalIssuers: 0,
        councilMembers: 0,
    });

    const [recentActivity, setRecentActivity] = useState<any[]>([]);

    useEffect(() => {
        // Fetch admin statistics
        fetchStats();
        fetchRecentActivity();
    }, []);

    const fetchStats = async () => {
        // Query blockchain for stats
        setStats({
        totalIdentities: 1247,
        totalCredentials: 3589,
        activeProposals: 5,
        totalIssuers: 28,
        councilMembers: 7,
        });
    };

    const fetchRecentActivity = async () => {
        setRecentActivity([
        {
            id: '1',
            type: 'proposal' as const,
            title: 'New Issuer Proposal',
            description: 'Hospital XYZ proposed as trusted issuer',
            timestamp: Date.now() - 3600000,
            actor: '0x1234...5678',
        },
        {
            id: '2',
            type: 'issued' as const,
            title: 'Credential Issued',
            description: 'Education credential issued',
            timestamp: Date.now() - 7200000,
            actor: '0xabcd...efgh',
        },
        ]);
    };

    return (
        <div className="p-8 space-y-8">
        <div>
            <h2 className="text-3xl font-bold text-white mb-2">Admin Dashboard</h2>
            <p className="text-slate-400">System overview and management</p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-4">
            <StatsCard title="Total Identities" value={stats.totalIdentities} color="blue" />
            <StatsCard title="Total Credentials" value={stats.totalCredentials} color="green" />
            <StatsCard title="Active Proposals" value={stats.activeProposals} color="purple" />
            <StatsCard title="Trusted Issuers" value={stats.totalIssuers} color="blue" />
            <StatsCard title="Council Members" value={stats.councilMembers} color="green" />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Recent Activity</h3>
            <TimelineComponent events={recentActivity} maxEvents={5} />
            </div>

            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-white mb-4">System Health</h3>
            <div className="space-y-4">
                <div>
                <div className="flex justify-between text-sm mb-2">
                    <span className="text-slate-400">Blockchain Sync</span>
                    <span className="text-green-400">100%</span>
                </div>
                <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                    <div className="h-full bg-green-500" style={{ width: '100%' }} />
                </div>
                </div>
                <div>
                <div className="flex justify-between text-sm mb-2">
                    <span className="text-slate-400">Storage Usage</span>
                    <span className="text-yellow-400">67%</span>
                </div>
                <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                    <div className="h-full bg-yellow-500" style={{ width: '67%' }} />
                </div>
                </div>
                <div>
                <div className="flex justify-between text-sm mb-2">
                    <span className="text-slate-400">Network Load</span>
                    <span className="text-blue-400">42%</span>
                </div>
                <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                    <div className="h-full bg-blue-500" style={{ width: '42%' }} />
                </div>
                </div>
            </div>
            </div>
        </div>
        </div>
    );
}