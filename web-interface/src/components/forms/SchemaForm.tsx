import { useState } from "react";
import { CredentialType } from "../../types";

export function SchemaForm({ onSubmit, loading }: any) {
    const [fields, setFields] = useState([{ name: '', required: true }]);
    const [credentialType, setCredentialType] = useState(CredentialType.Education);

    const addField = () => {
        setFields([...fields, { name: '', required: false }]);
    };

    const removeField = (index: number) => {
        setFields(fields.filter((_, i) => i !== index));
    };

    const updateField = (index: number, key: 'name' | 'required', value: any) => {
        const updated = [...fields];
        updated[index][key] = value;
        setFields(updated);
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        onSubmit({
        credentialType,
        fields: fields.map(f => f.name),
        requiredFields: fields.map(f => f.required),
        });
    };

    return (
        <form onSubmit={handleSubmit} className="space-y-6">
        <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">Credential Type</label>
            <select
            value={credentialType}
            onChange={(e) => setCredentialType(e.target.value as CredentialType)}
            className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white"
            >
            {Object.values(CredentialType).map(type => (
                <option key={type} value={type}>{type}</option>
            ))}
            </select>
        </div>

        <div>
            <label className="block text-sm font-medium text-slate-300 mb-3">Schema Fields</label>
            <div className="space-y-3">
            {fields.map((field, index) => (
                <div key={index} className="flex gap-3">
                <input
                    type="text"
                    value={field.name}
                    onChange={(e) => updateField(index, 'name', e.target.value)}
                    placeholder="Field name"
                    className="flex-1 bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-2 text-white"
                    required
                />
                <label className="flex items-center gap-2 px-4 bg-slate-700/50 border border-slate-600 rounded-lg">
                    <input
                    type="checkbox"
                    checked={field.required}
                    onChange={(e) => updateField(index, 'required', e.target.checked)}
                    />
                    <span className="text-sm text-slate-300">Required</span>
                </label>
                {fields.length > 1 && (
                    <button
                    type="button"
                    onClick={() => removeField(index)}
                    className="px-3 bg-red-600/20 text-red-400 rounded-lg hover:bg-red-600/30"
                    >
                    ✕
                    </button>
                )}
                </div>
            ))}
            </div>
            <button
            type="button"
            onClick={addField}
            className="mt-3 w-full px-4 py-2 bg-slate-700/50 border border-slate-600 rounded-lg text-slate-300 hover:bg-slate-700 transition"
            >
            + Add Field
            </button>
        </div>

        <button
            type="submit"
            disabled={loading}
            className="w-full px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-lg font-semibold"
        >
            {loading ? 'Creating...' : 'Create Schema'}
        </button>
        </form>
    );
}