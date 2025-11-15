/**
 * Identity structure from blockchain
 */
export interface Identity {
    controller: string;    // Account ID
    publicKey: string;     // Public key hex
    createdAt: number;     // Unix timestamp
    updatedAt: number;     // Unix timestamp
    active: boolean;       // Active status
}

/**
 * DID Document structure
 */
export interface DidDocument {
    did: string;
    publicKeys: string[];
    authentication: string[];
    services: string[];
}

/**
 * Extended identity with DID
 */
export interface IdentityWithDid extends Identity {
    did: string;
    didHash: string;
    didDocument?: DidDocument;
}

/**
 * Identity creation request
 */
export interface IdentityCreationRequest {
    mnemonic: string;
    name?: string;
    metadata?: IdentityMetadata;
}

/**
 * Identity metadata
 */
export interface IdentityMetadata {
    name?: string;
    avatar?: string;
    bio?: string;
    email?: string;
    website?: string;
    createdWith?: string;
    version?: string;
}

/**
 * Identity update request
 */
export interface IdentityUpdateRequest {
    publicKey?: string;
    metadata?: Partial<IdentityMetadata>;
}

/**
 * Account information
 */
export interface Account {
    address: string;
    publicKey: string;
    name?: string;
    balance?: AccountBalance;
}

/**
 * Account balance
 */
export interface AccountBalance {
    free: string;
    reserved: string;
    frozen: string;
    total: string;
    transferable: string;
}

/**
 * KeyPair information
 */
export interface KeyPairInfo {
    pair: any; // KeyringPair from @polkadot/keyring
    mnemonic: string;
    address: string;
    publicKey: string;
    meta?: KeyPairMeta;
}

/**
 * KeyPair metadata
 */
export interface KeyPairMeta {
    name?: string;
    createdAt?: number;
    lastUsed?: number;
    derivationPath?: string;
}

/**
 * DID method types
 */
export enum DidMethod {
    Identity = 'identity',
    Polkadot = 'polkadot',
    Substrate = 'substrate',
}

/**
 * DID resolution result
 */
export interface DidResolutionResult {
    did: string;
    didDocument?: DidDocument;
    identity?: Identity;
    metadata?: DidMetadata;
    exists: boolean;
}

/**
 * DID metadata
 */
export interface DidMetadata {
    created: number;
    updated: number;
    deactivated?: boolean;
    versionId?: string;
}

/**
 * Identity verification status
 */
export interface IdentityVerificationStatus {
    verified: boolean;
    level: VerificationLevel;
    verifiedBy?: string;
    verifiedAt?: number;
    expiresAt?: number;
}

/**
 * Verification level
 */
export enum VerificationLevel {
    None = 'none',
    Basic = 'basic',
    Enhanced = 'enhanced',
    Full = 'full',
}

/**
 * Identity query options
 */
export interface IdentityQueryOptions {
    includeMetadata?: boolean;
    includeCredentials?: boolean;
    includeDidDocument?: boolean;
}

/**
 * Identity query result
 */
export interface IdentityQueryResult {
    exists: boolean;
    identity?: Identity;
    didDocument?: DidDocument;
    metadata?: IdentityMetadata;
    credentials?: string[];
}

/**
 * Multi-identity support
 */
export interface IdentityProfile {
    id: string;
    did: string;
    name: string;
    isPrimary: boolean;
    createdAt: number;
    lastUsed: number;
    credentialCount: number;
}

/**
 * Authentication method
 */
export enum AuthenticationMethod {
    Biometric = 'biometric',
    Pin = 'pin',
    Password = 'password',
    Mnemonic = 'mnemonic',
}

/**
 * Authentication result
 */
export interface AuthenticationResult {
    success: boolean;
    method: AuthenticationMethod;
    timestamp: number;
    error?: string;
}

/**
 * Backup information
 */
export interface BackupInfo {
    hasBackup: boolean;
    lastBackup?: number;
    backupLocation?: BackupLocation;
    encrypted: boolean;
}

/**
 * Backup location
 */
export enum BackupLocation {
    Local = 'local',
    Cloud = 'cloud',
    External = 'external',
}

/**
 * Recovery options
 */
export interface RecoveryOptions {
    hasMnemonic: boolean;
    hasKeyFile: boolean;
    hasBiometric: boolean;
    hasPin: boolean;
}

/**
 * Identity lifecycle events
 */
export enum IdentityLifecycleEvent {
    Created = 'created',
    Updated = 'updated',
    Activated = 'activated',
    Deactivated = 'deactivated',
    KeyRotated = 'key_rotated',
    Backed_Up = 'backed_up',
    Restored = 'restored',
}

/**
 * Identity event log
 */
export interface IdentityEventLog {
    event: IdentityLifecycleEvent;
    timestamp: number;
    details?: any;
    blockNumber?: number;
    transactionHash?: string;
}