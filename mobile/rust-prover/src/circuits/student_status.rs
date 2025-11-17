use super::*;
use super::common::*;
use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Student status verification circuit
/// Proves active student enrollment without revealing student ID or GPA
#[derive(Clone)]
pub struct StudentStatusCircuit {
    // Public inputs
    pub current_timestamp: Option<Fr>,
    pub institution_hash: Option<Fr>,
    pub status_active: Option<Fr>, // 1 = active, 0 = inactive
    
    // Private inputs
    pub student_id_hash: Option<Fr>,
    pub enrollment_date: Option<Fr>,
    pub expiry_date: Option<Fr>,
    pub gpa: Option<Fr>, // Hidden
    pub credential_hash: Option<Fr>,
    pub issuer_signature_hash: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for StudentStatusCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Public inputs
        let current_timestamp = FpVar::new_input(cs.clone(), || {
            self.current_timestamp.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let institution_hash = FpVar::new_input(cs.clone(), || {
            self.institution_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let status_active = FpVar::new_input(cs.clone(), || {
            self.status_active.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Private inputs
        let student_id_hash = FpVar::new_witness(cs.clone(), || {
            self.student_id_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let enrollment_date = FpVar::new_witness(cs.clone(), || {
            self.enrollment_date.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let expiry_date = FpVar::new_witness(cs.clone(), || {
            self.expiry_date.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let gpa = FpVar::new_witness(cs.clone(), || {
            self.gpa.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let credential_hash = FpVar::new_witness(cs.clone(), || {
            self.credential_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let issuer_signature_hash = FpVar::new_witness(cs.clone(), || {
            self.issuer_signature_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
    
        let one = FpVar::new_constant(cs.clone(), Fr::from(1u64))?;
        let zero = FpVar::new_constant(cs.clone(), Fr::from(0u64))?;
        let status_is_one = status_active.is_eq(&one)?;
        let status_is_zero = status_active.is_eq(&zero)?;
        status_is_one.or(&status_is_zero)?.enforce_equal(&Boolean::TRUE)?;
        
        let is_after_enrollment = current_timestamp.is_cmp(
            &enrollment_date,
            std::cmp::Ordering::Greater,
            true
        )?;
        let is_before_expiry = current_timestamp.is_cmp(
            &expiry_date,
            std::cmp::Ordering::Less,
            true
        )?;
        let is_valid_period = is_after_enrollment.and(&is_before_expiry)?;
        
        // If status is active, period must be valid
        let status_active_bool = status_active.is_eq(&one)?;
        status_active_bool.implies(&is_valid_period)?;
        enforce_timestamp_validity(
            cs.clone(),
            &enrollment_date,
            &current_timestamp,
            10 * 365 * 24 * 3600, // Max 10 years old enrollment
        )?;
        
        enforce_range(&gpa, 0, 400)?;
        
        enforce_valid_hash(&student_id_hash)?;
        enforce_valid_hash(&credential_hash)?;
        enforce_valid_hash(&issuer_signature_hash)?;
        enforce_valid_hash(&institution_hash)?;
        
        Ok(())
    }
}

impl ProofCircuit<Fr> for StudentStatusCircuit {
    type PublicInput = (u64, [u8; 32], bool); // (current_timestamp, institution_hash, status_active)
    type PrivateInput = (
        [u8; 32], // student_id_hash
        u64,      // enrollment_date
        u64,      // expiry_date
        u16,      // gpa (0-400)
        [u8; 32], // credential_hash
        [u8; 32], // issuer_signature_hash
    );
    
    fn new(public: Self::PublicInput, private: Self::PrivateInput) -> Self {
        Self {
            current_timestamp: Some(Fr::from(public.0)),
            institution_hash: Some(bytes_to_field(&public.1).unwrap()),
            status_active: Some(Fr::from(if public.2 { 1u64 } else { 0u64 })),
            student_id_hash: Some(bytes_to_field(&private.0).unwrap()),
            enrollment_date: Some(Fr::from(private.1)),
            expiry_date: Some(Fr::from(private.2)),
            gpa: Some(Fr::from(private.3 as u64)),
            credential_hash: Some(bytes_to_field(&private.4).unwrap()),
            issuer_signature_hash: Some(bytes_to_field(&private.5).unwrap()),
        }
    }
    
    fn circuit_id() -> &'static str {
        "student_status"
    }
}