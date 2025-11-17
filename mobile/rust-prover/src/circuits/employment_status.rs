use super::*;
use super::common::*;
use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Employment status circuit
/// Proves employment without revealing salary or employee ID
#[derive(Clone)]
pub struct EmploymentStatusCircuit {
    // Public inputs
    pub current_timestamp: Option<Fr>,
    pub company_hash: Option<Fr>,
    pub employment_type_hash: Option<Fr>, // Full-time, part-time, etc.
    
    // Private inputs
    pub employee_id_hash: Option<Fr>,
    pub start_date: Option<Fr>,
    pub end_date: Option<Fr>, // 0 if still employed
    pub salary: Option<Fr>, // Hidden
    pub position_hash: Option<Fr>,
    pub credential_hash: Option<Fr>,
    pub issuer_signature_hash: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for EmploymentStatusCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Public inputs
        let current_timestamp = FpVar::new_input(cs.clone(), || {
            self.current_timestamp.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let company_hash = FpVar::new_input(cs.clone(), || {
            self.company_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let employment_type_hash = FpVar::new_input(cs.clone(), || {
            self.employment_type_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // Private inputs
        let employee_id_hash = FpVar::new_witness(cs.clone(), || {
            self.employee_id_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let start_date = FpVar::new_witness(cs.clone(), || {
            self.start_date.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let end_date = FpVar::new_witness(cs.clone(), || {
            self.end_date.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let salary = FpVar::new_witness(cs.clone(), || {
            self.salary.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let position_hash = FpVar::new_witness(cs.clone(), || {
            self.position_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let credential_hash = FpVar::new_witness(cs.clone(), || {
            self.credential_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let issuer_signature_hash = FpVar::new_witness(cs.clone(), || {
            self.issuer_signature_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let zero = FpVar::new_constant(cs.clone(), Fr::from(0u64))?;
        
        // Constraint 1: Start date is valid
        enforce_timestamp_validity(
            cs.clone(),
            &start_date,
            &current_timestamp,
            50 * 365 * 24 * 3600, // Max 50 years old
        )?;
        
        // Constraint 2: Employment period logic
        let still_employed = end_date.is_eq(&zero)?;
        
        // If end_date > 0, check validity
        let end_date_valid = end_date.is_cmp(&start_date, std::cmp::Ordering::Greater, false)?;
        let end_date_not_future = end_date.is_cmp(&current_timestamp, std::cmp::Ordering::Less, true)?;
        let end_date_checks = end_date_valid.and(&end_date_not_future)?;
        
        // If not still employed, end_date must pass checks
        still_employed.not().implies(&end_date_checks)?;
        
        // If still employed, current time must be after start
        still_employed.implies(&current_timestamp.is_cmp(&start_date, std::cmp::Ordering::Greater, false)?)?;
        
        // Constraint 3: Salary is reasonable (non-zero if employed)
        let salary_positive = salary.is_cmp(&zero, std::cmp::Ordering::Greater, false)?;
        salary_positive.enforce_equal(&Boolean::TRUE)?;
        
        // Constraint 4: Valid hashes
        enforce_valid_hash(&employee_id_hash)?;
        enforce_valid_hash(&company_hash)?;
        enforce_valid_hash(&employment_type_hash)?;
        enforce_valid_hash(&position_hash)?;
        enforce_valid_hash(&credential_hash)?;
        enforce_valid_hash(&issuer_signature_hash)?;
        
        Ok(())
    }
}

impl ProofCircuit<Fr> for EmploymentStatusCircuit {
    type PublicInput = (u64, [u8; 32], [u8; 32]); // (current_timestamp, company_hash, employment_type_hash)
    type PrivateInput = (
        [u8; 32], // employee_id_hash
        u64,      // start_date
        u64,      // end_date (0 if still employed)
        u64,      // salary (annual in cents)
        [u8; 32], // position_hash
        [u8; 32], // credential_hash
        [u8; 32], // issuer_signature_hash
    );
    
    fn new(public: Self::PublicInput, private: Self::PrivateInput) -> Self {
        Self {
            current_timestamp: Some(Fr::from(public.0)),
            company_hash: Some(bytes_to_field(&public.1).unwrap()),
            employment_type_hash: Some(bytes_to_field(&public.2).unwrap()),
            employee_id_hash: Some(bytes_to_field(&private.0).unwrap()),
            start_date: Some(Fr::from(private.1)),
            end_date: Some(Fr::from(private.2)),
            salary: Some(Fr::from(private.3)),
            position_hash: Some(bytes_to_field(&private.4).unwrap()),
            credential_hash: Some(bytes_to_field(&private.5).unwrap()),
            issuer_signature_hash: Some(bytes_to_field(&private.6).unwrap()),
        }
    }
    
    fn circuit_id() -> &'static str {
        "employment_status"
    }
}