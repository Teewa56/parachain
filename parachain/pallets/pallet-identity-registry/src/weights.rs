use frame_support::weights::Weight;

pub trait WeightInfo {
    fn create_identity() -> Weight;
    fn update_identity() -> Weight;
    fn deactivate_identity() -> Weight;
    fn reactivate_identity() -> Weight;
    fn update_did_document() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_identity() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn update_identity() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn deactivate_identity() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn reactivate_identity() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn update_did_document() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn create_identity() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn update_identity() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn deactivate_identity() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn reactivate_identity() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn update_did_document() -> Weight { Weight::from_parts(35_000_000, 0) }
}