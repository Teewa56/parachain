use frame_support::weights::Weight;

pub trait WeightInfo {
    fn propose_add_issuer() -> Weight;
    fn vote() -> Weight;
    fn finalize_proposal() -> Weight;
    fn add_council_member() -> Weight;
    fn remove_council_member() -> Weight;
    fn emergency_remove_issuer() -> Weight;
    fn cancel_proposal() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_add_issuer() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    
    fn vote() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    
    fn finalize_proposal() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    
    fn add_council_member() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn remove_council_member() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn emergency_remove_issuer() -> Weight {
        Weight::from_parts(70_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(10)) // Multiple credential types
    }
    
    fn cancel_proposal() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

impl WeightInfo for () {
    fn propose_add_issuer() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn vote() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn finalize_proposal() -> Weight { Weight::from_parts(100_000_000, 0) }
    fn add_council_member() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn remove_council_member() -> Weight { Weight::from_parts(35_000_000, 0) }
    fn emergency_remove_issuer() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn cancel_proposal() -> Weight { Weight::from_parts(50_000_000, 0) }
}