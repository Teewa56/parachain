import { useState } from 'react';
import * as base64 from 'base-64';
import { generateProof } from './prover';
import type { ProverRequest, ProverResponse } from './types';

export function useProver() {
    const [busy, setBusy] = useState(false);
    const [lastError, setLastError] = useState<string | null>(null);

    async function runProof(circuitId: string, publicInputs: string[], privateInputs: Uint8Array, options?: any): Promise<ProverResponse> {
        setBusy(true);
        setLastError(null);

        // Convert private inputs to base64 to pass into native safely
        const private_b64 = base64.encode(String.fromCharCode(...privateInputs));

        const req: ProverRequest = {
            circuit_id: circuitId,
            public_inputs: publicInputs,
            private_inputs_b64: private_b64,
            options,
        };

        try {
            const resp = await generateProof(req);
            if (!resp.ok) {
                setLastError(resp.error ?? 'unknown prover error');
            }
            return resp;
        } finally {
            setBusy(false);
        }
    }

    return { runProof, busy, lastError };
}
