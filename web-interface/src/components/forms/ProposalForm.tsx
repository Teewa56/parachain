import { useState } from "react";
import { CredentialType } from "../../types";

export function ProposalForm({ onSubmit, loading }: any) {
    const [formData, setFormData] = useState({
        issuerDid: '',
        credentialTypes: [] as string[],
        description: '',
    });

    const credentialTypeOptions = Object.values(CredentialType);

    const handleTypeToggle = (type: string) => {
        setFormData(prev => ({
        ...prev,
        credentialTypes: prev.credentialTypes.includes(type)
            ? prev.credentialTypes.filter(t => t !== type)
            : [...prev.credentialTypes, type]
        }));
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        onSubmit(formData);
    };

    return (
        <form onSubmit={handleSubmit} className="space-y-6">
        <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">Issuer DID</label>
            <input
            type="text"
            value={formData.issuerDid}
            onChange={(e) => setFormData({ ...formData, issuerDid: e.target.value })}
            placeholder="did:identity:issuer123"
            className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:border-blue-500 outline-none"
            required
            />
        </div>

        <div>
            <label className="block text-sm font-medium text-slate-300 mb-3">
            Credential Types
            </label>
            <div className="grid grid-cols-2 gap-3">
            {credentialTypeOptions.map((type) => (
                <label
                key={type}
                className={`flex items-center gap-2 p-3 rounded-lg border cursor-pointer transition ${
                    formData.credentialTypes.includes(type)
                    ? 'bg-blue-600/20 border-blue-500'
                    : 'bg-slate-700/30 border-slate-600 hover:border-slate-500'
                }`}
                >
                <input
                    type="checkbox"
                    checked={formData.credentialTypes.includes(type)}
                    onChange={() => handleTypeToggle(type)}
                    className="w-4 h-4"
                />
                <span className="text-sm text-white">{type}</span>
                </label>
            ))}
            </div>
        </div>

        <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">Description</label>
            <textarea
            value={formData.description}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            placeholder="Describe why this issuer should be trusted..."
            rows={4}
            className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:border-blue-500 outline-none resize-none"
            required
            />
        </div>

        <button
            type="submit"
            disabled={loading}
            className="w-full px-6 py-3 bg-gradient-to-r from-purple-500 to-pink-600 hover:from-purple-600 hover:to-pink-700 text-white rounded-lg font-semibold transition disabled:opacity-50"
        >
            {loading ? 'Submitting...' : 'Submit Proposal'}
        </button>
        </form>
    );
}