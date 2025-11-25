## Real-World Use Case: Global Universal Basic Income (UBI) Distribution

Imagine a **decentralized UBI system** running across multiple Polkadot parachains where every unique human receives monthly cryptocurrency payments, but the system must prevent:
- **Sybil attacks** (one person creating multiple identities to claim multiple payments)
- **Privacy violations** (not exposing biometric data or linking identities across services)
- **Lost identity recovery** (people losing access shouldn't lose their UBI forever)

### How this System Solves This:

**Registration Phase**: Alice uses her smartphone's fingerprint sensor. Her device computes `nullifier = Hash(fingerprint)` and `commitment = Hash(nullifier || random_salt)` entirely offline. She registers on Parachain A (the "Identity Chain") by submitting the commitment and nullifier with a ZK proof. The blockchain verifies she's unique without ever seeing her fingerprint. Her ZK proof cryptographically proves "I have a valid biometric" without revealing what it is. She's now registered and starts receiving 100 tokens monthly to her DID.

**Cross-Chain Usage**: Alice wants to vote in a DAO on Parachain B, use a lending protocol on Parachain C, and access healthcare records on Parachain D—all requiring proof she's a unique human without re-registering everywhere. Each parachain calls `verify_existence_proof()` with Alice's nullifier and a Merkle proof from the Identity Chain's state root (obtained via XCM). They instantly verify she's a registered unique human without storing redundant data or compromising privacy. She participates in governance (one person = one vote), borrows funds (personhood-gated rates), and accesses services—all with a single biometric registration.

**Recovery After Phone Loss**: Two years later, Alice loses her phone and can't access her fingerprint-derived nullifier. She initiates recovery by scanning her fingerprint on a new device, generating `new_nullifier = Hash(fingerprint)` (same biometric, same hash), and submitting a recovery request with her trusted guardians (family/friends). After a 6-month timelock and 2/3 guardian approval, the system replaces her old nullifier with the new one while preserving her DID and UBI payment history. The ZK recovery proof cryptographically links her old and new identities without exposing her biometric. She resumes receiving UBI payments seamlessly, and all parachains automatically recognize her new nullifier through updated Merkle proofs—no re-registration needed across dozens of services.