use frame_support::weights::Weight;
use frame_support::traits::Get;

pub trait WeightInfo {
    fn register_personhood() -> Weight;
    fn request_recovery() -> Weight;
    fn approve_recovery() -> Weight;
    fn finalize_recovery() -> Weight;
    fn cancel_recovery() -> Weight;
    fn record_activity() -> Weight;
    fn add_guardian() -> Weight;
    fn initiate_progressive_recovery() -> Weight;
    fn finalize_progressive_recovery() -> Weight;
    fn submit_recovery_evidence() -> Weight;
    fn challenge_recovery() -> Weight;
    fn record_behavioral_pattern() -> Weight;
    fn register_primary_personhood() -> Weight;
    fn bind_additional_biometrics() -> Weight;
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

    fn add_guardian() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn initiate_progressive_recovery() -> Weight {
        Weight::from_parts(130_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn finalize_progressive_recovery() -> Weight {
        Weight::from_parts(160_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    fn submit_recovery_evidence() -> Weight {
        Weight::from_parts(90_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn challenge_recovery() -> Weight {
        Weight::from_parts(110_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn record_behavioral_pattern() -> Weight {
        Weight::from_parts(70_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn register_primary_personhood() -> Weight {
        Weight::from_parts(140_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn bind_additional_biometrics() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

impl WeightInfo for () {
    fn register_personhood() -> Weight { Weight::from_parts(150_000_000, 0) }
    fn request_recovery() -> Weight { Weight::from_parts(120_000_000, 0) }
    fn approve_recovery() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn finalize_recovery() -> Weight { Weight::from_parts(180_000_000, 0) }
    fn cancel_recovery() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn record_activity() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn add_guardian() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn initiate_progressive_recovery() -> Weight { Weight::from_parts(130_000_000, 0) }
    fn finalize_progressive_recovery() -> Weight { Weight::from_parts(160_000_000, 0) }
    fn submit_recovery_evidence() -> Weight { Weight::from_parts(90_000_000, 0) }
    fn challenge_recovery() -> Weight { Weight::from_parts(110_000_000, 0) }
    fn record_behavioral_pattern() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn register_primary_personhood() -> Weight { Weight::from_parts(140_000_000, 0) }
    fn bind_additional_biometrics() -> Weight { Weight::from_parts(100_000_000, 0) }
}