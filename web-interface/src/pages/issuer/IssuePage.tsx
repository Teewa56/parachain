import { useState } from 'react';
import { CredentialType } from '../../types/credential';

export function IssuePage({ onIssue, loading }: any) {
  const [formData, setFormData] = useState({
    subjectDid: '',
    credentialType: CredentialType.Education,
    dataHash: '',
  });

  const handleSubmit = () => {
    if (formData.subjectDid && formData.dataHash) {
      onIssue(
        formData.subjectDid,
        formData.credentialType,
        formData.dataHash
      );
      setFormData({ subjectDid: '', credentialType: CredentialType.Education, dataHash: '' });
    }
  };

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-white mb-8">Issue New Credential</h2>
      <div className="max-w-2xl bg-slate-800/50 border border-slate-700 rounded-lg p-8">
        <div className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Subject DID
            </label>
            <input
              type="text"
              placeholder="did:identity:subject"
              value={formData.subjectDid}
              onChange={(e) =>
                setFormData({ ...formData, subjectDid: e.target.value })
              }
              className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:border-blue-500 outline-none"
            />
            </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Credential Type
            </label>
            <select
              value={formData.credentialType}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  credentialType: e.target.value as CredentialType,
                })
              }
              className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-2 text-white focus:border-blue-500 outline-none"
            >
              {Object.values(CredentialType).map((type) => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Data Hash
            </label>
            <input
              type="text"
              placeholder="0x..."
              value={formData.dataHash}
              onChange={(e) =>
                setFormData({ ...formData, dataHash: e.target.value })
              }
              className="w-full bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:border-blue-500 outline-none font-mono text-sm"
            />
          </div>
          <button
            onClick={handleSubmit}
            disabled={loading}
            className="w-full px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg font-medium transition disabled:opacity-50"
          >
            {loading ? 'Issuing...' : 'Issue Credential'}
          </button>
        </div>
      </div>
    </div>
  );
}
