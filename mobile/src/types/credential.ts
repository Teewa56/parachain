/**
 * Credential type enumeration
 */
export enum CredentialType {
    Education = 'Education',
    Health = 'Health',
    Employment = 'Employment',
    Age = 'Age',
    Address = 'Address',
    Custom = 'Custom',
}

/**
 * Credential status enumeration
 */
export enum CredentialStatus {
    Active = 'Active',
    Revoked = 'Revoked',
    Expired = 'Expired',
    Suspended = 'Suspended',
}

/**
 * Main Credential structure
 */
export interface Credential {
    subject: string;           // Subject DID hash
    issuer: string;            // Issuer DID hash
    credentialType: CredentialType;
    dataHash: string;          // Hash of credential data
    issuedAt: number;          // Unix timestamp
    expiresAt: number;         // Unix timestamp (0 = no expiry)
    status: CredentialStatus;
    signature: string;         // Issuer's signature
    metadataHash: string;      // Additional metadata hash
}

/**
 * Credential with metadata (for UI)
 */
export interface CredentialWithMetadata extends Credential {
    id: string;
    subjectName?: string;
    issuerName?: string;
    metadata?: CredentialMetadata;
    fields?: CredentialField[];
}

/**
 * Credential metadata
 */
export interface CredentialMetadata {
    version: string;
    schemaId?: string;
    schemaUrl?: string;
    icon?: string;
    color?: string;
    description?: string;
    tags?: string[];
}

/**
 * Credential field definition
 */
export interface CredentialField {
    index: number;
    name: string;
    value: any;
    type: FieldType;
    required: boolean;
    visible: boolean;
    encrypted: boolean;
}

/**
 * Field type enumeration
 */
export enum FieldType {
    String = 'string',
    Number = 'number',
    Date = 'date',
    Boolean = 'boolean',
    URL = 'url',
    Email = 'email',
    Phone = 'phone',
    Address = 'address',
    Custom = 'custom',
}

/**
 * Credential schema
 */
export interface CredentialSchema {
    id: string;
    name: string;
    version: string;
    credentialType: CredentialType;
    fields: SchemaField[];
    createdAt: number;
    updatedAt: number;
}

/**
 * Schema field definition
 */
export interface SchemaField {
    name: string;
    type: FieldType;
    required: boolean;
    description?: string;
    validation?: FieldValidation;
    defaultValue?: any;
}

/**
 * Field validation rules
 */
export interface FieldValidation {
    min?: number;
    max?: number;
    pattern?: string;
    enum?: any[];
    custom?: (value: any) => boolean;
}

/**
 * Credential issuance request
 */
export interface CredentialIssuanceRequest {
    subjectDid: string;
    credentialType: CredentialType;
    fields: Record<string, any>;
    expiresIn?: number;
    metadata?: Partial<CredentialMetadata>;
}

/**
 * Credential verification request
 */
export interface CredentialVerificationRequest {
    credentialId: string;
    fieldsToVerify?: number[];
    verifierDid?: string;
}

/**
 * Credential verification result
 */
export interface CredentialVerificationResult {
    valid: boolean;
    credentialId: string;
    status: CredentialStatus;
    issuerTrusted: boolean;
    notExpired: boolean;
    signatureValid: boolean;
    revoked: boolean;
    timestamp: number;
    error?: string;
}

/**
 * Selective disclosure request
 */
export interface SelectiveDisclosureRequest {
    credentialId: string;
    fieldsToReveal: number[];
    proofType: ProofType;
    verifierDid?: string;
}

/**
 * Proof type enumeration
 */
export enum ProofType {
    AgeAbove = 'AgeAbove',
    StudentStatus = 'StudentStatus',
    VaccinationStatus = 'VaccinationStatus',
    EmploymentStatus = 'EmploymentStatus',
    Custom = 'Custom',
}

/**
 * Zero-knowledge proof structure
 */
export interface ZkProof {
    proofType: ProofType;
    proofData: Uint8Array;
    publicInputs: Uint8Array[];
    credentialHash: string;
    createdAt: number;
    nonce: string;
}

/**
 * Proof verification result
 */
export interface ProofVerificationResult {
    valid: boolean;
    proofType: ProofType;
    credentialHash: string;
    verifiedAt: number;
    error?: string;
}

/**
 * Credential filter options
 */
export interface CredentialFilterOptions {
    type?: CredentialType;
    status?: CredentialStatus;
    issuer?: string;
    issuedAfter?: number;
    issuedBefore?: number;
    expiringWithinDays?: number;
    searchQuery?: string;
}

/**
 * Credential sort options
 */
export interface CredentialSortOptions {
    field: 'issuedAt' | 'expiresAt' | 'type' | 'status';
    order: 'asc' | 'desc';
}