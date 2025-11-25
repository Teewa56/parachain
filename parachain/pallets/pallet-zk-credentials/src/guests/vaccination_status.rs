#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CredentialInput {
    expiry_date: u64,
    vaccination_type_hash: [u8; 32],
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

pub fn main() {
    let public_input = sp1_zkvm::io::read::<CredentialInput>();
    let private_cred = sp1_zkvm::io::read::<PrivateCredential>();

    // Verification logic
    let is_valid = private_cred.is_valid
        && private_cred.vaccination_date < public_input.expiry_date
        && private_cred.issuer_pub_key_hash != [0u8; 32]
        && !private_cred.patient_id.is_empty();

    let output = VerificationOutput { is_valid };
    
    sp1_zkvm::io::commit(&output);
}