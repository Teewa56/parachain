import type { NetworkConfig } from '../types/substrate';

// Network endpoints
export const NETWORKS: Record<string, NetworkConfig> = {
    local: {
        name: 'Local Development',
        endpoint: 'ws://127.0.0.1:9944',
        paraId: 1000,
    },
    testnet: {
        name: 'Rococo Testnet',
        endpoint: 'wss://rococo-rpc.polkadot.io',
        paraId: 1000,
    },
    production: {
        name: 'Production Parachain',
        endpoint: 'wss://identity-parachain.example.com',
        paraId: 2000,
    },
};

// Default network 
export const DEFAULT_NETWORK = __DEV__ ? 'local' : 'production';

// Connection settings
export const CONNECTION_CONFIG = {
    reconnectAttempts: 5,
    reconnectDelay: 3000, // 3 seconds
    timeout: 30000, // 30 seconds
};

// Transaction settings
export const TRANSACTION_CONFIG = {
    defaultTip: 0,
    mortalityPeriod: 64, // blocks
};

// Storage keys for caching
export const STORAGE_KEYS = {
    SELECTED_NETWORK: '@identity_wallet/selected_network',
    CACHED_METADATA: '@identity_wallet/cached_metadata',
    LAST_BLOCK_NUMBER: '@identity_wallet/last_block_number',
};

// Pallet names 
export const PALLETS = {
    IDENTITY_REGISTRY: 'identityRegistry',
    VERIFIABLE_CREDENTIALS: 'verifiableCredentials',
    ZK_CREDENTIALS: 'zkCredentials',
    CREDENTIAL_GOVERNANCE: 'credentialGovernance',
    XCM_CREDENTIALS: 'xcmCredentials',
};

// Extrinsic names
export const EXTRINSICS = {
    IDENTITY: {
        CREATE: 'createIdentity',
        UPDATE: 'updateIdentity',
        DEACTIVATE: 'deactivateIdentity',
        REACTIVATE: 'reactivateIdentity',
    },
    CREDENTIALS: {
        VERIFY: 'verifyCredential',
        SELECTIVE_DISCLOSURE: 'selectiveDisclosure',
    },
    ZK_PROOFS: {
        VERIFY_PROOF: 'verifyProof',
        CREATE_SCHEMA: 'createProofSchema',
    },
};

// Query methods
export const QUERIES = {
    IDENTITY: {
        IDENTITIES: 'identities',
        ACCOUNT_DIDS: 'accountDids',
        DID_DOCUMENTS: 'didDocuments',
    },
    CREDENTIALS: {
        CREDENTIALS: 'credentials',
        CREDENTIALS_OF: 'credentialsOf',
        ISSUED_BY: 'issuedBy',
        TRUSTED_ISSUERS: 'trustedIssuers',
    },
    ZK: {
        VERIFYING_KEYS: 'verifyingKeys',
        VERIFIED_PROOFS: 'verifiedProofs',
        PROOF_SCHEMAS: 'proofSchemas',
    },
};

// Type registry for custom types
export const CUSTOM_TYPES = {
    Identity: {
        controller: 'AccountId',
        publicKey: 'H256',
        createdAt: 'u64',
        updatedAt: 'u64',
        active: 'bool',
    },
    Credential: {
        subject: 'H256',
        issuer: 'H256',
        credentialType: 'CredentialType',
        dataHash: 'H256',
        issuedAt: 'u64',
        expiresAt: 'u64',
        status: 'CredentialStatus',
        signature: 'H256',
        metadataHash: 'H256',
    },
    CredentialType: {
        _enum: ['Education', 'Health', 'Employment', 'Age', 'Address', 'Custom'],
    },
    CredentialStatus: {
        _enum: ['Active', 'Revoked', 'Expired', 'Suspended'],
    },
    ZkProof: {
        proofType: 'ProofType',
        proofData: 'Vec<u8>',
        publicInputs: 'Vec<Vec<u8>>',
        credentialHash: 'H256',
        createdAt: 'u64',
        nonce: 'H256',
    },
    ProofType: {
        _enum: ['AgeAbove', 'StudentStatus', 'VaccinationStatus', 'EmploymentStatus', 'Custom'],
    },
};