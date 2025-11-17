use ark_bn254::{Bn254, Fr};
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_std::rand::thread_rng;
use std::fs;
use std::path::Path;

use rust_prover::circuits::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Generating proving and verifying keys for all circuits...\n");

    let output_dir = Path::new("../assets/proving-keys");
    fs::create_dir_all(output_dir)?;

    // Generate keys for each circuit
    generate_age_keys(output_dir)?;
    generate_student_keys(output_dir)?;
    generate_vaccination_keys(output_dir)?;
    generate_employment_keys(output_dir)?;

    println!("\n✅ All keys generated successfully!");
    println!("📁 Keys saved to: {}", output_dir.display());
    
    Ok(())
}

fn generate_age_keys(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating age_verification keys...");
    
    // Create dummy circuit for setup
    let circuit = AgeVerificationCircuit {
        current_timestamp: Some(Fr::from(0u64)),
        age_threshold_years: Some(Fr::from(18u64)),
        credential_type_hash: Some(Fr::from(0u64)),
        birth_timestamp: Some(Fr::from(0u64)),
        credential_hash: Some(Fr::from(0u64)),
        issuer_signature_hash: Some(Fr::from(0u64)),
    };

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)?;

    // Serialize proving key
    let mut pk_bytes = Vec::new();
    pk.serialize_compressed(&mut pk_bytes)?;
    fs::write(output_dir.join("age_verification.pk"), &pk_bytes)?;
    println!("  ✓ Proving key: {} bytes", pk_bytes.len());

    // Serialize verifying key
    let mut vk_bytes = Vec::new();
    vk.serialize_compressed(&mut vk_bytes)?;
    fs::write(output_dir.join("age_verification.vk"), &vk_bytes)?;
    println!("  ✓ Verifying key: {} bytes", vk_bytes.len());

    Ok(())
}

fn generate_student_keys(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating student_status keys...");
    
    let circuit = StudentStatusCircuit {
        current_timestamp: Some(Fr::from(0u64)),
        institution_hash: Some(Fr::from(0u64)),
        status_active: Some(Fr::from(1u64)),
        student_id_hash: Some(Fr::from(0u64)),
        enrollment_date: Some(Fr::from(0u64)),
        expiry_date: Some(Fr::from(0u64)),
        gpa: Some(Fr::from(0u64)),
        credential_hash: Some(Fr::from(0u64)),
        issuer_signature_hash: Some(Fr::from(0u64)),
    };

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)?;

    let mut pk_bytes = Vec::new();
    pk.serialize_compressed(&mut pk_bytes)?;
    fs::write(output_dir.join("student_status.pk"), &pk_bytes)?;
    println!("  ✓ Proving key: {} bytes", pk_bytes.len());

    let mut vk_bytes = Vec::new();
    vk.serialize_compressed(&mut vk_bytes)?;
    fs::write(output_dir.join("student_status.vk"), &vk_bytes)?;
    println!("  ✓ Verifying key: {} bytes", vk_bytes.len());

    Ok(())
}

fn generate_vaccination_keys(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating vaccination_status keys...");
    
    let circuit = VaccinationStatusCircuit {
        current_timestamp: Some(Fr::from(0u64)),
        vaccination_type_hash: Some(Fr::from(0u64)),
        min_doses_required: Some(Fr::from(2u64)),
        patient_id_hash: Some(Fr::from(0u64)),
        vaccination_date: Some(Fr::from(0u64)),
        expiry_date: Some(Fr::from(0u64)),
        doses_received: Some(Fr::from(2u64)),
        batch_number_hash: Some(Fr::from(0u64)),
        credential_hash: Some(Fr::from(0u64)),
        issuer_signature_hash: Some(Fr::from(0u64)),
    };

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)?;

    let mut pk_bytes = Vec::new();
    pk.serialize_compressed(&mut pk_bytes)?;
    fs::write(output_dir.join("vaccination_status.pk"), &pk_bytes)?;
    println!("  ✓ Proving key: {} bytes", pk_bytes.len());

    let mut vk_bytes = Vec::new();
    vk.serialize_compressed(&mut vk_bytes)?;
    fs::write(output_dir.join("vaccination_status.vk"), &vk_bytes)?;
    println!("  ✓ Verifying key: {} bytes", vk_bytes.len());

    Ok(())
}

fn generate_employment_keys(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating employment_status keys...");
    
    let circuit = EmploymentStatusCircuit {
        current_timestamp: Some(Fr::from(0u64)),
        company_hash: Some(Fr::from(0u64)),
        employment_type_hash: Some(Fr::from(0u64)),
        employee_id_hash: Some(Fr::from(0u64)),
        start_date: Some(Fr::from(0u64)),
        end_date: Some(Fr::from(0u64)),
        salary: Some(Fr::from(50000u64)),
        position_hash: Some(Fr::from(0u64)),
        credential_hash: Some(Fr::from(0u64)),
        issuer_signature_hash: Some(Fr::from(0u64)),
    };

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)?;

    let mut pk_bytes = Vec::new();
    pk.serialize_compressed(&mut pk_bytes)?;
    fs::write(output_dir.join("employment_status.pk"), &pk_bytes)?;
    println!("  ✓ Proving key: {} bytes", pk_bytes.len());

    let mut vk_bytes = Vec::new();
    vk.serialize_compressed(&mut vk_bytes)?;
    fs::write(output_dir.join("employment_status.vk"), &vk_bytes)?;
    println!("  ✓ Verifying key: {} bytes", vk_bytes.len());

    Ok(())
}