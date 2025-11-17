#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::H256;
use frame_support::traits::Currency;
use pallet_identity_registry;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_personhood() {
        let caller: T::AccountId = whitelisted_caller();
        
        // Create identity first
        let did = b"did:identity:person".to_vec();
        let pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(caller.clone()).into(),
            did.clone(),
            pk
        ).unwrap();
        
        let did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&did);
        
        // Give caller funds for deposit
        let deposit = T::RegistrationDeposit::get();
        T::Currency::make_free_balance_be(&caller, deposit * 2u32.into());
        
        let nullifier = H256::from_low_u64_be(999);
        let commitment = H256::from_low_u64_be(888);
        let proof = vec![1u8; 256];

        #[extrinsic_call]
        register_personhood(
            RawOrigin::Signed(caller),
            did_hash,
            nullifier,
            commitment,
            proof
        );

        assert!(PersonhoodRegistry::<T>::contains_key(&nullifier));
    }

    #[benchmark]
    fn request_recovery() {
        let caller: T::AccountId = whitelisted_caller();
        let guardian1: T::AccountId = account("guardian1", 0, 0);
        let guardian2: T::AccountId = account("guardian2", 0, 0);
        
        // Setup: Create identity and register personhood
        let did = b"did:identity:person".to_vec();
        let pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(caller.clone()).into(),
            did.clone(),
            pk
        ).unwrap();
        
        let did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&did);
        
        let deposit = T::RegistrationDeposit::get() + T::RecoveryDeposit::get();
        T::Currency::make_free_balance_be(&caller, deposit * 3u32.into());
        
        let old_nullifier = H256::from_low_u64_be(999);
        let old_commitment = H256::from_low_u64_be(888);
        let old_proof = vec![1u8; 256];
        
        Pallet::<T>::register_personhood(
            RawOrigin::Signed(caller.clone()).into(),
            did_hash,
            old_nullifier,
            old_commitment,
            old_proof
        ).unwrap();
        
        let new_nullifier = H256::from_low_u64_be(777);
        let new_commitment = H256::from_low_u64_be(666);
        let recovery_proof = vec![2u8; 256];
        let guardians = vec![guardian1, guardian2];

        #[extrinsic_call]
        request_recovery(
            RawOrigin::Signed(caller),
            did_hash,
            new_nullifier,
            new_commitment,
            recovery_proof,
            guardians
        );

        assert!(PendingRecoveries::<T>::contains_key(&did_hash));
    }

    #[benchmark]
    fn approve_recovery() {
        let requester: T::AccountId = account("requester", 0, 0);
        let guardian: T::AccountId = whitelisted_caller();
        
        // Setup
        let did = b"did:identity:person".to_vec();
        let pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(requester.clone()).into(),
            did.clone(),
            pk
        ).unwrap();
        
        let did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&did);
        
        let deposit = T::RegistrationDeposit::get() + T::RecoveryDeposit::get();
        T::Currency::make_free_balance_be(&requester, deposit * 3u32.into());
        
        let old_nullifier = H256::from_low_u64_be(999);
        let old_commitment = H256::from_low_u64_be(888);
        let old_proof = vec![1u8; 256];
        
        Pallet::<T>::register_personhood(
            RawOrigin::Signed(requester.clone()).into(),
            did_hash,
            old_nullifier,
            old_commitment,
            old_proof
        ).unwrap();
        
        let new_nullifier = H256::from_low_u64_be(777);
        let new_commitment = H256::from_low_u64_be(666);
        let recovery_proof = vec![2u8; 256];
        let guardians = vec![guardian.clone()];
        
        Pallet::<T>::request_recovery(
            RawOrigin::Signed(requester).into(),
            did_hash,
            new_nullifier,
            new_commitment,
            recovery_proof,
            guardians
        ).unwrap();

        #[extrinsic_call]
        approve_recovery(
            RawOrigin::Signed(guardian.clone()),
            did_hash
        );

        let approvals = GuardianApprovals::<T>::get(&did_hash);
        assert!(approvals.contains(&guardian));
    }

    #[benchmark]
    fn finalize_recovery() {
        let requester: T::AccountId = whitelisted_caller();
        let guardian1: T::AccountId = account("guardian1", 0, 0);
        let guardian2: T::AccountId = account("guardian2", 0, 0);
        
        // Setup
        let did = b"did:identity:person".to_vec();
        let pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(requester.clone()).into(),
            did.clone(),
            pk
        ).unwrap();
        
        let did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&did);
        
        let deposit = T::RegistrationDeposit::get() + T::RecoveryDeposit::get();
        T::Currency::make_free_balance_be(&requester, deposit * 3u32.into());
        
        let old_nullifier = H256::from_low_u64_be(999);
        let old_commitment = H256::from_low_u64_be(888);
        let old_proof = vec![1u8; 256];
        
        Pallet::<T>::register_personhood(
            RawOrigin::Signed(requester.clone()).into(),
            did_hash,
            old_nullifier,
            old_commitment,
            old_proof
        ).unwrap();
        
        let new_nullifier = H256::from_low_u64_be(777);
        let new_commitment = H256::from_low_u64_be(666);
        let recovery_proof = vec![2u8; 256];
        let guardians = vec![guardian1.clone(), guardian2.clone()];
        
        Pallet::<T>::request_recovery(
            RawOrigin::Signed(requester.clone()).into(),
            did_hash,
            new_nullifier,
            new_commitment,
            recovery_proof,
            guardians
        ).unwrap();
        
        // Approve from guardians
        Pallet::<T>::approve_recovery(
            RawOrigin::Signed(guardian1).into(),
            did_hash
        ).unwrap();
        
        Pallet::<T>::approve_recovery(
            RawOrigin::Signed(guardian2).into(),
            did_hash
        ).unwrap();
        
        // Fast-forward time
        let mut request = PendingRecoveries::<T>::get(&did_hash).unwrap();
        request.active_at = 0u64;
        PendingRecoveries::<T>::insert(&did_hash, request);

        #[extrinsic_call]
        finalize_recovery(
            RawOrigin::Signed(requester),
            did_hash
        );

        assert!(PersonhoodRegistry::<T>::contains_key(&new_nullifier));
        assert!(!PersonhoodRegistry::<T>::contains_key(&old_nullifier));
    }

    #[benchmark]
    fn cancel_recovery() {
        let requester: T::AccountId = whitelisted_caller();
        
        // Setup
        let did = b"did:identity:person".to_vec();
        let pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(requester.clone()).into(),
            did.clone(),
            pk
        ).unwrap();
        
        let did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&did);
        
        let deposit = T::RegistrationDeposit::get() + T::RecoveryDeposit::get();
        T::Currency::make_free_balance_be(&requester, deposit * 3u32.into());
        
        let old_nullifier = H256::from_low_u64_be(999);
        let old_commitment = H256::from_low_u64_be(888);
        let old_proof = vec![1u8; 256];
        
        Pallet::<T>::register_personhood(
            RawOrigin::Signed(requester.clone()).into(),
            did_hash,
            old_nullifier,
            old_commitment,
            old_proof
        ).unwrap();
        
        let new_nullifier = H256::from_low_u64_be(777);
        let new_commitment = H256::from_low_u64_be(666);
        let recovery_proof = vec![2u8; 256];
        let guardians = vec![account("guardian1", 0, 0)];
        
        Pallet::<T>::request_recovery(
            RawOrigin::Signed(requester.clone()).into(),
            did_hash,
            new_nullifier,
            new_commitment,
            recovery_proof,
            guardians
        ).unwrap();

        #[extrinsic_call]
        cancel_recovery(
            RawOrigin::Signed(requester),
            did_hash
        );

        assert!(!PendingRecoveries::<T>::contains_key(&did_hash));
    }

    #[benchmark]
    fn record_activity() {
        let caller: T::AccountId = whitelisted_caller();
        
        // Setup
        let did = b"did:identity:person".to_vec();
        let pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(caller.clone()).into(),
            did.clone(),
            pk
        ).unwrap();
        
        let did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&did);
        
        let deposit = T::RegistrationDeposit::get();
        T::Currency::make_free_balance_be(&caller, deposit * 2u32.into());
        
        let nullifier = H256::from_low_u64_be(999);
        let commitment = H256::from_low_u64_be(888);
        let proof = vec![1u8; 256];
        
        Pallet::<T>::register_personhood(
            RawOrigin::Signed(caller.clone()).into(),
            did_hash,
            nullifier,
            commitment,
            proof
        ).unwrap();

        #[extrinsic_call]
        record_activity(RawOrigin::Signed(caller));

        assert!(LastActivity::<T>::contains_key(&did_hash));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}