use sp1_sdk::{ProverClient, SP1Stdin, SP1Stdout};
use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;

#[derive(Serialize, Deserialize)]
struct CredentialInput {
    current_timestamp: u64,
    age_threshold_years: u64,
    credential_type_hash: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct PrivateCredential {
    birth_timestamp: u64,
    credential_hash: [u8; 32],
    issuer_signature_hash: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct VerificationOutput {
    is_valid: bool,
}

fn main() {
    let stdin = SP1Stdin::new();
    let public_input: CredentialInput = bincode::deserialize(&stdin.read_public()).unwrap();
    let private_cred: PrivateCredential = bincode::deserialize(&stdin.read_private()).unwrap();

    // Logic: age >= threshold
    let age_seconds = public_input.current_timestamp - private_cred.birth_timestamp;
    let seconds_per_year = 365 * 24 * 3600;
    let age_threshold_seconds = public_input.age_threshold_years * seconds_per_year;
    let is_valid = age_seconds >= age_threshold_seconds
        && private_cred.birth_timestamp < public_input.current_timestamp  // Valid timestamp
        && private_cred.credential_hash != [0u8; 32]  // Non-zero hash
        && private_cred.issuer_signature_hash != [0u8; 32]  // Non-zero sig
        && public_input.credential_type_hash != [0u8; 32];  // Non-zero type

    // Output
    let output = VerificationOutput { is_valid };
    let mut stdout = SP1Stdout::new();
    bincode::serialize_into(&mut stdout, &output).unwrap();
    stdout.flush();
}

sp1_sdk::build_elf!(main);