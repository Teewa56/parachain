#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_verifiable_credentials;
    use frame_support::{
        assert_ok, assert_noop, parameter_types,
        traits::{ConstU32, ConstU64, Time},
    };
    use frame_system as system;
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use pallet_identity_registry;

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
    type Block = frame_system::mocking::MockBlock<Test>;

    // Configure a mock runtime for testing
    frame_support::construct_runtime!(
        pub enum Test where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system,
            IdentityRegistry: pallet_identity_registry,
            VerifiableCredentials: pallet_verifiable_credentials,
            Timestamp: pallet_timestamp,
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
    }

    impl system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = BlockHashCount;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = ConstU32<16>;
    }

    impl pallet_timestamp::Config for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ConstU64<5>;
        type WeightInfo = ();
    }

    impl pallet_identity_registry::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type TimeProvider = Timestamp;
    }

    impl pallet_verifiable_credentials::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type TimeProvider = Timestamp;
    }

    // Test helpers
    fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    fn create_test_identity(account: u64, did: Vec<u8>) -> H256 {
        let public_key = H256::from_low_u64_be(account);
        assert_ok!(IdentityRegistry::create_identity(
            RuntimeOrigin::signed(account),
            did.clone(),
            public_key
        ));
        IdentityRegistry::hash_did(&did)
    }

    // Tests
    #[test]
    fn test_create_identity_works() {
        new_test_ext().execute_with(|| {
            let account = 1u64;
            let did = b"did:identity:alice".to_vec();
            let public_key = H256::from_low_u64_be(1);

            assert_ok!(IdentityRegistry::create_identity(
                RuntimeOrigin::signed(account),
                did.clone(),
                public_key
            ));

            let did_hash = IdentityRegistry::hash_did(&did);
            let identity = IdentityRegistry::identities(&did_hash).unwrap();

            assert_eq!(identity.controller, account);
            assert_eq!(identity.public_key, public_key);
            assert_eq!(identity.active, true);
        });
    }

    #[test]
    fn test_issue_credential_works() {
        new_test_ext().execute_with(|| {
            // Setup
            let issuer_account = 1u64;
            let subject_account = 2u64;

            // Create identities
            let issuer_did = create_test_identity(issuer_account, b"did:identity:university".to_vec());
            let subject_did = create_test_identity(subject_account, b"did:identity:student".to_vec());

            // Add issuer as trusted (needs root)
            assert_ok!(VerifiableCredentials::add_trusted_issuer(
                RuntimeOrigin::root(),
                CredentialType::Education,
                issuer_did
            ));

            // Issue credential
            let data_hash = H256::from_low_u64_be(123);
            let signature = H256::from_low_u64_be(456);
            let expires_at = 1735689600u64;

            assert_ok!(VerifiableCredentials::issue_credential(
                RuntimeOrigin::signed(issuer_account),
                subject_did,
                CredentialType::Education,
                data_hash,
                expires_at,
                signature
            ));

            // Verify credential was created
            let subject_creds = VerifiableCredentials::credentials_of(&subject_did);
            assert_eq!(subject_creds.len(), 1);
        });
    }

    #[test]
    fn test_revoke_credential_works() {
        new_test_ext().execute_with(|| {
            // Setup
            let issuer_account = 1u64;
            let subject_account = 2u64;

            let issuer_did = create_test_identity(issuer_account, b"did:identity:university".to_vec());
            let subject_did = create_test_identity(subject_account, b"did:identity:student".to_vec());

            // Add trusted issuer and issue credential
            assert_ok!(VerifiableCredentials::add_trusted_issuer(
                RuntimeOrigin::root(),
                CredentialType::Education,
                issuer_did
            ));

            let data_hash = H256::from_low_u64_be(123);
            let signature = H256::from_low_u64_be(456);

            assert_ok!(VerifiableCredentials::issue_credential(
                RuntimeOrigin::signed(issuer_account),
                subject_did,
                CredentialType::Education,
                data_hash,
                0,
                signature
            ));

            let subject_creds = VerifiableCredentials::credentials_of(&subject_did);
            let credential_id = subject_creds[0];

            // Revoke credential
            assert_ok!(VerifiableCredentials::revoke_credential(
                RuntimeOrigin::signed(issuer_account),
                credential_id
            ));

            // Verify credential is revoked
            let credential = VerifiableCredentials::credentials(&credential_id).unwrap();
            assert_eq!(credential.status, CredentialStatus::Revoked);
        });
    }

    #[test]
    fn test_verify_credential_fails_when_revoked() {
        new_test_ext().execute_with(|| {
            // Setup and issue credential
            let issuer_account = 1u64;
            let subject_account = 2u64;
            let verifier_account = 3u64;

            let issuer_did = create_test_identity(issuer_account, b"did:identity:university".to_vec());
            let subject_did = create_test_identity(subject_account, b"did:identity:student".to_vec());

            assert_ok!(VerifiableCredentials::add_trusted_issuer(
                RuntimeOrigin::root(),
                CredentialType::Education,
                issuer_did
            ));

            let data_hash = H256::from_low_u64_be(123);
            let signature = H256::from_low_u64_be(456);

            assert_ok!(VerifiableCredentials::issue_credential(
                RuntimeOrigin::signed(issuer_account),
                subject_did,
                CredentialType::Education,
                data_hash,
                0,
                signature
            ));

            let subject_creds = VerifiableCredentials::credentials_of(&subject_did);
            let credential_id = subject_creds[0];

            // Revoke credential
            assert_ok!(VerifiableCredentials::revoke_credential(
                RuntimeOrigin::signed(issuer_account),
                credential_id
            ));

            // Try to verify - should fail
            assert_noop!(
                VerifiableCredentials::verify_credential(
                    RuntimeOrigin::signed(verifier_account),
                    credential_id
                ),
                Error::<Test>::CredentialRevoked
            );
        });
    }

    #[test]
    fn test_untrusted_issuer_cannot_issue() {
        new_test_ext().execute_with(|| {
            let issuer_account = 1u64;
            let subject_account = 2u64;

            let issuer_did = create_test_identity(issuer_account, b"did:identity:university".to_vec());
            let subject_did = create_test_identity(subject_account, b"did:identity:student".to_vec());

            // Don't add as trusted issuer
            let data_hash = H256::from_low_u64_be(123);
            let signature = H256::from_low_u64_be(456);

            // Try to issue - should fail
            assert_noop!(
                VerifiableCredentials::issue_credential(
                    RuntimeOrigin::signed(issuer_account),
                    subject_did,
                    CredentialType::Education,
                    data_hash,
                    0,
                    signature
                ),
                Error::<Test>::IssuerNotTrusted
            );
        });
    }

    #[test]
    fn test_selective_disclosure() {
        new_test_ext().execute_with(|| {
            let issuer_account = 1u64;
            let subject_account = 2u64;

            let issuer_did = create_test_identity(issuer_account, b"did:identity:university".to_vec());
            let subject_did = create_test_identity(subject_account, b"did:identity:student".to_vec());

            assert_ok!(VerifiableCredentials::add_trusted_issuer(
                RuntimeOrigin::root(),
                CredentialType::Education,
                issuer_did
            ));

            let data_hash = H256::from_low_u64_be(123);
            let signature = H256::from_low_u64_be(456);

            assert_ok!(VerifiableCredentials::issue_credential(
                RuntimeOrigin::signed(issuer_account),
                subject_did,
                CredentialType::Education,
                data_hash,
                0,
                signature
            ));

            let subject_creds = VerifiableCredentials::credentials_of(&subject_did);
            let credential_id = subject_creds[0];

            // Perform selective disclosure
            let fields_to_reveal = vec![0, 2]; // Only reveal certain fields
            let proof = H256::from_low_u64_be(789);

            assert_ok!(VerifiableCredentials::selective_disclosure(
                RuntimeOrigin::signed(subject_account),
                credential_id,
                fields_to_reveal,
                proof
            ));
        });
    }

    #[test]
    fn test_create_credential_schema() {
        new_test_ext().execute_with(|| {
            let creator_account = 1u64;
            let creator_did = create_test_identity(creator_account, b"did:identity:university".to_vec());

            let fields = vec![
                b"institution".to_vec(),
                b"studentId".to_vec(),
                b"status".to_vec(),
                b"gpa".to_vec(),
            ];
            let required_fields = vec![true, true, true, false];

            assert_ok!(VerifiableCredentials::create_schema(
                RuntimeOrigin::signed(creator_account),
                CredentialType::Education,
                fields.clone(),
                required_fields.clone()
            ));

            // Check schema was created (would need to add getter for this)
        });
    }
}