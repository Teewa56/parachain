use frame_support::weights::Weight;

pub trait WeightInfo {
    fn issue_credential() -> Weight;
    fn revoke_credential() -> Weight;
    fn verify_credential() -> Weight;
    fn create_schema() -> Weight;
    fn add_trusted_issuer() -> Weight;
    fn remove_trusted_issuer() -> Weight;
    fn selective_disclosure() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn issue_credential() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    
    fn revoke_credential() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn verify_credential() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn create_schema() -> Weight {
        Weight::from_parts(70_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn add_trusted_issuer() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn remove_trusted_issuer() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn selective_disclosure() -> Weight {
        Weight::from_parts(150_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

impl WeightInfo for () {
    fn issue_credential() -> Weight { Weight::from_parts(100_000_000, 0) }
    fn revoke_credential() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn verify_credential() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn create_schema() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn add_trusted_issuer() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn remove_trusted_issuer() -> Weight { Weight::from_parts(45_000_000, 0) }
    fn selective_disclosure() -> Weight { Weight::from_parts(150_000_000, 0) }
}