#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};

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

pub fn main() {
    // Read public input
    let public_input = sp1_zkvm::io::read::<CredentialInput>();
    
    // Read private credential
    let private_cred = sp1_zkvm::io::read::<PrivateCredential>();

    // Verification logic: age >= threshold
    let age = public_input.current_year.saturating_sub(private_cred.birth_year);
    let is_valid = age >= public_input.age_threshold;

    // Output
    let output = VerificationOutput { is_valid };
    
    // Commit the result
    sp1_zkvm::io::commit(&output);
}