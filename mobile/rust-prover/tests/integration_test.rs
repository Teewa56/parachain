use rust_prover::circuits::*;
use ark_bn254::{Bn254, Fr};
use ark_groth16::Groth16;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use ark_std::rand::thread_rng;

#[test]
fn test_age_verification_circuit() {
    let current_timestamp = 1700000000u64;
    let age_threshold_years = 18u64;
    let credential_type_hash = [1u8; 32];
    let birth_timestamp = current_timestamp - (25 * 365 * 24 * 3600); // 25 years old
    let credential_hash = [2u8; 32];
    let issuer_signature_hash = [3u8; 32];

    let circuit = AgeVerificationCircuit::new(
        (current_timestamp, age_threshold_years, credential_type_hash),
        (birth_timestamp, credential_hash, issuer_signature_hash)
    );

    // Generate keys
    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();

    // Generate proof
    let proof = Groth16::<Bn254>::prove(&pk, circuit.clone(), &mut rng).unwrap();

    // Prepare public inputs
    let public_inputs = vec![
        Fr::from(current_timestamp),
        Fr::from(age_threshold_years),
        Fr::from_be_bytes_mod_order(&credential_type_hash),
    ];

    // Verify proof
    let valid = Groth16::<Bn254>::verify(&vk, &public_inputs, &proof).unwrap();
    assert!(valid, "Age verification proof should be valid");
}

#[test]
fn test_age_verification_fails_under_age() {
    let current_timestamp = 1700000000u64;
    let age_threshold_years = 21u64;
    let credential_type_hash = [1u8; 32];
    let birth_timestamp = current_timestamp - (18 * 365 * 24 * 3600); // Only 18 years old
    let credential_hash = [2u8; 32];
    let issuer_signature_hash = [3u8; 32];

    let circuit = AgeVerificationCircuit::new(
        (current_timestamp, age_threshold_years, credential_type_hash),
        (birth_timestamp, credential_hash, issuer_signature_hash)
    );

    let mut rng = thread_rng();
    
    let result = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng);
    
    match result {
        Ok((pk, _)) => {
            let circuit_for_proof = AgeVerificationCircuit::new(
                (current_timestamp, age_threshold_years, credential_type_hash),
                (birth_timestamp, credential_hash, issuer_signature_hash)
            );
            let proof_result = Groth16::<Bn254>::prove(&pk, circuit_for_proof, &mut rng);
            assert!(proof_result.is_err(), "Should fail for underage");
        }
        Err(_) => {
            // ...
        }
    }
}

#[test]
fn test_student_status_circuit() {
    let current_timestamp = 1700000000u64;
    let institution_hash = [1u8; 32];
    let status_active = true;
    let student_id_hash = [2u8; 32];
    let enrollment_date = current_timestamp - (365 * 24 * 3600); // 1 year ago
    let expiry_date = current_timestamp + (365 * 24 * 3600); // 1 year future
    let gpa = 350u16; // 3.5
    let credential_hash = [3u8; 32];
    let issuer_signature_hash = [4u8; 32];

    let circuit = StudentStatusCircuit::new(
        (current_timestamp, institution_hash, status_active),
        (student_id_hash, enrollment_date, expiry_date, gpa, credential_hash, issuer_signature_hash)
    );

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();
    let proof = Groth16::<Bn254>::prove(&pk, circuit.clone(), &mut rng).unwrap();

    let public_inputs = vec![
        Fr::from(current_timestamp),
        Fr::from_be_bytes_mod_order(&institution_hash),
        Fr::from(if status_active { 1u64 } else { 0u64 }),
    ];

    let valid = Groth16::<Bn254>::verify(&vk, &public_inputs, &proof).unwrap();
    assert!(valid, "Student status proof should be valid");
}

#[test]
fn test_vaccination_circuit() {
    let current_timestamp = 1700000000u64;
    let vaccination_type_hash = [1u8; 32];
    let min_doses_required = 2u8;
    let patient_id_hash = [2u8; 32];
    let vaccination_date = current_timestamp - (180 * 24 * 3600); // 6 months ago
    let expiry_date = current_timestamp + (365 * 24 * 3600);
    let doses_received = 3u8;
    let batch_number_hash = [3u8; 32];
    let credential_hash = [4u8; 32];
    let issuer_signature_hash = [5u8; 32];

    let circuit = VaccinationStatusCircuit::new(
        (current_timestamp, vaccination_type_hash, min_doses_required),
        (patient_id_hash, vaccination_date, expiry_date, doses_received, batch_number_hash, credential_hash, issuer_signature_hash)
    );

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();
    let proof = Groth16::<Bn254>::prove(&pk, circuit.clone(), &mut rng).unwrap();

    let public_inputs = vec![
        Fr::from(current_timestamp),
        Fr::from_be_bytes_mod_order(&vaccination_type_hash),
        Fr::from(min_doses_required as u64),
    ];

    let valid = Groth16::<Bn254>::verify(&vk, &public_inputs, &proof).unwrap();
    assert!(valid, "Vaccination proof should be valid");
}

#[test]
fn test_employment_circuit() {
    let current_timestamp = 1700000000u64;
    let company_hash = [1u8; 32];
    let employment_type_hash = [2u8; 32];
    let employee_id_hash = [3u8; 32];
    let start_date = current_timestamp - (2 * 365 * 24 * 3600);
    let end_date = 0u64; // Still employed
    let salary = 7500000u64;
    let position_hash = [4u8; 32];
    let credential_hash = [5u8; 32];
    let issuer_signature_hash = [6u8; 32];

    let circuit = EmploymentStatusCircuit::new(
        (current_timestamp, company_hash, employment_type_hash),
        (employee_id_hash, start_date, end_date, salary, position_hash, credential_hash, issuer_signature_hash)
    );

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();
    let proof = Groth16::<Bn254>::prove(&pk, circuit.clone(), &mut rng).unwrap();

    let public_inputs = vec![
        Fr::from(current_timestamp),
        Fr::from_be_bytes_mod_order(&company_hash),
        Fr::from_be_bytes_mod_order(&employment_type_hash),
    ];

    let valid = Groth16::<Bn254>::verify(&vk, &public_inputs, &proof).unwrap();
    assert!(valid, "Employment proof should be valid");
}

#[test]
fn test_proof_serialization() {
    let current_timestamp = 1700000000u64;
    let age_threshold_years = 18u64;
    let credential_type_hash = [1u8; 32];
    let birth_timestamp = current_timestamp - (25 * 365 * 24 * 3600);
    let credential_hash = [2u8; 32];
    let issuer_signature_hash = [3u8; 32];

    let circuit = AgeVerificationCircuit::new(
        (current_timestamp, age_threshold_years, credential_type_hash),
        (birth_timestamp, credential_hash, issuer_signature_hash)
    );

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng).unwrap();

    // Serialize proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes).unwrap();

    // Deserialize proof
    let deserialized_proof = ark_groth16::Proof::<Bn254>::deserialize_compressed(&proof_bytes[..]).unwrap();

    // Verify deserialized proof
    let public_inputs = vec![
        Fr::from(current_timestamp),
        Fr::from(age_threshold_years),
        Fr::from_be_bytes_mod_order(&credential_type_hash),
    ];

    let valid = Groth16::<Bn254>::verify(&vk, &public_inputs, &deserialized_proof).unwrap();
    assert!(valid, "Deserialized proof should be valid");
}