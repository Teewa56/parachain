import { useState } from 'react';

interface CouncilMember {
    address: string;
    votingPower: number;
    joinedAt: number;
    totalVotes: number;
}

export function CouncilManagement() {
    const [members, setMembers] = useState<CouncilMember[]>([]);

    const [showAddForm, setShowAddForm] = useState(false);
    const [newMember, setNewMember] = useState({ address: '', votingPower: 10 });

    const handleAddMember = async () => {
        // Call blockchain to add member
        console.log('Adding member:', newMember);
        setShowAddForm(false);
        setNewMember({ address: '', votingPower: 10 });
    };

    const handleRemoveMember = async (address: string) => {
        if (confirm('Remove this council member?')) {
        console.log('Removing member:', address);
        }
    };

    return (
        <div className="p-8 space-y-6">
        <div className="flex items-center justify-between">
            <div>
            <h2 className="text-3xl font-bold text-white mb-2">Council Management</h2>
            <p className="text-slate-400">Manage governance council members</p>
            </div>
            <button
            onClick={() => setShowAddForm(!showAddForm)}
            className="px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg font-semibold transition"
            >
            + Add Member
            </button>
        </div>

        {showAddForm && (
            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Add Council Member</h3>
            <div className="space-y-4">
                <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">Address</label>
                <input
                    type="text"
                    value={newMember.address}
                    onChange={(e) => setNewMember({ ...newMember, address: e.target.value })}
                    placeholder="0x..."
                    className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white"
                />
                </div>
                <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">Voting Power</label>
                <input
                    type="number"
                    value={newMember.votingPower}
                    onChange={(e) => setNewMember({ ...newMember, votingPower: parseInt(e.target.value) })}
                    className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white"
                />
                </div>
                <div className="flex gap-3">
                <button onClick={handleAddMember} className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg">
                    Add Member
                </button>
                <button onClick={() => setShowAddForm(false)} className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg">
                    Cancel
                </button>
                </div>
            </div>
            </div>
        )}

        <div className="bg-slate-800/50 border border-slate-700 rounded-lg overflow-hidden">
            <table className="w-full">
            <thead className="bg-slate-700/50 border-b border-slate-600">
                <tr>
                <th className="px-6 py-4 text-left text-sm font-semibold text-slate-300">Address</th>
                <th className="px-6 py-4 text-left text-sm font-semibold text-slate-300">Voting Power</th>
                <th className="px-6 py-4 text-left text-sm font-semibold text-slate-300">Total Votes</th>
                <th className="px-6 py-4 text-left text-sm font-semibold text-slate-300">Joined</th>
                <th className="px-6 py-4 text-right text-sm font-semibold text-slate-300">Actions</th>
                </tr>
            </thead>
            <tbody className="divide-y divide-slate-700">
                {members.map((member) => (
                <tr key={member.address} className="hover:bg-slate-700/20">
                    <td className="px-6 py-4 font-mono text-sm text-white">{member.address}</td>
                    <td className="px-6 py-4 text-slate-300">{member.votingPower}</td>
                    <td className="px-6 py-4 text-slate-300">{member.totalVotes}</td>
                    <td className="px-6 py-4 text-slate-400 text-sm">
                    {new Date(member.joinedAt).toLocaleDateString()}
                    </td>
                    <td className="px-6 py-4 text-right">
                    <button
                        onClick={() => handleRemoveMember(member.address)}
                        className="text-red-400 hover:text-red-300 text-sm font-medium"
                    >
                        Remove
                    </button>
                    </td>
                </tr>
                ))}
            </tbody>
            </table>
        </div>
        </div>
    );
}