use super::*;
use super::common::*;
use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Age verification circuit
/// Proves that: current_timestamp - birth_timestamp >= age_threshold
/// Without revealing the actual birth_timestamp
#[derive(Clone)]
pub struct AgeVerificationCircuit {
    // Public inputs
    pub current_timestamp: Option<Fr>,
    pub age_threshold_years: Option<Fr>,
    pub credential_type_hash: Option<Fr>,
    
    // Private inputs (witness)
    pub birth_timestamp: Option<Fr>,
    pub credential_hash: Option<Fr>,
    pub issuer_signature_hash: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for AgeVerificationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let current_timestamp = FpVar::new_input(cs.clone(), || {
            self.current_timestamp.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let age_threshold_years = FpVar::new_input(cs.clone(), || {
            self.age_threshold_years.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let credential_type_hash = FpVar::new_input(cs.clone(), || {
            self.credential_type_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let birth_timestamp = FpVar::new_witness(cs.clone(), || {
            self.birth_timestamp.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let credential_hash = FpVar::new_witness(cs.clone(), || {
            self.credential_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let issuer_signature_hash = FpVar::new_witness(cs.clone(), || {
            self.issuer_signature_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let max_age_seconds = 150 * 365 * 24 * 3600; // 150 years max
        enforce_timestamp_validity(
            cs.clone(),
            &birth_timestamp,
            &current_timestamp,
            max_age_seconds,
        )?;
        
        let age_seconds = &current_timestamp - &birth_timestamp;
        let seconds_per_year = FpVar::new_constant(cs.clone(), Fr::from(365 * 24 * 3600))?;
        let age_threshold_seconds = &age_threshold_years * &seconds_per_year;
        
        age_seconds.enforce_cmp(&age_threshold_seconds, std::cmp::Ordering::Greater, true)?;
        enforce_valid_hash(&credential_hash)?;
        enforce_valid_hash(&issuer_signature_hash)?;
        enforce_valid_hash(&credential_type_hash)?;
        
        Ok(())
    }
}

impl ProofCircuit<Fr> for AgeVerificationCircuit {
    type PublicInput = (u64, u64, [u8; 32]); // (current_timestamp, age_threshold_years, credential_type_hash)
    type PrivateInput = (u64, [u8; 32], [u8; 32]); // (birth_timestamp, credential_hash, issuer_sig_hash)
    
    fn new(public: Self::PublicInput, private: Self::PrivateInput) -> Self {
        Self {
            current_timestamp: Some(Fr::from(public.0)),
            age_threshold_years: Some(Fr::from(public.1)),
            credential_type_hash: Some(bytes_to_field(&public.2).unwrap()),
            birth_timestamp: Some(Fr::from(private.0)),
            credential_hash: Some(bytes_to_field(&private.1).unwrap()),
            issuer_signature_hash: Some(bytes_to_field(&private.2).unwrap()),
        }
    }
    
    fn circuit_id() -> &'static str {
        "age_verification"
    }
}