use frame_support::weights::Weight;

pub trait WeightInfo {
    fn register_verification_key() -> Weight;
    fn verify_proof() -> Weight;
    fn create_proof_schema() -> Weight;
    fn batch_verify_proofs(n: u32) -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_verification_key() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn verify_proof() -> Weight {
        // ZK proof verification is computationally expensive
        Weight::from_parts(500_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn create_proof_schema() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn batch_verify_proofs(n: u32) -> Weight {
        // Base weight + per-proof weight
        Weight::from_parts(100_000_000, 0)
            .saturating_add(Weight::from_parts(400_000_000, 0).saturating_mul(n as u64))
            .saturating_add(T::DbWeight::get().reads(1 + n as u64))
            .saturating_add(T::DbWeight::get().writes(n as u64))
    }
}

impl WeightInfo for () {
    fn register_verification_key() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn verify_proof() -> Weight { Weight::from_parts(500_000_000, 0) }
    fn create_proof_schema() -> Weight { Weight::from_parts(35_000_000, 0) }
    fn batch_verify_proofs(n: u32) -> Weight { 
        Weight::from_parts(100_000_000 + (400_000_000 * n as u64), 0) 
    }
}