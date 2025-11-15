import { NativeModules, Platform } from 'react-native';
import { ProverRequest, ProverResponse } from './types';

const { ZKProverModule } = NativeModules;

export async function generateProof(request: ProverRequest): Promise<ProverResponse> {
  const inputJson = JSON.stringify(request);

  if (!ZKProverModule || typeof ZKProverModule.generateProof !== 'function') {
    return { ok: false, error: 'Native ZKProverModule not found. Ensure you built custom dev client.' };
  }

  try {
    const resJson: string = await ZKProverModule.generateProof(inputJson);
    const parsed: ProverResponse = JSON.parse(resJson);
    return parsed;
  } catch (e: any) {
    return { ok: false, error: e?.message ?? String(e) };
  }
}
