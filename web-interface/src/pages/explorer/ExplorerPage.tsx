import { useState } from "react";

export function ExplorerPage() {
    const [searchType, setSearchType] = useState<'identity' | 'credential'>('identity');
    const [searchQuery, setSearchQuery] = useState('');
    const [results, setResults] = useState<any[]>([]);

    const handleSearch = async () => {
        console.log('Searching:', searchType, searchQuery);
    };

    return (
        <div className="p-8 space-y-6">
        <div>
            <h2 className="text-3xl font-bold text-white mb-2">Explorer</h2>
            <p className="text-slate-400">Search identities and credentials</p>
        </div>

        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex gap-3 mb-4">
            <button
                onClick={() => setSearchType('identity')}
                className={`px-4 py-2 rounded-lg transition ${
                searchType === 'identity'
                    ? 'bg-blue-600 text-white'
                    : 'bg-slate-700/50 text-slate-300'
                }`}
            >
                Identity
            </button>
            <button
                onClick={() => setSearchType('credential')}
                className={`px-4 py-2 rounded-lg transition ${
                searchType === 'credential'
                    ? 'bg-blue-600 text-white'
                    : 'bg-slate-700/50 text-slate-300'
                }`}
            >
                Credential
            </button>
            </div>

            <div className="flex gap-3">
            <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder={`Enter ${searchType} ID...`}
                className="flex-1 bg-slate-700/50 border border-slate-600 rounded-lg px-4 py-3 text-white"
            />
            <button
                onClick={handleSearch}
                className="px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-lg font-semibold"
            >
                Search
            </button>
            </div>
        </div>

        {results.length > 0 && (
            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Results</h3>
            <div className="space-y-3">
                {results.map((result, idx) => (
                <div key={idx} className="p-4 bg-slate-700/30 rounded-lg">
                    <pre className="text-xs text-slate-300 overflow-auto">
                    {JSON.stringify(result, null, 2)}
                    </pre>
                </div>
                ))}
            </div>
            </div>
        )}
        </div>
    );
}