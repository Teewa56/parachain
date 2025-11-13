#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::H256;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_verification_key() {
        let vk_data = vec![0u8; 128];
        let registered_by = H256::from_low_u64_be(1);

        #[extrinsic_call]
        register_verification_key(
            RawOrigin::Root,
            ProofType::AgeAbove,
            vk_data.clone(),
            registered_by
        );

        assert!(VerifyingKeys::<T>::contains_key(&ProofType::AgeAbove));
    }

    #[benchmark]
    fn verify_proof() {
        let caller: T::AccountId = whitelisted_caller();
        
        // Setup: Register verification key first
        let vk_data = vec![0u8; 128];
        let registered_by = H256::from_low_u64_be(1);
        Pallet::<T>::register_verification_key(
            RawOrigin::Root.into(),
            ProofType::AgeAbove,
            vk_data,
            registered_by
        ).unwrap();
        
        // Create proof
        let proof = ZkProof {
            proof_type: ProofType::AgeAbove,
            proof_data: vec![0u8; 256],
            public_inputs: vec![
                21u32.to_le_bytes().to_vec(),
                2024u32.to_le_bytes().to_vec(),
            ],
            credential_hash: H256::from_low_u64_be(123),
            created_at: 1234567890,
            nonce: H256::from_low_u64_be(999),
        };

        #[extrinsic_call]
        verify_proof(RawOrigin::Signed(caller), proof.clone());

        let proof_hash = Pallet::<T>::hash_proof(&proof);
        assert!(VerifiedProofs::<T>::contains_key(&proof_hash));
    }

    #[benchmark]
    fn create_proof_schema() {
        let caller: T::AccountId = whitelisted_caller();
        
        let field_descriptions = vec![
            b"age_threshold".to_vec(),
            b"current_year".to_vec(),
        ];

        #[extrinsic_call]
        create_proof_schema(
            RawOrigin::Signed(caller),
            ProofType::AgeAbove,
            field_descriptions
        );

        assert!(ProofSchemas::<T>::contains_key(&ProofType::AgeAbove));
    }

    #[benchmark]
    fn batch_verify_proofs() {
        let caller: T::AccountId = whitelisted_caller();
        
        // Setup: Register verification key
        let vk_data = vec![0u8; 128];
        let registered_by = H256::from_low_u64_be(1);
        Pallet::<T>::register_verification_key(
            RawOrigin::Root.into(),
            ProofType::StudentStatus,
            vk_data,
            registered_by
        ).unwrap();
        
        // Create multiple proofs
        let mut proofs = Vec::new();
        for i in 0..3 {
            let proof = ZkProof {
                proof_type: ProofType::StudentStatus,
                proof_data: vec![i as u8; 256],
                public_inputs: vec![
                    H256::from_low_u64_be(i as u64).as_bytes().to_vec(),
                ],
                credential_hash: H256::from_low_u64_be(100 + i as u64),
                created_at: 1234567890 + i as u64,
                nonce: H256::from_low_u64_be(i as u64),
            };
            proofs.push(proof);
        }

        #[extrinsic_call]
        batch_verify_proofs(RawOrigin::Signed(caller), proofs.clone());

        // Verify all were stored
        for proof in proofs {
            let proof_hash = Pallet::<T>::hash_proof(&proof);
            assert!(VerifiedProofs::<T>::contains_key(&proof_hash));
        }
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}