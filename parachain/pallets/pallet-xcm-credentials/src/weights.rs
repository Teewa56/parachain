use frame_support::weights::Weight;

pub trait WeightInfo {
    fn register_parachain() -> Weight;
    fn request_cross_chain_verification() -> Weight;
    fn export_credential() -> Weight;
    fn import_credential() -> Weight;
    fn handle_verification_response() -> Weight;
    fn deregister_parachain() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_parachain() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn request_cross_chain_verification() -> Weight {
        // Includes XCM message sending overhead
        Weight::from_parts(200_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    
    fn export_credential() -> Weight {
        // Includes XCM message sending overhead
        Weight::from_parts(250_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    
    fn import_credential() -> Weight {
        Weight::from_parts(70_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn handle_verification_response() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn deregister_parachain() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn register_parachain() -> Weight { Weight::from_parts(45_000_000, 0) }
    fn request_cross_chain_verification() -> Weight { Weight::from_parts(200_000_000, 0) }
    fn export_credential() -> Weight { Weight::from_parts(250_000_000, 0) }
    fn import_credential() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn handle_verification_response() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn deregister_parachain() -> Weight { Weight::from_parts(40_000_000, 0) }
}