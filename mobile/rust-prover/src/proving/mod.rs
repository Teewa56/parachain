use sp1_sdk::{ProverClient, SP1Stdin};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct ProofResult {
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub struct VerificationOutput {
    pub is_valid: bool,
}

// Generic prover
pub fn generate_sp1_proof(
    circuit_id: &str,
    public_input: Vec<u8>,
    private_input: Vec<u8>,
) -> Result<ProofResult, Box<dyn Error>> {
    let elf_bytes = match circuit_id {
        "age_verification" => include_bytes!("assets/age_verification.elf"),
        "student_status" => include_bytes!("assets/student_status.elf"),
        "vaccination_status" => include_bytes!("assets/vaccination_status.elf"),
        "employment_status" => include_bytes!("assets/employment_status.elf"),
        "custom" => include_bytes!("assets/custom.elf"),
        _ => return Err(format!("Unsupported circuit: {}", circuit_id).into()),
    };

    let mut stdin = SP1Stdin::new();
    stdin.write(&public_input);
    stdin.write(&private_input);

    let client = ProverClient::new();
    let (pk, _vk) = client.setup(elf_bytes);
    let proof = client.prove(&pk, stdin).run()?;

    let public_inputs = vec![public_input];

    Ok(ProofResult {
        proof_bytes: proof.bytes(),
        public_inputs,
    })
}

// Age proof
pub fn generate_age_proof(
    current_timestamp: u64,
    age_threshold_years: u64,
    credential_type_hash: &[u8; 32],
    birth_timestamp: u64,
    credential_hash: &[u8; 32],
    issuer_signature_hash: &[u8; 32],
    _proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    #[derive(Serialize)]
    struct Public { current_timestamp: u64, age_threshold_years: u64, credential_type_hash: [u8; 32] }
    #[derive(Serialize)]
    struct Private { birth_timestamp: u64, credential_hash: [u8; 32], issuer_signature_hash: [u8; 32] }

    let public_bytes = bincode::serialize(&Public { current_timestamp, age_threshold_years, credential_type_hash: *credential_type_hash })?;
    let private_bytes = bincode::serialize(&Private { birth_timestamp, credential_hash: *credential_hash, issuer_signature_hash: *issuer_signature_hash })?;

    generate_sp1_proof("age_verification", public_bytes, private_bytes)
}

// Student proof
pub fn generate_student_proof(
    current_timestamp: u64,
    institution_hash: &[u8; 32],
    status_active: bool,
    student_id_hash: &[u8; 32],
    enrollment_date: u64,
    expiry_date: u64,
    gpa: u16,
    credential_hash: &[u8; 32],
    issuer_signature_hash: &[u8; 32],
    _proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    #[derive(Serialize)]
    struct Public { current_timestamp: u64, institution_hash: [u8; 32], status_active: bool }
    #[derive(Serialize)]
    struct Private { student_id_hash: [u8; 32], enrollment_date: u64, expiry_date: u64, gpa: u16, credential_hash: [u8; 32], issuer_signature_hash: [u8; 32] }

    let public_bytes = bincode::serialize(&Public { current_timestamp, institution_hash: *institution_hash, status_active })?;
    let private_bytes = bincode::serialize(&Private { student_id_hash: *student_id_hash, enrollment_date, expiry_date, gpa, credential_hash: *credential_hash, issuer_signature_hash: *issuer_signature_hash })?;

    generate_sp1_proof("student_status", public_bytes, private_bytes)
}

// Vaccination proof
pub fn generate_vaccination_proof(
    current_timestamp: u64,
    vaccination_type_hash: &[u8; 32],
    min_doses_required: u8,
    patient_id_hash: &[u8; 32],
    vaccination_date: u64,
    expiry_date: u64,
    doses_received: u8,
    batch_number_hash: &[u8; 32],
    credential_hash: &[u8; 32],
    issuer_signature_hash: &[u8; 32],
    _proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    #[derive(Serialize)]
    struct Public { current_timestamp: u64, vaccination_type_hash: [u8; 32], min_doses_required: u8 }
    #[derive(Serialize)]
    struct Private { patient_id_hash: [u8; 32], vaccination_date: u64, expiry_date: u64, doses_received: u8, batch_number_hash: [u8; 32], credential_hash: [u8; 32], issuer_signature_hash: [u8; 32] }

    let public_bytes = bincode::serialize(&Public { current_timestamp, vaccination_type_hash: *vaccination_type_hash, min_doses_required })?;
    let private_bytes = bincode::serialize(&Private { patient_id_hash: *patient_id_hash, vaccination_date, expiry_date, doses_received, batch_number_hash: *batch_number_hash, credential_hash: *credential_hash, issuer_signature_hash: *issuer_signature_hash })?;

    generate_sp1_proof("vaccination_status", public_bytes, private_bytes)
}

// Employment proof
pub fn generate_employment_proof(
    current_timestamp: u64,
    company_hash: &[u8; 32],
    employment_type_hash: &[u8; 32],
    employee_id_hash: &[u8; 32],
    start_date: u64,
    end_date: u64,
    salary: u64,
    position_hash: &[u8; 32],
    credential_hash: &[u8; 32],
    issuer_signature_hash: &[u8; 32],
    _proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    #[derive(Serialize)]
    struct Public { current_timestamp: u64, company_hash: [u8; 32], employment_type_hash: [u8; 32] }
    #[derive(Serialize)]
    struct Private { employee_id_hash: [u8; 32], start_date: u64, end_date: u64, salary: u64, position_hash: [u8; 32], credential_hash: [u8; 32], issuer_signature_hash: [u8; 32] }

    let public_bytes = bincode::serialize(&Public { current_timestamp, company_hash: *company_hash, employment_type_hash: *employment_type_hash })?;
    let private_bytes = bincode::serialize(&Private { employee_id_hash: *employee_id_hash, start_date, end_date, salary, position_hash: *position_hash, credential_hash: *credential_hash, issuer_signature_hash: *issuer_signature_hash })?;

    generate_sp1_proof("employment_status", public_bytes, private_bytes)
}

// Custom proof - flexible, pass arbitrary bytes
pub fn generate_custom_proof(
    public_data: Vec<u8>,
    private_data: Vec<u8>,
    _proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    generate_sp1_proof("custom", public_data, private_data)
}

// Helper: Convert bytes to field bytes (if needed for legacy/custom)
fn bytes_to_field_bytes(bytes: &[u8; 32]) -> Vec<u8> {
    bytes.to_vec()
}