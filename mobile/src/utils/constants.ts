// Application constants

// Storage keys
export const STORAGE_KEYS = {
    // Authentication
    IS_AUTHENTICATED: '@identity_wallet/is_authenticated',
    BIOMETRIC_ENABLED: '@identity_wallet/biometric_enabled',
    HAS_COMPLETED_ONBOARDING: '@identity_wallet/has_onboarding',
    LAST_LOGIN: '@identity_wallet/last_login',

    // Identity
    CURRENT_DID: '@identity_wallet/current_did',
    DID_HASH: '@identity_wallet/did_hash',
    IDENTITY_CREATED_AT: '@identity_wallet/identity_created_at',

    // Network
    SELECTED_NETWORK: '@identity_wallet/selected_network',
    CUSTOM_ENDPOINT: '@identity_wallet/custom_endpoint',

    // Preferences
    THEME: '@identity_wallet/theme',
    LANGUAGE: '@identity_wallet/language',
    NOTIFICATIONS_ENABLED: '@identity_wallet/notifications',

    // Cache
    CACHED_CREDENTIALS: '@identity_wallet/cached_credentials',
    CACHED_PROOFS: '@identity_wallet/cached_proofs',
    LAST_SYNC: '@identity_wallet/last_sync',
};

// Time constants (in milliseconds)
export const TIME = {
    SECOND: 1000,
    MINUTE: 60 * 1000,
    HOUR: 60 * 60 * 1000,
    DAY: 24 * 60 * 60 * 1000,
    WEEK: 7 * 24 * 60 * 60 * 1000,
};

// Blockchain constants
export const BLOCKCHAIN = {
    BLOCK_TIME: 6000, // 6 seconds per block
    DECIMALS: 12, // Token decimals
    SS58_FORMAT: 42, // Substrate default
    EXISTENTIAL_DEPOSIT: '10000000000', // 0.01 UNIT
};

// Transaction constants
export const TRANSACTION = {
    DEFAULT_TIP: '0',
    MAX_WAIT_TIME: 30000, // 30 seconds
    RETRY_ATTEMPTS: 3,
    RETRY_DELAY: 2000, // 2 seconds
};

// Proof constants
export const PROOF = {
  MAX_FIELDS_TO_REVEAL: 50,
  PROOF_VALIDITY_PERIOD: 3600, // 1 hour in seconds
  MAX_PROOF_AGE: 86400, // 24 hours in seconds
};

// Credential constants
export const CREDENTIAL = {
    EXPIRY_WARNING_DAYS: 30, // Warn when credential expires in 30 days
    MAX_CREDENTIALS_PER_FETCH: 20,
    CACHE_DURATION: 5 * TIME.MINUTE,
};

// UI constants
export const UI = {
    TOAST_DURATION: 3000, // 3 seconds
    LOADING_DELAY: 300, // Show loading after 300ms
    DEBOUNCE_DELAY: 500, // 500ms debounce for search
    ANIMATION_DURATION: 300,
    QR_CODE_SIZE: 280,
    QR_CODE_TIMEOUT: 120000, // 2 minutes
};

// Validation rules
export const VALIDATION = {
    MIN_PIN_LENGTH: 6,
    MAX_PIN_LENGTH: 8,
    MIN_PASSWORD_LENGTH: 8,
    MAX_DID_LENGTH: 255,
    MIN_DID_LENGTH: 7,
    MNEMONIC_12_WORDS: 12,
    MNEMONIC_24_WORDS: 24,
};

// Error messages
export const ERROR_MESSAGES = {
    // Network errors
    NETWORK_ERROR: 'Network connection failed. Please check your internet connection.',
    API_CONNECTION_FAILED: 'Failed to connect to the blockchain. Please try again.',
    TRANSACTION_FAILED: 'Transaction failed. Please try again.',
    TIMEOUT: 'Request timed out. Please try again.',

    // Authentication errors
    BIOMETRIC_FAILED: 'Biometric authentication failed.',
    BIOMETRIC_NOT_AVAILABLE: 'Biometric authentication is not available on this device.',
    PIN_INCORRECT: 'Incorrect PIN. Please try again.',
    AUTHENTICATION_REQUIRED: 'Authentication required.',

    // Identity errors
    IDENTITY_NOT_FOUND: 'Identity not found.',
    IDENTITY_ALREADY_EXISTS: 'Identity already exists.',
    INVALID_DID_FORMAT: 'Invalid DID format.',
    IDENTITY_CREATION_FAILED: 'Failed to create identity.',

    // Credential errors
    CREDENTIAL_NOT_FOUND: 'Credential not found.',
    CREDENTIAL_EXPIRED: 'This credential has expired.',
    CREDENTIAL_REVOKED: 'This credential has been revoked.',
    ISSUER_NOT_TRUSTED: 'The issuer is not trusted.',

    // Proof errors
    PROOF_GENERATION_FAILED: 'Failed to generate proof.',
    PROOF_VERIFICATION_FAILED: 'Proof verification failed.',
    INVALID_PROOF: 'Invalid proof data.',

    // Storage errors
    STORAGE_ERROR: 'Failed to access secure storage.',
    KEYPAIR_NOT_FOUND: 'Keypair not found. Please create or import an identity.',

    // Generic errors
    UNKNOWN_ERROR: 'An unknown error occurred.',
    INVALID_INPUT: 'Invalid input. Please check your data.',
    PERMISSION_DENIED: 'Permission denied.',
};

// Success messages
export const SUCCESS_MESSAGES = {
    IDENTITY_CREATED: 'Identity created successfully!',
    IDENTITY_UPDATED: 'Identity updated successfully!',
    CREDENTIAL_VERIFIED: 'Credential verified successfully!',
    PROOF_GENERATED: 'Proof generated successfully!',
    BACKUP_COMPLETED: 'Backup completed successfully!',
    SETTINGS_SAVED: 'Settings saved successfully!',
};

// DID Methods
export const DID_METHODS = {
    IDENTITY: 'identity',
    POLKADOT: 'polkadot',
    SUBSTRATE: 'substrate',
};

// Credential types
export const CREDENTIAL_TYPES = {
    EDUCATION: 'Education',
    HEALTH: 'Health',
    EMPLOYMENT: 'Employment',
    AGE: 'Age',
    ADDRESS: 'Address',
    CUSTOM: 'Custom',
};

// Credential status
export const CREDENTIAL_STATUS = {
    ACTIVE: 'Active',
    REVOKED: 'Revoked',
    EXPIRED: 'Expired',
    SUSPENDED: 'Suspended',
};

// Proof types
export const PROOF_TYPES = {
    AGE_ABOVE: 'AgeAbove',
    STUDENT_STATUS: 'StudentStatus',
    VACCINATION_STATUS: 'VaccinationStatus',
    EMPLOYMENT_STATUS: 'EmploymentStatus',
    CUSTOM: 'Custom',
};

// Navigation routes
export const ROUTES = {
    // Auth
    LOGIN: '/(auth)/login',
    REGISTER: '/(auth)/register',
    RECOVERY: '/(auth)/recovery',

    // Wallet
    DASHBOARD: '/(wallet)',
    IDENTITY_LIST: '/(wallet)/identity',
    IDENTITY_DETAILS: '/(wallet)/identity/[id]',
    IDENTITY_CREATE: '/(wallet)/identity/create',
    IDENTITY_MANAGE: '/(wallet)/identity/manage',

    CREDENTIALS_LIST: '/(wallet)/credentials',
    CREDENTIAL_DETAILS: '/(wallet)/credentials/[id]',
    CREDENTIAL_SHARE: '/(wallet)/credentials/share',
    CREDENTIAL_QR: '/(wallet)/credentials/qr',

    PROOF_GENERATE: '/(wallet)/proof',
    PROOF_CONFIRM: '/(wallet)/proof/confirm',
    PROOF_HISTORY: '/(wallet)/proof/history',

    SETTINGS: '/(wallet)/settings',
    SETTINGS_BIOMETRIC: '/(wallet)/settings/biometric',
    SETTINGS_BACKUP: '/(wallet)/settings/backup',
};

// Regular expressions
export const REGEX = {
    DID: /^did:[a-z0-9]+:[a-z0-9\-_]+$/i,
    HEX: /^0x[0-9a-fA-F]+$/,
    NUMERIC: /^\d+$/,
    ALPHANUMERIC: /^[a-zA-Z0-9]+$/,
};

// API endpoints 
export const API_ENDPOINTS = {
    IPFS_GATEWAY: 'https://ipfs.io/ipfs/',
    BACKUP_SERVICE: 'https://backup.identity-wallet.com/api',
};

// App limits
export const LIMITS = {
    MAX_CREDENTIALS: 1000,
    MAX_PROOFS_HISTORY: 100,
    MAX_IDENTITIES: 5, // for multi identity support
    MAX_RETRY_ATTEMPTS: 3,
};