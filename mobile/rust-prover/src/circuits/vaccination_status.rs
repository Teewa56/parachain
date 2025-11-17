use super::*;
use super::common::*;
use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Vaccination status circuit
/// Proves vaccination without revealing patient ID or specific dates
#[derive(Clone)]
pub struct VaccinationStatusCircuit {
    // Public inputs
    pub current_timestamp: Option<Fr>,
    pub vaccination_type_hash: Option<Fr>,
    pub min_doses_required: Option<Fr>,
    
    // Private inputs
    pub patient_id_hash: Option<Fr>,
    pub vaccination_date: Option<Fr>,
    pub expiry_date: Option<Fr>,
    pub doses_received: Option<Fr>,
    pub batch_number_hash: Option<Fr>,
    pub credential_hash: Option<Fr>,
    pub issuer_signature_hash: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for VaccinationStatusCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Public inputs
        let current_timestamp = FpVar::new_input(cs.clone(), || {
            self.current_timestamp.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let vaccination_type_hash = FpVar::new_input(cs.clone(), || {
            self.vaccination_type_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let min_doses_required = FpVar::new_input(cs.clone(), || {
            self.min_doses_required.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Private inputs
        let patient_id_hash = FpVar::new_witness(cs.clone(), || {
            self.patient_id_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let vaccination_date = FpVar::new_witness(cs.clone(), || {
            self.vaccination_date.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let expiry_date = FpVar::new_witness(cs.clone(), || {
            self.expiry_date.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let doses_received = FpVar::new_witness(cs.clone(), || {
            self.doses_received.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let batch_number_hash = FpVar::new_witness(cs.clone(), || {
            self.batch_number_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let credential_hash = FpVar::new_witness(cs.clone(), || {
            self.credential_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let issuer_signature_hash = FpVar::new_witness(cs.clone(), || {
            self.issuer_signature_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Constraint 1: Vaccination date is valid
        enforce_timestamp_validity(
            cs.clone(),
            &vaccination_date,
            &current_timestamp,
            5 * 365 * 24 * 3600, // Max 5 years old
        )?;
        
        // Constraint 2: Not expired
        current_timestamp.enforce_cmp(&expiry_date, std::cmp::Ordering::Less, true)?;
        
        // Constraint 3: Sufficient doses received
        doses_received.enforce_cmp(&min_doses_required, std::cmp::Ordering::Greater, true)?;
        
        // Constraint 4: Doses in valid range (1-10)
        enforce_range(&doses_received, 1, 10)?;
        
        // Constraint 5: Valid hashes
        enforce_valid_hash(&patient_id_hash)?;
        enforce_valid_hash(&vaccination_type_hash)?;
        enforce_valid_hash(&batch_number_hash)?;
        enforce_valid_hash(&credential_hash)?;
        enforce_valid_hash(&issuer_signature_hash)?;
        
        Ok(())
    }
}

impl ProofCircuit<Fr> for VaccinationStatusCircuit {
    type PublicInput = (u64, [u8; 32], u8); // (current_timestamp, vaccination_type_hash, min_doses)
    type PrivateInput = (
        [u8; 32], // patient_id_hash
        u64,      // vaccination_date
        u64,      // expiry_date
        u8,       // doses_received
        [u8; 32], // batch_number_hash
        [u8; 32], // credential_hash
        [u8; 32], // issuer_signature_hash
    );
    
    fn new(public: Self::PublicInput, private: Self::PrivateInput) -> Self {
        Self {
            current_timestamp: Some(Fr::from(public.0)),
            vaccination_type_hash: Some(bytes_to_field(&public.1).unwrap()),
            min_doses_required: Some(Fr::from(public.2 as u64)),
            patient_id_hash: Some(bytes_to_field(&private.0).unwrap()),
            vaccination_date: Some(Fr::from(private.1)),
            expiry_date: Some(Fr::from(private.2)),
            doses_received: Some(Fr::from(private.3 as u64)),
            batch_number_hash: Some(bytes_to_field(&private.4).unwrap()),
            credential_hash: Some(bytes_to_field(&private.5).unwrap()),
            issuer_signature_hash: Some(bytes_to_field(&private.6).unwrap()),
        }
    }
    
    fn circuit_id() -> &'static str {
        "vaccination_status"
    }
}