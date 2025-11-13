#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::H256;
use pallet_identity_registry;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn issue_credential() {
        let issuer: T::AccountId = whitelisted_caller();
        let subject: T::AccountId = account("subject", 0, 0);
        
        // Setup: Create issuer identity
        let issuer_did = b"did:identity:issuer".to_vec();
        let issuer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(issuer.clone()).into(),
            issuer_did.clone(),
            issuer_pk
        ).unwrap();
        
        let issuer_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&issuer_did);
        
        // Setup: Create subject identity
        let subject_did = b"did:identity:subject".to_vec();
        let subject_pk = H256::from_low_u64_be(2);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(subject.clone()).into(),
            subject_did.clone(),
            subject_pk
        ).unwrap();
        
        let subject_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&subject_did);
        
        // Setup: Add issuer as trusted
        TrustedIssuers::<T>::insert((&CredentialType::Education, &issuer_did_hash), true);
        
        let data_hash = H256::from_low_u64_be(123);
        let signature = H256::from_low_u64_be(456);
        let expires_at = 1735689600u64;

        #[extrinsic_call]
        issue_credential(
            RawOrigin::Signed(issuer),
            subject_did_hash,
            CredentialType::Education,
            data_hash,
            expires_at,
            signature
        );

        assert!(Credentials::<T>::iter().count() > 0);
    }

    #[benchmark]
    fn revoke_credential() {
        let issuer: T::AccountId = whitelisted_caller();
        let subject: T::AccountId = account("subject", 0, 0);
        
        // Setup: Create identities and issue credential
        let issuer_did = b"did:identity:issuer".to_vec();
        let issuer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(issuer.clone()).into(),
            issuer_did.clone(),
            issuer_pk
        ).unwrap();
        
        let issuer_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&issuer_did);
        
        let subject_did = b"did:identity:subject".to_vec();
        let subject_pk = H256::from_low_u64_be(2);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(subject.clone()).into(),
            subject_did.clone(),
            subject_pk
        ).unwrap();
        
        let subject_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&subject_did);
        
        TrustedIssuers::<T>::insert((&CredentialType::Education, &issuer_did_hash), true);
        
        Pallet::<T>::issue_credential(
            RawOrigin::Signed(issuer.clone()).into(),
            subject_did_hash,
            CredentialType::Education,
            H256::from_low_u64_be(123),
            0,
            H256::from_low_u64_be(456)
        ).unwrap();
        
        let creds = CredentialsOf::<T>::get(&subject_did_hash);
        let credential_id = creds[0];

        #[extrinsic_call]
        revoke_credential(RawOrigin::Signed(issuer), credential_id);

        let credential = Credentials::<T>::get(&credential_id).unwrap();
        assert_eq!(credential.status, CredentialStatus::Revoked);
    }

    #[benchmark]
    fn verify_credential() {
        let issuer: T::AccountId = account("issuer", 0, 0);
        let subject: T::AccountId = account("subject", 0, 0);
        let verifier: T::AccountId = whitelisted_caller();
        
        // Setup
        let issuer_did = b"did:identity:issuer".to_vec();
        let issuer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(issuer.clone()).into(),
            issuer_did.clone(),
            issuer_pk
        ).unwrap();
        
        let issuer_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&issuer_did);
        
        let subject_did = b"did:identity:subject".to_vec();
        let subject_pk = H256::from_low_u64_be(2);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(subject.clone()).into(),
            subject_did.clone(),
            subject_pk
        ).unwrap();
        
        let subject_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&subject_did);
        
        TrustedIssuers::<T>::insert((&CredentialType::Education, &issuer_did_hash), true);
        
        Pallet::<T>::issue_credential(
            RawOrigin::Signed(issuer.clone()).into(),
            subject_did_hash,
            CredentialType::Education,
            H256::from_low_u64_be(123),
            0,
            H256::from_low_u64_be(456)
        ).unwrap();
        
        let creds = CredentialsOf::<T>::get(&subject_did_hash);
        let credential_id = creds[0];

        #[extrinsic_call]
        verify_credential(RawOrigin::Signed(verifier), credential_id);
    }

    #[benchmark]
    fn create_schema() {
        let creator: T::AccountId = whitelisted_caller();
        
        // Setup: Create creator identity
        let creator_did = b"did:identity:creator".to_vec();
        let creator_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(creator.clone()).into(),
            creator_did.clone(),
            creator_pk
        ).unwrap();
        
        let fields = vec![
            b"institution".to_vec(),
            b"studentId".to_vec(),
            b"status".to_vec(),
            b"gpa".to_vec(),
        ];
        let required_fields = vec![true, true, true, false];

        #[extrinsic_call]
        create_schema(
            RawOrigin::Signed(creator),
            CredentialType::Education,
            fields,
            required_fields
        );

        assert!(Schemas::<T>::iter().count() > 0);
    }

    #[benchmark]
    fn add_trusted_issuer() {
        let issuer_account: T::AccountId = account("issuer", 0, 0);
        
        // Setup: Create issuer identity
        let issuer_did = b"did:identity:issuer".to_vec();
        let issuer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(issuer_account.clone()).into(),
            issuer_did.clone(),
            issuer_pk
        ).unwrap();
        
        let issuer_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&issuer_did);

        #[extrinsic_call]
        add_trusted_issuer(
            RawOrigin::Root,
            CredentialType::Education,
            issuer_did_hash
        );

        assert_eq!(TrustedIssuers::<T>::get((&CredentialType::Education, &issuer_did_hash)), true);
    }

    #[benchmark]
    fn remove_trusted_issuer() {
        let issuer_account: T::AccountId = account("issuer", 0, 0);
        
        // Setup
        let issuer_did = b"did:identity:issuer".to_vec();
        let issuer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(issuer_account.clone()).into(),
            issuer_did.clone(),
            issuer_pk
        ).unwrap();
        
        let issuer_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&issuer_did);
        TrustedIssuers::<T>::insert((&CredentialType::Education, &issuer_did_hash), true);

        #[extrinsic_call]
        remove_trusted_issuer(
            RawOrigin::Root,
            CredentialType::Education,
            issuer_did_hash
        );

        assert_eq!(TrustedIssuers::<T>::get((&CredentialType::Education, &issuer_did_hash)), false);
    }

    #[benchmark]
    fn selective_disclosure() {
        let issuer: T::AccountId = account("issuer", 0, 0);
        let subject: T::AccountId = whitelisted_caller();
        
        // Setup
        let issuer_did = b"did:identity:issuer".to_vec();
        let issuer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(issuer.clone()).into(),
            issuer_did.clone(),
            issuer_pk
        ).unwrap();
        
        let issuer_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&issuer_did);
        
        let subject_did = b"did:identity:subject".to_vec();
        let subject_pk = H256::from_low_u64_be(2);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(subject.clone()).into(),
            subject_did.clone(),
            subject_pk
        ).unwrap();
        
        let subject_did_hash = pallet_identity_registry::Pallet::<T>::hash_did(&subject_did);
        
        TrustedIssuers::<T>::insert((&CredentialType::Education, &issuer_did_hash), true);
        
        // Create schema first
        let fields = vec![
            b"field0".to_vec(),
            b"field1".to_vec(),
            b"field2".to_vec(),
        ];
        let required = vec![true, true, false];
        Pallet::<T>::create_schema(
            RawOrigin::Signed(issuer.clone()).into(),
            CredentialType::Education,
            fields,
            required
        ).unwrap();
        
        Pallet::<T>::issue_credential(
            RawOrigin::Signed(issuer.clone()).into(),
            subject_did_hash,
            CredentialType::Education,
            H256::from_low_u64_be(123),
            0,
            H256::from_low_u64_be(456)
        ).unwrap();
        
        let creds = CredentialsOf::<T>::get(&subject_did_hash);
        let credential_id = creds[0];
        
        let fields_to_reveal = vec![0, 1];
        let proof = H256::from_low_u64_be(789);

        #[extrinsic_call]
        selective_disclosure(
            RawOrigin::Signed(subject),
            credential_id,
            fields_to_reveal,
            proof
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}