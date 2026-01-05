# PortableID Web Interface: Technical Documentation

## 1. Introduction

The PortableID Web Interface is a comprehensive management portal for the PortableID ecosystem. It provides the necessary tools for ecosystem participants—including administrators, trusted issuers, and general users—to interact with the parachain, manage identities, and monitor ecosystem health.

## 2. Platform Roles

The application is structured around four primary functional areas:

- **Admin Portal**: For ecosystem governance, managed by the DAO. Used to approve/revoke trusted issuers, update system parameters, and manage emergency pauses.
- **Issuer Dashboard**: For authorized entities to issue Verifiable Credentials to users. Features include batch issuance, credential template management, and analytics.
- **Blockchain Explorer**: A real-time data visualizer for the PortableID parachain, showing blocks, extrinsics, events, and DID statistics.
- **User Portal**: A simplified interface for users to view their on-chain DID status and public profile.

## 3. Technology Stack

- **Frontend Framework**: React 18+
- **Build Tool**: Vite
- **Language**: TypeScript
- **State Management**: Redux Toolkit
- **Styling**: Vanilla CSS with Tailwind CSS (PostCSS)
- **Blockchain Interface**: @polkadot/api & @polkadot/extension-dapp
- **Icons**: Lucide React
- **Charts**: Recharts (for analytics and explorer)

## 4. Architecture

The codebase follows a clear separation of concerns:

### 4.1 Page Architecture (`src/pages`)
- **`admin/`**: Governance tools, council voting, and system configuration.
- **`issuer/`**: Credential issuance forms, revocation management, and dashboard.
- **`explorer/`**: Block list, transaction details, and network health monitors.
- **`auth/`**: Wallet connection logic (Polkadot{.js}, Talisman, SubWallet).

### 4.2 Service Layer (`src/services`)
- **`substrate/`**: 
    - `connection.ts`: Manages the WebSocket connection to the parachain node.
    - `calls.ts`: Logic for signed extrinsics (issuer voting, credential issuance).
    - `queries.ts`: Data fetching for chain state (DID metadata, schema details).
    - `events.ts`: Listener for on-chain events to provide real-time UI updates.
- **`api/`**: Interaction with off-chain services (e.g., the Machine Learning Oracle or IPFS).

### 4.3 State Management (`src/store`)
Uses Redux Toolkit to manage:
- **Blockchain State**: Connected account, network status, latest block data.
- **User Session**: Authentication state and permissions based on on-chain roles.
- **UI State**: Theme, notifications, and modal management.

## 5. Key Features

### 5.1 Credential Issuance Flow
1. **Selection**: Issuer selects a credential schema (e.g., "University Degree").
2. **Data Input**: Issuer enters the subject's DID and required data fields.
3. **Hashing**: The app computes the `data_hash` as per the parachain specification.
4. **Signing**: The issuer signs the payload using their browser wallet extension.
5. **Submission**: The extrinsic is submitted to the `pallet-verifiable-credentials`.
6. **Confirmation**: UI monitors the chain for the `CredentialIssued` event.

### 5.2 Ecosystem Governance
The Web Interface provides a visual UI for Polkadot's governance pallets:
- **Motion Proposing**: Propose new issuers or changes to trusted schemas.
- **Voting**: Democracy-style voting for councillors and token holders.
- **Enactment**: Tracking the status of proposals from creation to enactment.

### 5.3 Real-time Explorer
- **Block Stream**: Live feed of blocks being finalized by the Aura consensus.
- **Extrinsic Inspection**: Decoding of on-chain calls to show human-readable actions.
- **Identity Search**: Look up any DID to see its public keys and service endpoints.

## 6. Security

- **Wallet-Based Auth**: No passwords; authentication is handled by cryptographic signatures via Polkadot browser extensions.
- **Role-Based Access Control (RBAC)**: Interface components are strictly guarded based on the connected account's on-chain permissions.
- **Sanitized Inputs**: All user-provided data is validated and sanitized before being used in cryptographic operations.

## 7. Development and Deployment

### 7.1 Local Development
```bash
npm install
npm run dev
```

### 7.2 Configuration
Environment variables (.env):
- `VITE_WS_ENDPOINT`: WebSocket URL of the parachain (e.g., `ws://127.0.0.1:9944`)
- `VITE_EXPLORER_API`: (Optional) Backend API for historical data indexing.

---
*Documentation Version: 1.0.0*
