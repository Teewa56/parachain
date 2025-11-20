use frame_support::weights::Weight;
use frame_support::traits::Get;

pub trait WeightInfo {
    fn register_personhood() -> Weight;
    fn request_recovery() -> Weight;
    fn approve_recovery() -> Weight;
    fn finalize_recovery() -> Weight;
    fn cancel_recovery() -> Weight;
    fn record_activity() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_personhood() -> Weight {
        Weight::from_parts(150_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    
    fn request_recovery() -> Weight {
        Weight::from_parts(120_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    
    fn approve_recovery() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn finalize_recovery() -> Weight {
        Weight::from_parts(180_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(6))
    }
    
    fn cancel_recovery() -> Weight {
        Weight::from_parts(70_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    
    fn record_activity() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn register_personhood() -> Weight { Weight::from_parts(150_000_000, 0) }
    fn request_recovery() -> Weight { Weight::from_parts(120_000_000, 0) }
    fn approve_recovery() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn finalize_recovery() -> Weight { Weight::from_parts(180_000_000, 0) }
    fn cancel_recovery() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn record_activity() -> Weight { Weight::from_parts(50_000_000, 0) }
}