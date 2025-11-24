use sp1_sdk::{ProverClient, SP1Stdin, SP1Stdout};

#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;
use blake2::Blake2s256;

#[derive(Serialize, Deserialize)]
struct CredentialInput {
    employer_hash: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct PrivateCredential {
    employee_id: Vec<u8>,
    employment_date: u64,
    salary_min: u64,
    salary_max: u64,
    is_active: bool,
}

#[derive(Serialize, Deserialize)]
struct VerificationOutput {
    is_valid: bool,
}

fn main() {
    let public_input = sp1_zkvm::io::read::<CredentialInput>();
    let private_cred = sp1_zkvm::io::read::<PrivateCredential>();

    // Hash employee ID (non-zero check)
    let mut hasher = Blake2s256::new();
    hasher.update(&private_cred.employee_id);
    let employee_hash = hasher.finalize();
    if employee_hash == [0u8; 32] {
        let output = VerificationOutput { is_valid: false };
        let mut stdout = SP1Stdout::new();
        bincode::serialize_into(&mut stdout, &output).unwrap();
        sp1_zkvm::io::commit(&output);
        return;
    }

    // Verification logic
    let is_valid = private_cred.is_active
        && public_input.employer_hash != [0u8; 32]  // Non-zero employer
        && private_cred.salary_min > 0
        && private_cred.employment_date < std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();  // Past date

    // Output
    let output = VerificationOutput { is_valid };
    let mut stdout = SP1Stdout::new();
    bincode::serialize_into(&mut stdout, &output).unwrap();
    sp1_zkvm::io::commit(&output);
}

sp1_sdk::build_elf!(main);