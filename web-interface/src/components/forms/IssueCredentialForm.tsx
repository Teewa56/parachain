import { useState } from 'react';
import { CredentialType } from '../../types/credential';

interface IssueCredentialFormProps {
    onSubmit: (data: {
        subjectDid: string;
        credentialType: CredentialType;
        dataHash: string;
        expiresAt?: number;
    }) => void;
    loading?: boolean;
}

export function IssueCredentialForm({ onSubmit, loading }: IssueCredentialFormProps) {
    const [formData, setFormData] = useState({
        subjectDid: '',
        credentialType: CredentialType.Education,
        dataHash: '',
        expiresAt: '',
    });

    const [errors, setErrors] = useState<Record<string, string>>({});

    const validate = () => {
        const newErrors: Record<string, string> = {};

        if (!formData.subjectDid.match(/^did:/)) {
        newErrors.subjectDid = 'Must be a valid DID format (did:method:id)';
        }

        if (!formData.dataHash.match(/^0x[a-fA-F0-9]{64}$/)) {
        newErrors.dataHash = 'Must be a valid hex hash (0x followed by 64 hex chars)';
        }

        if (formData.expiresAt) {
        const expiry = new Date(formData.expiresAt).getTime();
        if (expiry <= Date.now()) {
            newErrors.expiresAt = 'Expiration date must be in the future';
        }
        }

        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (validate()) {
        onSubmit({
            subjectDid: formData.subjectDid,
            credentialType: formData.credentialType,
            dataHash: formData.dataHash,
            expiresAt: formData.expiresAt ? new Date(formData.expiresAt).getTime() : undefined,
        });
        setFormData({
            subjectDid: '',
            credentialType: CredentialType.Education,
            dataHash: '',
            expiresAt: '',
        });
        }
    };

    return (
        <form onSubmit={handleSubmit} className="space-y-6">
        <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
            Subject DID *
            </label>
            <input
            type="text"
            value={formData.subjectDid}
            onChange={(e) => setFormData({ ...formData, subjectDid: e.target.value })}
            placeholder="did:identity:123456"
            className={`w-full bg-slate-700/50 border ${errors.subjectDid ? 'border-red-500' : 'border-slate-600'} rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition`}
            />
            {errors.subjectDid && <p className="mt-1 text-sm text-red-400">{errors.subjectDid}</p>}
        </div>

        <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
            Credential Type *
            </label>
            <select
            value={formData.credentialType}
            onChange={(e) => setFormData({ ...formData, credentialType: e.target.value as CredentialType })}
            className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition"
            >
            {Object.values(CredentialType).map((type) => (
                <option key={type} value={type}>{type}</option>
            ))}
            </select>
        </div>

        <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
            Data Hash *
            </label>
            <input
            type="text"
            value={formData.dataHash}
            onChange={(e) => setFormData({ ...formData, dataHash: e.target.value })}
            placeholder="0x..."
            className={`w-full bg-slate-700/50 border ${errors.dataHash ? 'border-red-500' : 'border-slate-600'} rounded-lg px-4 py-3 text-white placeholder-slate-500 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition`}
            />
            {errors.dataHash && <p className="mt-1 text-sm text-red-400">{errors.dataHash}</p>}
            <p className="mt-1 text-xs text-slate-500">Hash of encrypted credential data</p>
        </div>

        <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
            Expiration Date (Optional)
            </label>
            <input
            type="datetime-local"
            value={formData.expiresAt}
            onChange={(e) => setFormData({ ...formData, expiresAt: e.target.value })}
            className={`w-full bg-slate-700/50 border ${errors.expiresAt ? 'border-red-500' : 'border-slate-600'} rounded-lg px-4 py-3 text-white focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition`}
            />
            {errors.expiresAt && <p className="mt-1 text-sm text-red-400">{errors.expiresAt}</p>}
        </div>

        <button
            type="submit"
            disabled={loading}
            className="w-full px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg font-semibold transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
            {loading ? 'Issuing Credential...' : 'Issue Credential'}
        </button>
        </form>
    );
}
