use sp1_sdk::{ProverClient, SP1Stdin, SP1Stdout};

#![no_main]
sp1_zkvm::entrypoint!(main);

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
    let public_input = sp1_zkvm::io::read::<CredentialInput>();
    let private_cred = sp1_zkvm::io::read::<PrivateCredential>();

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
    sp1_zkvm::io::commit(&output);
}

sp1_sdk::build_elf!(main);