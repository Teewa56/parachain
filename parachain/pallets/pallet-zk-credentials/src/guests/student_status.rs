#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CredentialInput {
    institution: String,
    status: String,
    credential_hash: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct PrivateCredential {
    student_id: u64,
    gpa: f32,
    enrollment_date: u64,
}

#[derive(Serialize, Deserialize)]
struct VerificationOutput {
    is_valid: bool,
    revealed_status: String,
}

pub fn main() {
    let public_input = sp1_zkvm::io::read::<CredentialInput>();
    let private_cred = sp1_zkvm::io::read::<PrivateCredential>();

    // Simple hash computation (in production, use proper hashing)
    let mut data = Vec::new();
    data.extend_from_slice(&private_cred.student_id.to_be_bytes());
    data.extend_from_slice(&private_cred.gpa.to_be_bytes());
    data.extend_from_slice(&private_cred.enrollment_date.to_be_bytes());
    
    // Basic verification logic
    let is_valid = public_input.status == "Active"
        && private_cred.gpa >= 2.0;

    let output = VerificationOutput { 
        is_valid, 
        revealed_status: public_input.status 
    };
    
    sp1_zkvm::io::commit(&output);
}