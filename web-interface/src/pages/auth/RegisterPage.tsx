import { useState } from "react";

export function RegisterPage() {
    const [formData, setFormData] = useState({
        did: '',
        name: '',
        email: '',
    });

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        console.log('Register:', formData);
    };

    return (
        <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-4">
        <div className="max-w-md w-full bg-slate-800/50 backdrop-blur border border-slate-700 rounded-2xl p-8">
            <h2 className="text-3xl font-bold text-white mb-6">Create Identity</h2>
            <form onSubmit={handleSubmit} className="space-y-6">
            <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">Name</label>
                <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white"
                required
                />
            </div>
            <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">Email</label>
                <input
                type="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white"
                required
                />
            </div>
            <button
                type="submit"
                className="w-full px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-lg font-semibold"
            >
                Create Identity
            </button>
            </form>
        </div>
        </div>
    );
}