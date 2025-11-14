# Mobile App Development Plan - Identity Parachain Wallet

## Executive Summary

A production-ready React Native mobile wallet built with Expo Router that enables users to:
- Create and manage self-sovereign decentralized identities (DIDs)
- Store and share verifiable credentials privately
- Generate zero-knowledge proofs for selective disclosure
- Interact with the Identity Parachain via Polkadot.js API
- Secure key management with biometric authentication

---

## App Architecture Overview

```
┌───────────────────────────────────────────────────────────┐
│                    MOBILE WALLET APP                      │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Identity   │  │ Credentials  │  │  ZK Proofs   │     │
│  │  Management  │  │   Storage    │  │  Generation  │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                           │
│  ┌──────────────────────────────────────────────────────┐ │
│  │         Polkadot.js API (WebSocket)                  │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Secure     │  │  Biometric   │  │     QR       │     │
│  │   Storage    │  │    Auth      │  │  Code Share  │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                           │
└───────────────────────────────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │  Identity Parachain    │
              │  (WebSocket RPC)       │
              └────────────────────────┘
```

---

## Core Features & User Flows

### 1. **Onboarding Flow**
```
First Launch → Welcome Screen → Create/Import Identity
→ Set Biometric Lock → Security Backup → Dashboard
```

**Screens:**
- Welcome with app introduction
- Identity creation (generate keypair)
- Biometric setup (Face ID/Touch ID)
- Seed phrase backup (12/24 words)
- PIN setup as fallback

### 2. **Identity Management Flow**
```
Dashboard → Identity Tab → View DID Details
→ Update Public Key → Deactivate/Reactivate
```

**Features:**
- Display DID in QR code format
- Show identity status (Active/Inactive)
- View public key
- Rotate keys securely
- Export DID for sharing

### 3. **Credential Management Flow**
```
Dashboard → Credentials Tab → View All Credentials
→ Select Credential → View Details → Share/Generate Proof
```

**Features:**
- List all received credentials
- Filter by type (Education, Health, Employment)
- View issuer information
- Check expiration dates
- View revocation status

### 4. **Zero-Knowledge Proof Flow**
```
Credentials → Select Credential → Generate Proof
→ Choose Fields to Reveal → Create ZK Proof → Share QR
```

**Features:**
- Field selector UI (checkboxes)
- Real-time proof preview
- Generate ZK proof locally
- Display as QR code
- Share proof to verifier

### 5. **QR Code Sharing Flow**
```
Proof/Credential → Generate QR → Display → Scan by Verifier
→ Verify On-Chain → Result
```

**Features:**
- Generate QR codes for credentials
- Scan QR codes from issuers
- Display verification status
- History of shared proofs

---

## Technical Stack

### Core Technologies
- **Framework**: React Native 0.81.5 + Expo SDK 54
- **Routing**: Expo Router 6.0 (file-based routing)
- **Language**: TypeScript 5.9
- **State Management**: Zustand (lightweight, performant)
- **Blockchain API**: @polkadot/api 10.11.1
- **Cryptography**: @polkadot/util-crypto 12.5.1

### Key Dependencies
```json
{
  "@polkadot/api": "^10.11.1",
  "@polkadot/keyring": "^12.5.1",
  "@polkadot/util-crypto": "^12.5.1",
  "expo-secure-store": "~13.0.1",
  "expo-local-authentication": "~14.0.1",
  "expo-camera": "~15.0.14",
  "react-native-qrcode-svg": "^6.3.0",
  "zustand": "^4.5.0"
}
```

### Security Features
- **Encrypted Storage**: expo-secure-store (hardware-backed)
- **Biometric Auth**: expo-local-authentication
- **Key Derivation**: BIP39/BIP44 for seed phrases
- **Secure Enclave**: iOS Keychain, Android Keystore

---

## State Management Architecture

### Zustand Stores

#### 1. **authStore.ts**
```typescript
interface AuthState {
  isAuthenticated: boolean;
  biometricEnabled: boolean;
  hasSeedPhrase: boolean;
  login: () => Promise<void>;
  logout: () => void;
  enableBiometric: () => Promise<void>;
}
```

#### 2. **identityStore.ts**
```typescript
interface IdentityState {
  did: string | null;
  publicKey: string | null;
  isActive: boolean;
  createdAt: number;
  createIdentity: (seed: string) => Promise<void>;
  updateIdentity: (newKey: string) => Promise<void>;
  deactivateIdentity: () => Promise<void>;
}
```

#### 3. **credentialStore.ts**
```typescript
interface CredentialState {
  credentials: Credential[];
  fetchCredentials: () => Promise<void>;
  getCredentialById: (id: string) => Credential | null;
  refreshCredential: (id: string) => Promise<void>;
}
```

#### 4. **uiStore.ts**
```typescript
interface UIState {
  isLoading: boolean;
  error: string | null;
  toast: Toast | null;
  setLoading: (loading: boolean) => void;
  showToast: (message: string) => void;
}
```

---

## Screen-by-Screen Breakdown

### Auth Screens (`app/(auth)/`)

#### `login.tsx`
- Biometric authentication prompt
- PIN fallback option
- "Forgot PIN?" recovery flow
- Auto-navigate to dashboard on success

#### `register.tsx`
- Generate new seed phrase
- Display 12-word mnemonic
- Confirm backup (re-enter words)
- Create PIN/biometric lock
- Submit to parachain

#### `recovery.tsx`
- Import existing seed phrase
- 12/24 word input fields
- Validate mnemonic
- Restore identity from blockchain

### Wallet Screens (`app/(wallet)/`)

#### `index.tsx` (Dashboard)
- Overview statistics
- Recent credentials
- Quick actions (scan QR, generate proof)
- Identity status widget
- Network connection indicator

#### Identity Screens (`identity/`)

**`index.tsx`** - Identity List
- Display current DID
- QR code representation
- Copy DID button
- Status badge (Active/Inactive)

**`create.tsx`** - Create New Identity
- Generate keypair form
- DID format preview
- Submit to parachain
- Success confirmation

**`[id].tsx`** - Identity Details
- Full DID display
- Public key (truncated + copy)
- Creation timestamp
- Associated credentials count
- Actions (update, deactivate)

**`manage.tsx`** - Manage Identity
- Update public key
- Deactivate identity
- View transaction history
- Export identity data

#### Credential Screens (`credentials/`)

**`index.tsx`** - Credentials List
- Grid/list view of credentials
- Filter by type (Education, Health, etc.)
- Sort by date/issuer
- Search functionality
- Pull-to-refresh

**`[id].tsx`** - Credential Details
- Issuer information
- Credential type
- Issue/expiry dates
- Status (Active/Revoked/Expired)
- Field preview (encrypted)
- Actions (share, generate proof)

**`share.tsx`** - Share Credential
- Select sharing method (QR, NFC)
- Choose fields to reveal
- Generate temporary share link
- Expiry timer

**`qr.tsx`** - QR Code Display
- Full-screen QR code
- Credential summary
- Auto-expire after 2 minutes
- Brightness boost

#### Proof Screens (`proof/`)

**`index.tsx`** - Generate Proof
- Select credential
- Choose proof type (Age, Student, etc.)
- Field selector (checkboxes)
- Real-time proof preview
- Generate button

**`confirm.tsx`** - Proof Confirmation
- Review fields to reveal
- Estimated verification time
- Warning about irreversibility
- Confirm/Cancel buttons
- Submit to chain

**`history.tsx`** - Proof History
- List of generated proofs
- Timestamp and verifier
- Fields revealed
- Verification status
- Proof validity period

#### Settings Screens (`settings/`)

**`index.tsx`** - Settings Home
- Security settings
- Network configuration
- Backup & restore
- About/version info
- Clear cache

**`biometric.tsx`** - Biometric Setup
- Enable/disable Face ID/Touch ID
- Test biometric
- Fallback PIN
- Supported methods detection

**`backup.tsx`** - Backup & Recovery
- View seed phrase (requires auth)
- Export encrypted backup
- Cloud backup (optional)
- Restore instructions

---

## Key Services Implementation

### 1. **Substrate API Service** (`services/substrate/api.ts`)

```typescript
class SubstrateAPI {
  private api: ApiPromise | null;
  private wsProvider: WsProvider | null;
  
  async connect(endpoint: string): Promise<void>;
  async disconnect(): Promise<void>;
  getApi(): ApiPromise;
  isConnected(): boolean;
  subscribeToBlocks(callback: (block) => void): Unsubscribe;
}
```

**Features:**
- WebSocket connection management
- Auto-reconnect on failure
- Connection status monitoring
- Event subscriptions

### 2. **Key Management Service** (`services/crypto/keyManagement.ts`)

```typescript
class KeyManagement {
  async generateSeedPhrase(): Promise<string>;
  async importFromSeed(seed: string): Promise<KeyringPair>;
  async storeKeyPair(pair: KeyringPair): Promise<void>;
  async getKeyPair(): Promise<KeyringPair>;
  async deleteKeyPair(): Promise<void>;
  async exportKeyPair(): Promise<string>;
}
```

**Security:**
- Keys stored in Secure Enclave
- Never expose private keys
- Encrypted at rest
- Biometric-protected access

### 3. **Transaction Service** (`services/substrate/calls.ts`)

```typescript
class TransactionService {
  async createIdentity(did: string, publicKey: string): Promise<Hash>;
  async updateIdentity(newKey: string): Promise<Hash>;
  async deactivateIdentity(): Promise<Hash>;
  async verifyCredential(credentialId: string): Promise<boolean>;
  async generateProof(credentialId: string, fields: number[]): Promise<string>;
}
```

**Features:**
- Transaction signing
- Gas estimation
- Nonce management
- Error handling

### 4. **Storage Service** (`services/storage/secureStorage.ts`)

```typescript
class SecureStorage {
  async setItem(key: string, value: string): Promise<void>;
  async getItem(key: string): Promise<string | null>;
  async removeItem(key: string): Promise<void>;
  async requireAuth(key: string): Promise<boolean>;
}
```

**Data Stored:**
- Encrypted seed phrase
- Key pairs
- User preferences
- Cached credentials
- Proof history

---

## UI/UX Design Principles

### Color Palette
```typescript
const colors = {
  primary: '#6366F1',      // Indigo - Trust
  secondary: '#8B5CF6',    // Purple - Innovation
  success: '#10B981',      // Green - Active/Verified
  warning: '#F59E0B',      // Amber - Expiring
  error: '#EF4444',        // Red - Revoked/Error
  background: '#FFFFFF',   // White
  surface: '#F9FAFB',      // Light gray
  text: '#111827',         // Dark gray
  textSecondary: '#6B7280', // Medium gray
};
```

### Typography
```typescript
const typography = {
  h1: { fontSize: 32, fontWeight: '700' },
  h2: { fontSize: 24, fontWeight: '600' },
  h3: { fontSize: 20, fontWeight: '600' },
  body: { fontSize: 16, fontWeight: '400' },
  caption: { fontSize: 14, fontWeight: '400' },
  button: { fontSize: 16, fontWeight: '600' },
};
```

### Spacing System
```typescript
const spacing = {
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
  xxl: 48,
};
```

---

## Error Handling Strategy

### Error Types
1. **Network Errors**: Connection failures, timeouts
2. **Authentication Errors**: Invalid credentials, expired sessions
3. **Transaction Errors**: Insufficient balance, nonce issues
4. **Validation Errors**: Invalid input, missing fields
5. **Storage Errors**: Secure store failures

### Error Display
- Toast notifications for minor errors
- Modal dialogs for critical errors
- Inline validation for form errors
- Retry buttons for recoverable errors
- Error logging for debugging

---

## Performance Optimization

### Strategies
1. **Lazy Loading**: Load screens on demand
2. **Memoization**: Cache expensive computations
3. **Virtual Lists**: Render only visible items
4. **Image Optimization**: Compress and cache images
5. **API Batching**: Combine multiple requests
6. **Local Caching**: Store frequently accessed data

### Metrics to Monitor
- App startup time < 2s
- Screen transition < 300ms
- API response time < 1s
- Memory usage < 200MB
- Battery consumption (background)

---

## Testing Strategy

### Unit Tests
- Store actions and reducers
- Utility functions
- Crypto operations
- Validation logic

### Integration Tests
- API service interactions
- Storage service operations
- Transaction flows
- Authentication flows

### E2E Tests (Detox)
- Complete user journeys
- Critical paths (create ID, receive credential)
- Error scenarios
- Edge cases

---

## Development Phases

### Phase 1: Foundation
- ✅ Project setup with Expo
- ✅ Folder structure
- ✅ Navigation setup (Expo Router)
- ✅ Core UI components
- ✅ Theme and styling system

### Phase 2: Blockchain Integration
- Polkadot.js API integration
- WebSocket connection management
- Transaction signing
- Account creation
- Basic identity operations

### Phase 3: Identity & Credentials
- Identity creation flow
- Credential display
- Credential verification
- QR code generation/scanning
- Local storage

### Phase 4: ZK Proofs
- Proof generation UI
- Field selector
- ZK circuit integration
- Proof verification
- Share proof functionality

### Phase 5: Security & Polish
- Biometric authentication
- Secure storage implementation
- Backup/recovery flows
- Error handling
- Performance optimization

### Phase 6: Testing & Deployment
- Comprehensive testing
- Bug fixes
- App store preparation
- Beta release
- Production deployment

---
