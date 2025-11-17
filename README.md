# 🔐 PortableID

A digital decentralised identity wallet for decentralized identity management with zero-knowledge proofs, verifiable credentials, on-chain governance, and cross-chain interoperability.
 
## 📋 Table of Contents

- [Overview](#overview)
- [Problem Statement](#problem-statement)
- [Solution](#solution)
- [Key Features](#key-features)
- [Technology Stack](#technology-stack)
- [Architecture](#architecture)
- [User Flows](#user-flows)
- [Technical Flow](#technical-flow)
- [Project Structure](#project-structure)
- [Installation & Setup](#installation--setup)
- [Development](#development)
- [API Reference](#api-reference)
- [Industry Standards & Best Practices](#industry-standards--best-practices)
- [Security Considerations](#security-considerations)
- [Testing](#testing)
- [Deployment](#deployment)
- [Contributing](#contributing)

---

## Overview

**PortableID** is a decentralized identity (DID) solution built on Polkadot that enables self-sovereign identity management with privacy-preserving verification. Users can prove claims about themselves without revealing underlying sensitive data using zero-knowledge proofs (ZK-SNARKs).

### Key Differentiators

- **Privacy-First**: ZK proofs enable selective disclosure
- **Decentralized**: No central authority controls identities
- **Interoperable**: Cross-chain credential verification via XCM
- **Governed**: Democratic issuer approval through on-chain voting
- **Enterprise-grade security and scalability**:

---

## Problem Statement

### Current Identity Challenges

1. **Privacy Violation**: Organizations demand full personal data (ID, SSN, medical history) for simple verification
2. **Data Centralization**: Personal data scattered across multiple centralized databases vulnerable to breaches
3. **No User Control**: Users cannot manage their own identity or data
4. **Credential Verification**: No standardized, trustless way to verify credentials across organizations
5. **Interoperability Gap**: Each platform maintains separate identity silos
6. **Compliance Burden**: Organizations must manage and secure sensitive user data (GDPR, privacy regulations)

### Real-World Examples

- **Student Discounts**: Universities share full student records just to verify enrollment
- **Age Verification**: Bars see your entire ID including address and organ donor status
- **Job Applications**: Candidates send complete employment history for one reference check
- **Healthcare**: Hospitals access entire medical records just to confirm vaccination status

---

## Solution

### How It Works

```
┌─────────────────────────────────────────────────────────────┐
│ Traditional System                                          │
├─────────────────────────────────────────────────────────────┤
│ User → [Full Data] → Organization → Stores Centrally       │
│ User's SSN, Address, Phone, Medical History All Visible    │
└─────────────────────────────────────────────────────────────┘

                            ↓↓↓

┌─────────────────────────────────────────────────────────────┐
│ PortableID Solution                                 │
├─────────────────────────────────────────────────────────────┤
│ User DID → Credential (Encrypted Hash) → ZK Proof          │
│ Verifier Only Sees: "User is a valid student" ✓            │
│ User's ID, GPA, Enrollment Date Remain Private             │
└─────────────────────────────────────────────────────────────┘
```

### Key Innovation: Selective Disclosure

Users can prove specific claims without revealing the entire credential:

```
Credential Fields:
├─ Institution: "MIT" ✓ (Revealed)
├─ Student ID: "12345" ✗ (Hidden)
├─ Status: "Active" ✓ (Revealed)
└─ GPA: "3.8" ✗ (Hidden)

ZK Proof: "I have a valid active credential from an accredited institution"
Verifier: ✓ Access Granted (knows only what's necessary)
User Privacy: ✓ Protected (sensitive data not disclosed)
```

---

## Key Features

### 1. Decentralized Identity Registry
- Self-sovereign DID creation and management
- Multiple authentication methods support
- Identity lifecycle management (create, update, deactivate, reactivate)
- DID Document standards compliant

### 2. Verifiable Credentials System
- Issue credentials from trusted organizations
- Support for multiple credential types (Education, Health, Employment, Age, Address)
- Credential expiration and revocation
- Batch credential operations for efficiency
- Credential schemas for standardization

### 3. Zero-Knowledge Proofs (ZK-SNARKs)
- Groth16 proofs using Arkworks library
- Age verification circuits
- Student status proofs
- Vaccination status proofs
- Replay attack prevention
- Batch proof verification for scalability

### 4. On-Chain Governance
- Democratic issuer approval through voting
- Council-based governance model
- Proposal deposit system (anti-spam)
- Voting period and approval thresholds
- Emergency revocation powers for root

### 5. Cross-Chain Credentials (XCM)
- Verify credentials across parachains
- Export and import credentials between chains
- Parachain registry and trust management
- Multi-chain validation with consensus

### 6. Advanced Cryptography
- Multiple signature schemes (Ed25519, Sr25519, ECDSA)
- Proper signature verification on-chain
- Merkle tree support for batch operations
- Field element conversion for ZK circuits

### 7. Mobile & Web Interfaces
- React Native mobile app (iOS/Android)
- Web portal for organizations (credential issuers)
- QR code credential sharing
- Biometric authentication support
- Secure local key storage

### 8. Proof of Personhood & Sybil Resistance
- Biometric-derived nullifiers (never stores raw biometrics)
- Zero-knowledge uniqueness proofs
- 6-month time-locked recovery mechanism
- Social recovery with guardian approvals
- Registration cooldown periods

---

## Technology Stack

### Parachain Layer
- **Runtime**: Polkadot SDK 2503.0.1 (FRAME)
- **Language**: Rust (Edition 2021)
- **Consensus**: Aura (for parachain blocks)
- **Custom Pallets**: 5 specialized modules

### Cryptography & ZK
- **ZK Framework**: Arkworks (BN254 curve)
- **Proof System**: Groth16 ZK-SNARKs
- **Hashing**: Blake2-256
- **Signatures**: Ed25519, Sr25519, ECDSA

### Proof of Personhood
- **Nullifier Generation**: Blake2-256 hashing
- **Commitment Schemes**: Pedersen commitments
- **Recovery Mechanism**: Time-locked with guardian approval
- **Sybil Prevention**: Cooldown periods, cost barriers

### Frontend Stack
- **Web**: React 18, TypeScript, Polkadot.js API
- **Mobile**: React Native, Expo, Native Storage
- **State**: Redux for global state
- **Styling**: TailwindCSS (web), Native StyleSheet (mobile)

### Infrastructure
- **Node**: Cumulus-based parachain node
- **RPC**: JSON-RPC over WebSocket
- **Database**: RocksDB (parachain state)
- **Testing**: FRAME testing framework, integration tests

### Dependencies (Latest Versions - Nov 2025)
```toml
polkadot-sdk = "2503.0.1"
arkworks = "0.4.0"
parity-scale-codec = "3.7.4"
scale-info = "2.11.6"
substrate-wasm-builder = "26.0.1"
```

---

## Architecture

### System Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                      Polkadot Relay Chain                      │
└────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
   ┌─────────┐          ┌──────────┐          ┌──────────┐
   │Parachain│          │ Identity │          │  Other   │
   │  A      │          │ Parachain│          │Parachains│
   └─────────┘          └──────────┘          └──────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
   ┌─────────────┐  ┌──────────────┐  ┌──────────────┐
   │   Mobile    │  │    Web       │  │   XCM        │
   │   Wallet    │  │   Portal     │  │ Integration  │
   └─────────────┘  └──────────────┘  └──────────────┘
```

### Parachain Runtime Composition

```
┌─────────────────────────────────────────────────────────────┐
│                    Runtime (lib.rs)                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Frame Pallets (System, Balances, Timestamp, etc.)         │
│                                                             │
│  Custom Pallets:                                           │
│  ├─ pallet-identity-registry                              │
│  │  └─ Create/manage DIDs, identity lifecycle             │
│  │                                                         │
│  ├─ pallet-verifiable-credentials                         │
│  │  └─ Issue/verify/revoke credentials                    │
│  │                                                         │
│  ├─ pallet-zk-credentials                                 │
│  │  └─ ZK proof verification, circuits                    │
│  │                                                         │
│  ├─ pallet-credential-governance                          │
│  │  └─ On-chain voting, issuer approval                   │
│  │                                                         │
│  └─ pallet-xcm-credentials                                │
│     └─ Cross-chain credential verification                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow Architecture

```
┌──────────────────┐
│  User Creates    │
│  DID             │
└────────┬─────────┘
         │
         ▼
┌──────────────────────────────────────┐
│ Identity Registry Pallet             │
├──────────────────────────────────────┤
│ - Hashes DID                         │
│ - Stores Identity{public_key, time}  │
│ - Creates DID Document               │
└────────┬─────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────┐
│ Issuer Issues Credential             │
├──────────────────────────────────────┤
│ - Verifies issuer is trusted         │
│ - Creates Credential{fields_hash}    │
│ - Signs credential                   │
└────────┬─────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────┐
│ User Generates ZK Proof              │
├──────────────────────────────────────┤
│ - Selects fields to reveal           │
│ - Generates zero-knowledge proof     │
│ - Creates disclosure request         │
└────────┬─────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────┐
│ Verifier Verifies On-Chain           │
├──────────────────────────────────────┤
│ - Checks ZK proof validity           │
│ - Verifies credential not revoked    │
│ - Checks expiration date             │
│ - Returns verification result        │
└──────────────────────────────────────┘
```

---

## User Flows

### Flow 1: Student Discount Scenario

```
SETUP PHASE
───────────
1. University registers as trusted issuer (governance vote)
   - Council votes on university credentials
   - Threshold met → University approved

2. University creates credential schema
   - Defines fields: institution, studentId, status, gpa
   - Sets required fields

3. Student creates DID
   - Student account: Alice
   - Generates keypair
   - Stores DID on parachain

ISSUANCE PHASE
──────────────
4. Student applies to university (off-chain)
   - Provides enrollment proof
   - University verifies student

5. University issues credential
   - Creates credential with encrypted data hash
   - Signs with private key
   - Stores on parachain

VERIFICATION PHASE
──────────────────
6. Student visits online store
   - Store requests "proof of student status"
   
7. Student generates ZK proof
   - Proves: institution ✓, status=Active ✓
   - Hides: studentId ✗, gpa ✗
   - Submits proof to store

8. Store verifies on-chain
   - Calls verify_credential() on parachain
   - Verifies ZK proof validity
   - Checks credential not revoked
   - Result: ✓ VALID

9. Store grants discount
   - Transaction completed
   - Privacy maintained
```

**Privacy Achieved**: Store only knows student is enrolled, not student ID or grades

### Flow 2: Credential Issuance

```
PARTICIPANT: Hospital Issuing Vaccination Credential
─────────────────────────────────────────────────────

Step 1: Hospital Setup (First Time)
   └─ Hospital registers as issuer
   └─ Governance vote: council approves
   └─ Creates schema: patient_id, vaccination_type, date

Step 2: Patient Gets Vaccinated
   └─ Patient presents off-chain ID
   └─ Hospital verifies against government database
   └─ Vaccination recorded

Step 3: Credential Issued On-Chain
   Transaction: issue_credential()
   ├─ subject_did: patient's DID
   ├─ credential_type: Health
   ├─ data_hash: H256(patient_data encrypted)
   ├─ expires_at: timestamp + 12 months
   └─ signature: hospital's signature

Step 4: Patient Proves Vaccination
   Transaction: selective_disclosure()
   ├─ credential_id: (from Step 3)
   ├─ fields_to_reveal: [vaccination_type, date]
   ├─ proof: ZK proof (patient hid: patient_id)
   └─ Result: "Patient vaccinated for COVID-19 on 2025-01-15"

Step 5: Event Venue Verifies
   Query: verify_credential()
   ├─ Checks: credential active
   ├─ Checks: not expired
   ├─ Checks: issuer trusted
   └─ Result: ✓ GRANTED ACCESS
```

### Flow 3: Governance Proposal

```
COUNCIL VOTING FLOW
──────────────────

Proposer: Healthcare Organization
   │
   ├─ Step 1: Create Proposal
   │  └─ propose_add_issuer()
   │     ├─ issuer_did: healthcare_org
   │     ├─ credential_types: [Health]
   │     ├─ description: "Regional Hospital - Trusted Provider"
   │     └─ deposit: 100 TOKENS (anti-spam)
   │
   ├─ Step 2: Council Voting Period (7 days)
   │  ├─ Council Member 1 votes: YES (10 voting power)
   │  ├─ Council Member 2 votes: YES (15 voting power)
   │  └─ Council Member 3 votes: NO (5 voting power)
   │
   ├─ Step 3: Voting Ends
   │  └─ Yes votes: 25, No votes: 5
   │  └─ Approval: (25/30) = 83% > 66% threshold
   │
   ├─ Step 4: Finalize Proposal
   │  └─ finalize_proposal()
   │     ├─ Status: Approved → Executed
   │     ├─ TrustedIssuers updated
   │     └─ Deposit returned to proposer
   │
   └─ Step 5: Issuer Can Now Issue
      └─ Healthcare org calls issue_credential()
         └─ Verification passes (issuer is trusted)
```

## Proof of Personhood Flow

### Registration with Biometric Nullifier
```
CLIENT SIDE (Never Leaves Device)
─────────────────────────────────
1. User provides biometric (fingerprint/face)
2. Extract template → biometric_data
3. Generate: salt = random_bytes(32)
4. Compute: nullifier = Hash(biometric_data)
5. Compute: commitment = Hash(biometric_data || salt)
6. Store salt encrypted in local secure storage

PARACHAIN VERIFICATION
──────────────────────
1. Check nullifier NOT in PersonhoodRegistry
2. Verify nullifier is unique (no duplicates)
3. Store: PersonhoodRegistry[nullifier] = {
     commitment,
     registered_at: timestamp,
     did: user_did
   }
4. Success: User registered as unique person
```

### Recovery After Device Loss
```
STEP 1: Request Recovery (User on New Device)
──────────────────────────────────────────────
- User captures NEW biometric on new device
- Generate new_nullifier = Hash(new_biometric)
- Nominate 3-5 guardians (trusted contacts)
- Submit recovery request to chain
- 6-month cooldown period starts

STEP 2: Guardian Approval (During 6 Months)
────────────────────────────────────────────
- Guardians notified on-chain
- Minimum 2/3 must approve recovery
- Each guardian calls approve_recovery()

STEP 3: Finalize Recovery (After 6 Months)
──────────────────────────────────────────
- User calls finalize_recovery()
- Checks: cooldown elapsed + guardian approvals
- Old nullifier deleted from registry
- New nullifier registered
- DID ownership transferred to new nullifier
```

---

## Technical Flow

### Credential Issuance Deep Dive

```
USER CALLS: issue_credential(
  subject_did: H256,
  credential_type: CredentialType,
  data_hash: H256,
  expires_at: u64,
  signature: H256
)

PARACHAIN EXECUTION:
│
├─ Step 1: Authorization Check
│  ├─ Get issuer DID from signer's account
│  ├─ Verify issuer identity exists and is active
│  └─ Error if not: IssuerIdentityNotFound
│
├─ Step 2: Subject Verification
│  ├─ Verify subject DID exists
│  ├─ Verify subject identity is active
│  └─ Error if not: SubjectIdentityNotFound
│
├─ Step 3: Issuer Trust Check
│  ├─ Query TrustedIssuers storage
│  ├─ Check: TrustedIssuers[credential_type][issuer_did] == true
│  └─ Error if not: IssuerNotTrusted
│
├─ Step 4: Credential Generation
│  ├─ Create Credential struct:
│  │  ├─ subject: subject_did
│  │  ├─ issuer: issuer_did
│  │  ├─ credential_type: credential_type
│  │  ├─ data_hash: data_hash (hash of encrypted data)
│  │  ├─ issued_at: current_block_timestamp
│  │  ├─ expires_at: expires_at
│  │  ├─ status: Active
│  │  └─ signature: signature
│  │
│  └─ Generate credential ID:
│     └─ credential_id = blake2_256([
│           subject_did.bytes,
│           issuer_did.bytes,
│           data_hash.bytes,
│           issued_at.bytes
│        ])
│
├─ Step 5: Storage Updates
│  ├─ Store credential in Credentials[credential_id] = credential
│  ├─ Add to CredentialsOf[subject_did] vec (for subject lookup)
│  └─ Add to IssuedBy[issuer_did] vec (for issuer lookup)
│
├─ Step 6: Event Emission
│  └─ Emit CredentialIssued {
│       credential_id,
│       subject: subject_did,
│       issuer: issuer_did,
│       credential_type
│     }
│
└─ Step 7: Return Result
   └─ Ok(()) - Transaction successful
```

### ZK Proof Verification Flow

```
USER CALLS: verify_proof(
  proof: ZkProof {
    proof_type: ProofType::StudentStatus,
    proof_data: vec![...],      // Groth16 proof
    public_inputs: vec![...],   // Revealed fields
    credential_hash: H256,
    created_at: u64
  }
)

PARACHAIN EXECUTION:
│
├─ Step 1: Get Verification Key
│  ├─ Query VerifyingKeys[proof_type]
│  ├─ Deserialize from storage
│  └─ Error if not: VerificationKeyNotFound
│
├─ Step 2: Replay Attack Prevention
│  ├─ Calculate proof_hash = blake2_256(proof_data + public_inputs + credential_hash)
│  ├─ Check: NOT VerifiedProofs[proof_hash]
│  └─ Error if replayed: ProofAlreadyVerified
│
├─ Step 3: Cryptographic Verification
│  ├─ Deserialize verification key:
│  │  └─ VerifyingKey::<Bn254>::deserialize_compressed(vk_data)
│  │
│  ├─ Prepare verification key:
│  │  └─ prepare_verifying_key(&vk)
│  │
│  ├─ Deserialize proof:
│  │  └─ Proof::<Bn254>::deserialize_compressed(proof_data)
│  │
│  ├─ Convert public inputs to field elements:
│  │  └─ For each input in public_inputs:
│  │     └─ Fr::from_be_bytes_mod_order(input)
│  │
│  └─ Execute Groth16 verification:
│     └─ ark_groth16::verify_proof(&pvk, &proof, &inputs)
│
├─ Step 4: Result Handling
│  ├─ If verification FAILS:
│  │  ├─ Emit ProofVerificationFailed event
│  │  └─ Return Error: ProofVerificationFailed
│  │
│  └─ If verification SUCCEEDS:
│     ├─ Store in VerifiedProofs[proof_hash] = (caller, timestamp)
│     ├─ Emit ProofVerified event
│     └─ Return Ok(())
│
└─ Step 5: Prevent Future Replay
   └─ proof_hash now in storage
   └─ Same proof cannot be verified again
   └─ Different proof with same data fails (only valid once)
```

### Cross-Chain Credential Verification

```
PARACHAIN A (Source):
│
├─ User has credential: H256(credential_data)
├─ Wants verification from Parachain B
│
└─ Calls: request_cross_chain_verification()
   ├─ Check parachain B is registered and trusted
   │
   ├─ Create verification request:
   │  └─ XcmCredentialRequest {
   │       source_para_id: 2000,
   │       credential_hash: H256,
   │       requester: user_account,
   │       timestamp: current_time
   │     }
   │
   ├─ Store pending request
   │
   └─ Send XCM message to Parachain B:
      └─ Xcm(vec![
           Transact {
             call: handle_verification_request(credential_hash),
             weight: 1_000_000_000
           }
         ])

                    ↓↓↓ XCM MESSAGE ↓↓↓

PARACHAIN B (Target):
│
├─ Receives XCM message
│
└─ Executes: handle_verification_request()
   ├─ Query local Credentials[credential_hash]
   ├─ Verify credential:
   │  ├─ Not revoked
   │  ├─ Not expired
   │  └─ Issuer trusted
   │
   ├─ Generate response:
   │  └─ XcmCredentialResponse {
   │       target_para_id: 2001,
   │       credential_hash: H256,
   │       is_valid: true,
   │       metadata: "..."
   │     }
   │
   └─ Send XCM response back to Parachain A:
      └─ Xcm(vec![
           Transact {
             call: handle_verification_response(
               credential_hash,
               is_valid,
               metadata
             )
           }
         ])

                    ↓↓↓ XCM MESSAGE ↓↓↓

PARACHAIN A:
│
└─ Receives response
   ├─ Store in VerificationResults[credential_hash]
   ├─ Majority consensus check (if multiple validators)
   └─ User can now prove cross-chain validation ✓
```

---

## Project Folder Structure

```
identity-parachain/
│
├── parachain/                    # Polkadot Parachain (Rust/FRAME)
│   ├── Cargo.toml
│   ├── Cargo.lock
│   │
│   ├── node/                     # Node binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── service.rs
│   │       └── command.rs
│   │
│   ├── runtime/                  # Runtime logic
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/
│   │       ├── lib.rs            # Main runtime file
│   │       ├── apis.rs           # Runtime APIs
│   │       ├── benchmarks.rs
│   │       ├── genesis_config_presets.rs
│   │       ├── configs/
│   │       │   ├── mod.rs
│   │       │   └── xcm_config.rs
│   │       └── weights/
│   │           ├── mod.rs
│   │           ├── block_weights.rs
│   │           ├── extrinsic_weights.rs
│   │           └── rocksdb_weights.rs
│   │
│   └── pallets/                  # Custom FRAME pallets
│       ├── pallet-identity-registry/
│       │   ├── Cargo.toml
│       │   └── src/
│       │       └── lib.rs
│       ├── pallet-proof-of-personhood/
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── benchmarking.rs
│       │       └── weights.rs
│       │
│       ├── pallet-verifiable-credentials/
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       └── tests.rs
│       │
│       ├── pallet-zk-credentials/
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       └── circuits.rs
│       │
│       ├── pallet-credential-governance/
│       │   ├── Cargo.toml
│       │   └── src/
│       │       └── lib.rs
│       │
│       └── pallet-xcm-credentials/
│           ├── Cargo.toml
│           └── src/
│               └── lib.rs
│
│
├── web/                          # Web Interface (React)
│   ├── package.json
│   ├── tsconfig.json
│   ├── tailwind.config.js
│   ├── .env.example
│   │
│   ├── public/
│   │   └── index.html
│   │
│   └── src/
│       ├── App.tsx
│       ├── index.tsx
│       │
│       ├── pages/
│       │   ├── auth/
│       │   │   ├── LoginPage.tsx
│       │   │   ├── RegisterPage.tsx
│       │   │   └── AdminLogin.tsx
│       │   │
│       │   ├── issuer/
│       │   │   ├── Dashboard.tsx
│       │   │   ├── IssuePage.tsx
│       │   │   ├── CredentialManagement.tsx
│       │   │   ├── GovernancePanel.tsx
│       │   │   └── AnalyticsPage.tsx
│       │   │
│       │   ├── admin/
│       │   │   ├── AdminDashboard.tsx
│       │   │   ├── ProposalPage.tsx
│       │   │   ├── CouncilManagement.tsx
│       │   │   └── AuditLog.tsx
│       │   │
│       │   └── explorer/
│       │       ├── ExplorerPage.tsx
│       │       ├── CredentialExplorer.tsx
│       │       └── IdentityExplorer.tsx
│       │
│       ├── components/
│       │   ├── layout/
│       │   │   ├── Header.tsx
│       │   │   ├── Sidebar.tsx
│       │   │   └── Footer.tsx
│       │   │
│       │   ├── forms/
│       │   │   ├── IssueCredentialForm.tsx
│       │   │   ├── ProposalForm.tsx
│       │   │   ├── VotingForm.tsx
│       │   │   └── SchemaForm.tsx
│       │   │
│       │   ├── displays/
│       │   │   ├── CredentialCard.tsx
│       │   │   ├── ProposalCard.tsx
│       │   │   ├── StatsCard.tsx
│       │   │   └── TimelineComponent.tsx
│       │   │
│       │   └── modals/
│       │       ├── ConfirmModal.tsx
│       │       ├── ErrorModal.tsx
│       │       └── SuccessModal.tsx
│       │
│       ├── services/
│       │   ├── api/
│       │   │   ├── client.ts
│       │   │   ├── auth.ts
│       │   │   ├── credentials.ts
│       │   │   ├── governance.ts
│       │   │   └── identity.ts
│       │   │
│       │   ├── substrate/
│       │   │   ├── connection.ts
│       │   │   ├── calls.ts
│       │   │   ├── queries.ts
│       │   │   └── events.ts
│       │   │
│       │   └── crypto/
│       │       ├── keyring.ts
│       │       └── signing.ts
│       │
│       ├── hooks/
│       │   ├── useApi.ts
│       │   ├── useAuth.ts
│       │   ├── useCredentials.ts
│       │   ├── useGovernance.ts
│       │   └── usePolling.ts
│       │
│       ├── store/
│       │   ├── authStore.ts
│       │   ├── credentialStore.ts
│       │   ├── governanceStore.ts
│       │   └── uiStore.ts
│       │
│       ├── types/
│       │   ├── index.ts
│       │   ├── api.ts
│       │   ├── credential.ts
│       │   └── governance.ts
│       │
│       ├── utils/
│       │   ├── formatting.ts
│       │   ├── validation.ts
│       │   ├── substrate.ts
│       │   └── errors.ts
│       │
│       └── styles/
│           ├── globals.css
│           ├── theme.css
│           └── animations.css
│
│
├──mobile/                                    # Mobile App (Expo + Expo Router)
│
├── package.json
├── app.json                              # Expo configuration
├── tsconfig.json
├── babel.config.js
├── .env.example
│
├── .expo/                                # Expo cache (auto-generated)
│
│
├── app/                                  # Expo Router routing (app directory)
│   ├── _layout.tsx                       # Root layout
│   ├── +not-found.tsx                    # 404 screen
│   │
│   ├── (auth)/                           # Auth group
│   │   ├── _layout.tsx
│   │   ├── login.tsx
│   │   ├── register.tsx
│   │   └── recovery.tsx
│   │
│   ├── (wallet)/                         # Main app group
│   │   ├── _layout.tsx
│   │   ├── index.tsx                     # Dashboard
│   │   │
│   │   ├── identity/
│   │   │   ├── _layout.tsx
│   │   │   ├── index.tsx                 # Identity list
│   │   │   ├── [id].tsx                  # Identity details
│   │   │   ├── create.tsx                # Create DID
│   │   │   └── manage.tsx                # Manage identity
│   │   │
│   │   ├── credentials/
│   │   │   ├── _layout.tsx
│   │   │   ├── index.tsx                 # Credentials list
│   │   │   ├── [id].tsx                  # Credential details
│   │   │   ├── share.tsx                 # Share credential
│   │   │   └── qr.tsx                    # QR code view
│   │   │
│   │   ├── proof/
│   │   │   ├── _layout.tsx
│   │   │   ├── index.tsx                 # Generate proof
│   │   │   ├── confirm.tsx               # Proof confirmation
│   │   │   └── history.tsx               # Proof history
│   │   │
│   │   └── settings/
│   │       ├── _layout.tsx
│   │       ├── index.tsx                 # Settings
│   │       ├── biometric.tsx             # Biometric setup
│   │       └── backup.tsx                # Backup recovery
│
├── src/                                  # Shared source code
│   │
│   ├── components/
│   │   ├── common/
│   │   │   ├── Button.tsx
│   │   │   ├── Input.tsx
│   │   │   ├── Card.tsx
│   │   │   ├── Loading.tsx
│   │   │   └── Modal.tsx
│   │   │
│   │   ├── credential/
│   │   │   ├── CredentialCard.tsx
│   │   │   ├── FieldSelector.tsx
│   │   │   └── ProofPreview.tsx
│   │   │
│   │   ├── identity/
│   │   │   ├── IdentityCard.tsx
│   │   │   ├── DIDDisplay.tsx
│   │   │   └── KeyPairManager.tsx
│   │   │
│   │   └── layout/
│   │       ├── SafeAreaView.tsx
│   │       ├── Header.tsx
│   │       └── TabBar.tsx
│   │
│   ├── services/
│   │   ├── substrate/
│   │   │   ├── api.ts                    # Polkadot.js setup
│   │   │   ├── calls.ts                  # Extrinsic calls
│   │   │   ├── queries.ts                # Storage queries
│   │   │   ├── types.ts                  # Substrate types
│   │   │   └── utils.ts             mobile/
│
├── app/                                  # Expo Router (screens)
│   ├── _layout.tsx
│   ├── +not-found.tsx
│   │
│   ├── (auth)/
│   │   ├── _layout.tsx
│   │   ├── login.tsx
│   │   ├── register.tsx
│   │   └── recovery.tsx
│   │
│   ├── (wallet)/
│   │   ├── _layout.tsx
│   │   ├── index.tsx                     # Dashboard
│   │   │
│   │   ├── identity/
│   │   │   ├── _layout.tsx
│   │   │   ├── index.tsx
│   │   │   ├── [id].tsx
│   │   │   └── create.tsx
│   │   │
│   │   ├── credentials/
│   │   │   ├── _layout.tsx
│   │   │   ├── index.tsx
│   │   │   ├── [id].tsx
│   │   │   ├── share.tsx
│   │   │   └── qr.tsx
│   │   │
│   │   ├── proof/
│   │   │   ├── _layout.tsx
│   │   │   ├── index.tsx                 # Proof Generation
│   │   │   ├── confirm.tsx
│   │   │   └── history.tsx
│   │   │
│   │   └── settings/
│   │       ├── _layout.tsx
│   │       ├── index.tsx
│   │       ├── biometric.tsx
│   │       └── backup.tsx
│   │
│
├── src/ #src code. 
│
├── rust-prover/                          # **Rust ZK Prover crate**
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                        # Rust proving logic (arkworks/halo2)
│   │   ├── ffi.rs                        # C ABI interface
│   │   ├── circuits/                     # ZK circuits
│   │   ├── proving/                      # Prover functions
│   │   └── utils/                        # Field/math utils
│   └── target/                           # Build artifacts (ignored in git)
│
├── android/
│   ├── app/
│   │   ├── src/main/
│   │   │   ├── java/com/mobile/zk/
│   │   │   │   ├── ZKProverModule.kt     # RN native module
│   │   │   │   ├── ZKProverPackage.kt    # RN module binder
│   │   │   │   └── ProverNative.kt       # JNI bridge to Rust
│   │   │   │
│   │   │   └── jniLibs/
│   │   │       ├── arm64-v8a/libprover.so
│   │   │       ├── armeabi-v7a/libprover.so
│   │   │       └── x86_64/libprover.so
│   │   │
│   │   └── AndroidManifest.xml
│   │
│   ├── build.gradle
│   ├── settings.gradle
│
├── ios/
│   ├── ZKProverModule.swift              # RN module implemented in Swift
│   ├── ProverBridge.swift                # Calls Rust C ABI
│   ├── rust-prover.xcframework/          # Rust built for iOS + Simulators
│   └── Podfile
│
├── modules/
│   └── zk-prover-expo-plugin/            # **Expo Config Plugin**
│       ├── app.plugin.js
│       ├── withZKProver.js               
│       └── README.md
│
├── scripts/
│   ├── build-rust-android.sh             # cargo-ndk build automation
│   ├── build-rust-ios.sh                 # xcframework builder
│   ├── clean.sh
│   └── verify-toolchains.sh
│
├── assets/
│   ├── images/
│   └── fonts/
│
├── app.json                              # Expo config
├── package.json
├── eas.json                              # EAS Build profiles
├── tsconfig.json
├── babel.config.js
└── .gitignore
├── README.md                     # Main README
├── LICENSE
└── .gitignore

```

---

## File Descriptions

### Parachain (Backend)

| File | Purpose |
|------|---------|
| `parachain/Cargo.toml` | Workspace config with all dependencies |
| `node/src/main.rs` | Node entry point |
| `node/src/service.rs` | Parachain service setup |
| `runtime/src/lib.rs` | Runtime construction |
| `runtime/src/apis.rs` | Runtime APIs for clients |
| `pallet-identity-registry/src/lib.rs` | DID management pallet |
| `pallet-verifiable-credentials/src/lib.rs` | Credential issuance pallet |
| `pallet-zk-credentials/src/lib.rs` | ZK proof verification pallet |
| `pallet-credential-governance/src/lib.rs` | On-chain voting pallet |
| `pallet-xcm-credentials/src/lib.rs` | Cross-chain credential pallet |
| `pallet-proof-of-personhood/src/lib.rs` | Biometric nullifier registration & recovery |

### Web (Frontend for Issuers)

| File | Purpose |
|------|---------|
| `web/package.json` | Dependencies & scripts |
| `web/src/App.tsx` | Root component |
| `web/src/pages/issuer/IssuePage.tsx` | Issue credentials UI |
| `web/src/pages/admin/ProposalPage.tsx` | Governance proposals |
| `web/src/services/substrate/calls.ts` | Blockchain transaction calls |
| `web/src/services/substrate/queries.ts` | Blockchain data queries |
| `web/src/store/authStore.ts` | Authentication state |
| `web/src/store/credentialStore.ts` | Credential state |

### Mobile (Wallet for Users)

| File | Purpose |
|------|---------|
| `mobile/package.json` | Dependencies & scripts |
| `mobile/app.json` | Expo configuration |
| `mobile/src/App.tsx` | Root component |
| `mobile/src/screens/identity/CreateDIDScreen.tsx` | Create wallet |
| `mobile/src/screens/credentials/CredentialListScreen.tsx` | View credentials |
| `mobile/src/screens/proof/GenerateProofScreen.tsx` | Generate ZK proof |
| `mobile/src/services/crypto/keyManagement.ts` | Key storage & management |
| `mobile/src/services/storage/biometric.ts` | Biometric authentication |
| `mobile/src/redux/slices/credentials.ts` | Credential state |

---

## Tech Stack Summary

| Layer | Technology | Version |
|-------|-----------|---------|
| **Parachain** | Rust + FRAME | 2025 |
| **Runtime** | Polkadot SDK | 2503.0.1 |
| **ZK Proofs** | Arkworks | 0.4.0 |
| **Crypto** | Blake2, Ed25519 | Latest |
| **Web Frontend** | React + TypeScript | 18+ |
| **Mobile** | React Native + Expo | Latest |
| **State Management** | Redux/Zustand | Latest |
| **API** | Polkadot.js | Latest |

---

## Installation Quick Start

### Parachain
```bash
cd parachain
cargo build --release
./target/release/parachain-template-node --dev
```

### Web
```bash
cd web
npm install
npm start
```

### Mobile
```bash
cd mobile
npm install
npm run ios  # or npm run android
```

# mobile .env file
# Environment Configuration
NODE_ENV=development #for development

# Network Configuration
#development
PARACHAIN_WS_ENDPOINT=ws://127.0.0.1:9944
PARACHAIN_ID=1000
NETWORK_NAME=Local Development

# API Configuration
API_TIMEOUT=30000
ENABLE_LOGGING=true

# Feature Flags
ENABLE_BIOMETRIC=true
ENABLE_QR_SHARING=true
ENABLE_MULTI_IDENTITY=false
ENABLE_CROSS_CHAIN=false

# App Configuration
APP_VERSION=1.0.0
MIN_PIN_LENGTH=6
MAX_PIN_LENGTH=8

# Security
PROOF_VALIDITY_PERIOD=3600
MAX_FIELDS_TO_DISCLOSE=50
PROOF_FRESHNESS_SECONDS=86400

# Storage Keys (Auto-prefixed with @identity_wallet/)
STORAGE_PREFIX=@identity_wallet

#Network Endpoints 

#Testnet
PARACHAIN_WS_ENDPOINT=wss://rococo-parachain-testnet.example.com
PARACHAIN_ID=1000
NETWORK_NAME=Rococo Testnet

---

## Development Workflow

### 1. Make Changes
- Edit files in respective directories
- Follow code style guidelines

### 2. Test
```bash
# Parachain tests
cd parachain && cargo test

# Web/Mobile (if applicable)
npm test
```

### 3. Build
```bash
# Parachain
cargo build --release

# Web
npm run build

# Mobile
eas build --platform ios
```

### 4. Deploy
- Push to testnet/production
- Update documentation
- Monitor for issues

---

## Key Pallet Structure

Each custom pallet follows this pattern:

```rust
#[frame_support::pallet]
pub mod pallet {
    // Config trait - define types needed
    pub trait Config: frame_system::Config { ... }
    
    // Storage - on-chain data
    #[pallet::storage]
    pub type Storage<T> = StorageMap<...>
    
    // Events - what happened
    #[pallet::event]
    pub enum Event<T> { ... }
    
    // Errors - what went wrong
    #[pallet::error]
    pub enum Error<T> { ... }
    
    // Calls - what users can do
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        pub fn some_function(...) -> DispatchResult { ... }
    }
    
    // Helpers - internal functions
    impl<T: Config> Pallet<T> {
        fn helper_function(...) { ... }
    }
}
```

---

## File Responsibilities

### Parachain Files
- **Pallets**: Business logic (identity, credentials, voting, ZK, XCM)
- **Runtime**: Composition of all pallets
- **Node**: Network communication

### Web Files
- **Pages**: Full-page views
- **Components**: Reusable UI pieces
- **Services**: API/blockchain calls
- **Store**: Global state

### Mobile Files
- **Screens**: Full-screen views
- **Components**: Reusable UI pieces
- **Services**: Substrate, crypto, storage
- **Redux**: Global state management

---

## Configuration Files

| File | Contains |
|------|----------|
| `.env` | Template for environment variables |
| `Cargo.toml` | Rust dependencies |
| `package.json` | Node dependencies |
| `tsconfig.json` | TypeScript settings |
| `tailwind.config.js` | CSS framework config |
| `app.json` | Mobile/Expo configuration |

### Proof of Personhood Security

**Biometric Safety**:
- Raw biometrics NEVER leave the device
- Only nullifiers (hashes) stored on-chain
- Biometric templates encrypted in device secure enclave
- Zero-knowledge proofs prevent identity linkage

**Sybil Attack Prevention**:
1. **Uniqueness**: Nullifiers prevent duplicate registrations
2. **Cost Barrier**: Registration deposits deter spam
3. **Time Locks**: 6-month cooldown between registrations
4. **Social Proof**: Guardian-based recovery requires trust networks

**Recovery Security**:
- 6-month delay prevents hasty account takeover
- 2/3 guardian approval required
- Active users can auto-cancel malicious recovery attempts
- Dormancy threshold (12 months) enables legitimate recovery

**Privacy Guarantees**:
- No biometric data on-chain (only commitments/nullifiers)
- ZK proofs reveal only "user is unique"
- Nullifiers cannot be reverse-engineered to biometrics
- Cross-DID unlinkability maintained