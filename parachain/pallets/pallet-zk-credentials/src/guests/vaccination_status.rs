use sp1_sdk::{ProverClient, SP1Stdin, SP1Stdout};

#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;
use blake2::Blake2s256;

#[derive(Serialize, Deserialize)]
struct CredentialInput {
    expiry_date: u64,
    vaccination_type_hash: [u8; 32],  // Public hash
}

#[derive(Serialize, Deserialize)]
struct PrivateCredential {
    patient_id: Vec<u8>,
    vaccination_date: u64,
    is_valid: bool,
    issuer_pub_key_hash: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct VerificationOutput {
    is_valid: bool,
}

fn main() {

    let public_input = sp1_zkvm::io::read::<CredentialInput>();
    let private_cred = sp1_zkvm::io::read::<PrivateCredential>();

    // Hash patient ID (non-zero check)
    let mut hasher = Blake2s256::new();
    hasher.update(&private_cred.patient_id);
    let patient_hash = hasher.finalize();
    if patient_hash == [0u8; 32] {
        // Invalid (zero hash)
        let output = VerificationOutput { is_valid: false };
        let mut stdout = SP1Stdout::new();
        bincode::serialize_into(&mut stdout, &output).unwrap();
        sp1_zkvm::io::commit(&output);
        return;
    }

    // Verification logic
    let is_valid = private_cred.is_valid
        && private_cred.vaccination_date < public_input.expiry_date
        && private_cred.issuer_pub_key_hash != [0u8; 32];  // Non-zero issuer

    // Output
    let output = VerificationOutput { is_valid };
    let mut stdout = SP1Stdout::new();
    bincode::serialize_into(&mut stdout, &output).unwrap();
    sp1_zkvm::io::commit(&output);
}

sp1_sdk::build_elf!(main);