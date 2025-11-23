use sp1_sdk::{ProverClient, SP1Stdin, SP1Stdout};
use serde::{Deserialize, Serialize};
use blake2::Blake2s256;
use std::time::UNIX_EPOCH;

#[derive(Serialize, Deserialize)]
struct CredentialInput {
    commitment: [u8; 32],  // Public commitment
}

#[derive(Serialize, Deserialize)]
struct PrivateCredential {
    biometric_hash: [u8; 32],
    salt: [u8; 32],
    did: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct VerificationOutput {
    is_valid: bool,
    nullifier: [u8; 32],
}

fn main() {
    let stdin = SP1Stdin::new();
    let public_input: CredentialInput = bincode::deserialize(&stdin.read_public()).unwrap();
    let private_cred: PrivateCredential = bincode::deserialize(&stdin.read_private()).unwrap();

    // Generate nullifier from biometric
    let mut hasher = Blake2s256::new();
    hasher.update(&private_cred.biometric_hash);
    let nullifier = hasher.finalize();

    // Verify commitment: Hash(biometric + salt) == commitment
    hasher = Blake2s256::new();
    hasher.update(&private_cred.biometric_hash);
    hasher.update(&private_cred.salt);
    let computed_commitment = hasher.finalize();

    let is_valid = computed_commitment == public_input.commitment
        && !private_cred.did.is_empty()  // Valid DID
        && private_cred.salt != [0u8; 32];  // Non-zero salt

    // Output
    let output = VerificationOutput { is_valid, nullifier };
    let mut stdout = SP1Stdout::new();
    bincode::serialize_into(&mut stdout, &output).unwrap();
    stdout.flush();
}

sp1_sdk::build_elf!(main);