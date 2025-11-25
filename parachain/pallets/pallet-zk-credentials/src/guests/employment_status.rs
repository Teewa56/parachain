#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};

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

pub fn main() {
    let public_input = sp1_zkvm::io::read::<CredentialInput>();
    let private_cred = sp1_zkvm::io::read::<PrivateCredential>();

    // Verification logic
    let is_valid = private_cred.is_active
        && public_input.employer_hash != [0u8; 32]
        && private_cred.salary_min > 0
        && !private_cred.employee_id.is_empty();

    let output = VerificationOutput { is_valid };
    
    sp1_zkvm::io::commit(&output);
}