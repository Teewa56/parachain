export * from './credential';
export * from './governance';
export * from './api';

export interface Identity {
  controller: string;
  publicKey: string;
  createdAt: number;
  updatedAt: number;
  active: boolean;
}

export interface DidDocument {
  did: string;
  publicKeys: string[];
  authentication: string[];
  services: string[];
}
