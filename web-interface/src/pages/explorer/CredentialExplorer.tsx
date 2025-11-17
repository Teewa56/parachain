import { useState } from "react";

export function CredentialExplorer() {
    const [credentials, setCredentials] = useState<any[]>([]);

    return (
        <div className="p-8">
        <h2 className="text-3xl font-bold text-white mb-6">Credential Explorer</h2>
        <div className="grid grid-cols-1 gap-4">
            {credentials.map((cred) => (
            <div key={cred.id} className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
                <div className="font-mono text-sm text-slate-300">{cred.id}</div>
            </div>
            ))}
        </div>
        </div>
    );
}
