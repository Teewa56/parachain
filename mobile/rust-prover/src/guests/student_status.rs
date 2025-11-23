use sp1_sdk::{ProverClient, SP1Stdin, SP1Stdout};
use serde::{Deserialize, Serialize};
use blake2::Blake2s256;
use std::time::UNIX_EPOCH;

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

fn main() {
    let stdin = SP1Stdin::new();
    let public_input: CredentialInput = bincode::deserialize(&stdin.read_public()).unwrap();
    let private_cred: PrivateCredential = bincode::deserialize(&stdin.read_private()).unwrap();

    // Hash computation
    let mut hasher = Blake2s256::new();
    hasher.update(&private_cred.student_id.to_be_bytes());
    hasher.update(private_cred.gpa.to_be_bytes());
    hasher.update(&private_cred.enrollment_date.to_be_bytes());
    let computed_hash = hasher.finalize();

    // Verification logic
    let is_valid = computed_hash == public_input.credential_hash
        && public_input.status == "Active"
        && private_cred.gpa >= 2.0
        && (std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - private_cred.enrollment_date) < 157680000;  // <5 years

    // Output
    let output = VerificationOutput { is_valid, revealed_status: public_input.status };
    let mut stdout = SP1Stdout::new();
    bincode::serialize_into(&mut stdout, &output).unwrap();
    stdout.flush();
}

sp1_sdk::build_elf!(main);