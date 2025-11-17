use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

pub mod age_verification;
pub mod student_status;
pub mod vaccination_status;
pub mod employment_status;
pub mod custom;
pub mod common;

pub use age_verification::AgeVerificationCircuit;
pub use student_status::StudentStatusCircuit;
pub use vaccination_status::VaccinationStatusCircuit;
pub use employment_status::EmploymentStatusCircuit;
pub use custom::CustomCircuit;

/// Circuit trait that all circuits must implement
pub trait ProofCircuit<F: PrimeField>: ConstraintSynthesizer<F> {
    type PublicInput;
    type PrivateInput;
    
    fn new(public: Self::PublicInput, private: Self::PrivateInput) -> Self;
    fn circuit_id() -> &'static str;
}

/// Convert bytes to field element
pub fn bytes_to_field<F: PrimeField>(bytes: &[u8]) -> Result<F, SynthesisError> {
    F::from_be_bytes_mod_order(bytes)
}

/// Convert u64 to field element
pub fn u64_to_field<F: PrimeField>(value: u64) -> F {
    F::from(value)
}