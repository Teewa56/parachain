#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::H256;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_parachain() {
        let para_id = 2001u32;
        let trusted = true;

        #[extrinsic_call]
        register_parachain(
            RawOrigin::Root,
            para_id,
            trusted
        );

        assert!(RegisteredParachains::<T>::contains_key(para_id));
    }

    #[benchmark]
    fn request_cross_chain_verification() {
        let caller: T::AccountId = whitelisted_caller();
        
        // Setup: Register target parachain
        let target_para_id = 2001u32;
        Pallet::<T>::register_parachain(
            RawOrigin::Root.into(),
            target_para_id,
            true
        ).unwrap();
        
        let credential_hash = H256::from_low_u64_be(123);

        #[extrinsic_call]
        request_cross_chain_verification(
            RawOrigin::Signed(caller),
            credential_hash,
            target_para_id
        );

        assert!(PendingRequests::<T>::iter().count() > 0);
    }

    #[benchmark]
    fn export_credential() {
        let caller: T::AccountId = whitelisted_caller();
        
        // Setup: Register destination parachain
        let destination_para_id = 2001u32;
        Pallet::<T>::register_parachain(
            RawOrigin::Root.into(),
            destination_para_id,
            true
        ).unwrap();
        
        let credential_hash = H256::from_low_u64_be(123);
        let credential_data = vec![0u8; 256];

        #[extrinsic_call]
        export_credential(
            RawOrigin::Signed(caller),
            credential_hash,
            destination_para_id,
            credential_data
        );

        assert_eq!(ExportedCredentials::<T>::get(credential_hash, destination_para_id), true);
    }

    #[benchmark]
    fn import_credential() {
        // Setup: Register source parachain
        let source_para_id = 2002u32;
        Pallet::<T>::register_parachain(
            RawOrigin::Root.into(),
            source_para_id,
            true
        ).unwrap();
        
        let credential_hash = H256::from_low_u64_be(123);
        let credential_data = vec![0u8; 256];

        #[extrinsic_call]
        import_credential(
            RawOrigin::Root,
            source_para_id,
            credential_hash,
            credential_data.clone()
        );

        assert!(ImportedCredentials::<T>::contains_key(source_para_id, credential_hash));
    }

    #[benchmark]
    fn handle_verification_response() {
        let credential_hash = H256::from_low_u64_be(123);
        let is_valid = true;
        let metadata = vec![0u8; 64];

        #[extrinsic_call]
        handle_verification_response(
            RawOrigin::Root,
            credential_hash,
            is_valid,
            metadata
        );

        assert!(VerificationResults::<T>::get(&credential_hash).len() > 0);
    }

    #[benchmark]
    fn deregister_parachain() {
        let para_id = 2001u32;
        
        // Setup: Register parachain first
        Pallet::<T>::register_parachain(
            RawOrigin::Root.into(),
            para_id,
            true
        ).unwrap();

        #[extrinsic_call]
        deregister_parachain(
            RawOrigin::Root,
            para_id
        );

        assert!(!RegisteredParachains::<T>::contains_key(para_id));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}