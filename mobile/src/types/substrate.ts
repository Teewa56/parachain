import type { ApiPromise } from '@polkadot/api';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { Hash, BlockNumber } from '@polkadot/types/interfaces';

// Connection status
export enum ConnectionStatus {
  DISCONNECTED = 'disconnected',
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  ERROR = 'error',
}

// Network configuration
export interface NetworkConfig {
  name: string;
  endpoint: string;
  paraId: number;
}

// Identity types from pallet-identity-registry
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

// Credential types from pallet-verifiable-credentials
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

export interface Credential {
  subject: string;
  issuer: string;
  credentialType: CredentialType;
  dataHash: string;
  issuedAt: number;
  expiresAt: number;
  status: CredentialStatus;
  signature: string;
  metadataHash: string;
}

export enum ProofType {
  AgeAbove = 'AgeAbove',
  StudentStatus = 'StudentStatus',
  VaccinationStatus = 'VaccinationStatus',
  EmploymentStatus = 'EmploymentStatus',
  Custom = 'Custom',
}

export interface ZkProof {
  proofType: ProofType;
  proofData: Uint8Array;
  publicInputs: Uint8Array[];
  credentialHash: string;
  createdAt: number;
  nonce: string;
}

// Transaction types
export interface TransactionResult {
  success: boolean;
  hash?: Hash;
  blockNumber?: BlockNumber;
  error?: string;
}

export interface ExtrinsicStatus {
  isFinalized: boolean;
  isInBlock: boolean;
  hash: string;
  error?: string;
}

// API Service types
export interface SubstrateApiInterface {
  api: ApiPromise | null;
  isConnected: boolean;
  connectionStatus: ConnectionStatus;
  connect: (endpoint: string) => Promise<void>;
  disconnect: () => Promise<void>;
  getApi: () => ApiPromise;
}

// Account types
export interface Account {
  address: string;
  publicKey: string;
  name?: string;
}

export interface KeyPairInfo {
  pair: KeyringPair;
  mnemonic: string;
  address: string;
  publicKey: string;
}

// Query result types
export interface IdentityQueryResult {
  exists: boolean;
  identity?: Identity;
  didDocument?: DidDocument;
}

export interface CredentialQueryResult {
  exists: boolean;
  credential?: Credential;
}

// Event types from pallets
export interface IdentityCreatedEvent {
  didHash: string;
  controller: string;
}

export interface CredentialIssuedEvent {
  credentialId: string;
  subject: string;
  issuer: string;
  credentialType: CredentialType;
}

export interface ProofVerifiedEvent {
  proofHash: string;
  verifier: string;
  proofType: ProofType;
}

// Blockchain event listener types
export type EventCallback = (event: any) => void;

export interface Unsubscribe {
  (): void;
}