export interface ProverRequest {
  circuit_id: string;
  public_inputs: string[];
  private_inputs_b64: string;
  options?: Record<string, any>;
}

export interface ProverResponse {
  ok: boolean;
  proof_base64?: string;
  public_inputs?: string[];
  error?: string;
}