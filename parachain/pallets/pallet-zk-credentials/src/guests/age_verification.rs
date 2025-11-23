use sp1_sdk::{ProverClient, SP1Stdin, SP1Stdout};
use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;

#[derive(Serialize, Deserialize)]
struct CredentialInput {
    age_threshold: u32,
    current_year: u32,
}

#[derive(Serialize, Deserialize)]
struct PrivateCredential {
    birth_year: u32,
}

#[derive(Serialize, Deserialize)]
struct VerificationOutput {
    is_valid: bool,
}

fn main() {
    let stdin = SP1Stdin::new();
    let public_input: CredentialInput = bincode::deserialize(&stdin.read_public()).unwrap();
    let private_cred: PrivateCredential = bincode::deserialize(&stdin.read_private()).unwrap();

    // Verification logic: age >= threshold
    let age = public_input.current_year - private_cred.birth_year;
    let is_valid = age >= public_input.age_threshold;

    // Output
    let output = VerificationOutput { is_valid };
    let mut stdout = SP1Stdout::new();
    bincode::serialize_into(&mut stdout, &output).unwrap();
    stdout.flush();
}

sp1_sdk::build_elf!(main);