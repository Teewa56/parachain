export interface Credential {
  id: string;
  subject: string;
  issuer: string;
  type: CredentialType;
  dataHash: string;
  issuedAt: number;
  expiresAt: number;
  status: CredentialStatus;
  signature: string;
}

export enum CredentialType {
  Education = 'Education',
  Health = 'Health',
  Employment = 'Employment',
  Age = 'Age',
  Address = 'Address',
  Custom = 'Custom',
}

export enum CredentialStatus {
  Active = 'Active',
  Revoked = 'Revoked',
  Expired = 'Expired',
  Suspended = 'Suspended',
}
export interface CredentialSchema {
  schemaId: string;
  credentialType: CredentialType;
  fields: string[];
  requiredFields: boolean[];
  creator: string;
}
