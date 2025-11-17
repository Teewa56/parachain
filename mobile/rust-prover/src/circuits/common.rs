use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;

/// Enforce that a timestamp is valid (not in future, not too old)
pub fn enforce_timestamp_validity<F: PrimeField>(
    cs: impl Into<ark_relations::r1cs::Namespace<F>>,
    timestamp: &FpVar<F>,
    current_time: &FpVar<F>,
    max_age_seconds: u64,
) -> Result<(), SynthesisError> {
    let cs = cs.into().cs();
    
    // Ensure timestamp <= current_time
    timestamp.enforce_cmp(current_time, std::cmp::Ordering::Less, true)?;
    
    // Ensure timestamp >= current_time - max_age
    let max_age_var = FpVar::new_constant(cs, F::from(max_age_seconds))?;
    let min_valid_time = current_time - &max_age_var;
    timestamp.enforce_cmp(&min_valid_time, std::cmp::Ordering::Greater, true)?;
    
    Ok(())
}

/// Enforce that a hash is non-zero
pub fn enforce_valid_hash<F: PrimeField>(
    hash: &FpVar<F>,
) -> Result<(), SynthesisError> {
    hash.enforce_not_equal(&FpVar::zero())?;
    Ok(())
}

/// Enforce range check (value is within [min, max])
pub fn enforce_range<F: PrimeField>(
    value: &FpVar<F>,
    min: u64,
    max: u64,
) -> Result<(), SynthesisError> {
    let min_var = FpVar::Constant(F::from(min));
    let max_var = FpVar::Constant(F::from(max));
    
    value.enforce_cmp(&min_var, std::cmp::Ordering::Greater, true)?;
    value.enforce_cmp(&max_var, std::cmp::Ordering::Less, true)?;
    
    Ok(())
}