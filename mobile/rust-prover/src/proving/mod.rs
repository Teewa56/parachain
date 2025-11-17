use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof, ProvingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::thread_rng;
use std::error::Error;
use zeroize::Zeroize;

use crate::circuits::*;

pub struct ProofResult {
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<Vec<u8>>,
}

/// Generate age verification proof
pub fn generate_age_proof(
    current_timestamp: u64,
    age_threshold_years: u64,
    credential_type_hash: &[u8; 32],
    birth_timestamp: u64,
    credential_hash: &[u8; 32],
    issuer_signature_hash: &[u8; 32],
    proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    // Deserialize proving key
    let pk = ProvingKey::<Bn254>::deserialize_compressed(proving_key_bytes)?;
    
    // Create circuit
    let circuit = AgeVerificationCircuit::new(
        (current_timestamp, age_threshold_years, *credential_type_hash),
        (birth_timestamp, *credential_hash, *issuer_signature_hash)
    );
    
    // Generate proof
    let mut rng = thread_rng();
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng)?;
    
    // Serialize proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)?;
    
    // Serialize public inputs
    let public_inputs = vec![
        Fr::from(current_timestamp).into_bigint().to_bytes_be(),
        Fr::from(age_threshold_years).into_bigint().to_bytes_be(),
        bytes_to_field_bytes(credential_type_hash),
    ];
    
    Ok(ProofResult {
        proof_bytes,
        public_inputs,
    })
}

/// Generate student status proof
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
    proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    // Deserialize proving key
    let pk = ProvingKey::<Bn254>::deserialize_compressed(proving_key_bytes)?;
    
    // Create circuit
    let circuit = StudentStatusCircuit::new(
        (current_timestamp, *institution_hash, status_active),
        (
            *student_id_hash,
            enrollment_date,
            expiry_date,
            gpa,
            *credential_hash,
            *issuer_signature_hash,
        )
    );
    
    // Generate proof
    let mut rng = thread_rng();
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng)?;
    
    // Serialize proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)?;
    
    // Serialize public inputs
    let public_inputs = vec![
        Fr::from(current_timestamp).into_bigint().to_bytes_be(),
        bytes_to_field_bytes(institution_hash),
        Fr::from(if status_active { 1u64 } else { 0u64 }).into_bigint().to_bytes_be(),
    ];
    
    Ok(ProofResult {
        proof_bytes,
        public_inputs,
    })
}

/// Generate vaccination status proof
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
    proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    // Deserialize proving key
    let pk = ProvingKey::<Bn254>::deserialize_compressed(proving_key_bytes)?;
    
    // Create circuit
    let circuit = VaccinationStatusCircuit::new(
        (current_timestamp, *vaccination_type_hash, min_doses_required),
        (
            *patient_id_hash,
            vaccination_date,
            expiry_date,
            doses_received,
            *batch_number_hash,
            *credential_hash,
            *issuer_signature_hash,
        )
    );
    
    // Generate proof
    let mut rng = thread_rng();
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng)?;
    
    // Serialize proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)?;
    
    // Serialize public inputs
    let public_inputs = vec![
        Fr::from(current_timestamp).into_bigint().to_bytes_be(),
        bytes_to_field_bytes(vaccination_type_hash),
        Fr::from(min_doses_required as u64).into_bigint().to_bytes_be(),
    ];
    
    Ok(ProofResult {
        proof_bytes,
        public_inputs,
    })
}

/// Generate employment status proof
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
    proving_key_bytes: &[u8],
) -> Result<ProofResult, Box<dyn Error>> {
    // Deserialize proving key
    let pk = ProvingKey::<Bn254>::deserialize_compressed(proving_key_bytes)?;
    
    // Create circuit
    let circuit = EmploymentStatusCircuit::new(
        (current_timestamp, *company_hash, *employment_type_hash),
        (
            *employee_id_hash,
            start_date,
            end_date,
            salary,
            *position_hash,
            *credential_hash,
            *issuer_signature_hash,
        )
    );
    
    // Generate proof
    let mut rng = thread_rng();
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng)?;
    
    // Serialize proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)?;
    
    // Serialize public inputs
    let public_inputs = vec![
        Fr::from(current_timestamp).into_bigint().to_bytes_be(),
        bytes_to_field_bytes(company_hash),
        bytes_to_field_bytes(employment_type_hash),
    ];
    
    Ok(ProofResult {
        proof_bytes,
        public_inputs,
    })
}

/// Helper: Convert 32-byte hash to field element bytes
fn bytes_to_field_bytes(bytes: &[u8; 32]) -> Vec<u8> {
    Fr::from_be_bytes_mod_order(bytes).into_bigint().to_bytes_be()
}