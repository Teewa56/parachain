#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::H256;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_identity() {
        let caller: T::AccountId = whitelisted_caller();
        let did = b"did:identity:benchmark".to_vec();
        let public_key = H256::from_low_u64_be(1);

        #[extrinsic_call]
        create_identity(RawOrigin::Signed(caller), did, public_key);

        assert!(Identities::<T>::iter().count() > 0);
    }

    #[benchmark]
    fn update_identity() {
        let caller: T::AccountId = whitelisted_caller();
        let did = b"did:identity:benchmark".to_vec();
        let public_key = H256::from_low_u64_be(1);
        
        Pallet::<T>::create_identity(
            RawOrigin::Signed(caller.clone()).into(),
            did,
            public_key
        ).unwrap();

        let new_key = H256::from_low_u64_be(2);

        #[extrinsic_call]
        update_identity(RawOrigin::Signed(caller), new_key);
    }

    #[benchmark]
    fn deactivate_identity() {
        let caller: T::AccountId = whitelisted_caller();
        let did = b"did:identity:benchmark".to_vec();
        let public_key = H256::from_low_u64_be(1);
        
        Pallet::<T>::create_identity(
            RawOrigin::Signed(caller.clone()).into(),
            did,
            public_key
        ).unwrap();

        #[extrinsic_call]
        deactivate_identity(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn reactivate_identity() {
        let caller: T::AccountId = whitelisted_caller();
        let did = b"did:identity:benchmark".to_vec();
        let public_key = H256::from_low_u64_be(1);
        
        Pallet::<T>::create_identity(
            RawOrigin::Signed(caller.clone()).into(),
            did,
            public_key
        ).unwrap();
        
        Pallet::<T>::deactivate_identity(
            RawOrigin::Signed(caller.clone()).into()
        ).unwrap();

        #[extrinsic_call]
        reactivate_identity(RawOrigin::Signed(caller));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}