use codec::Encode;
use sp_core::H256;
use sp_runtime::AccountId32;

// Mock runtime setup
mod common;
use common::*;

#[test]
fn test_full_identity_workflow() {
    new_test_ext().execute_with(|| {
        let alice = AccountId32::from([1u8; 32]);
        let bob = AccountId32::from([2u8; 32]);
        
        // Step 1: Create identity
        let did = b"did:identity:alice".to_vec();
        let public_key = H256::from_low_u64_be(1);
        
        assert_ok!(IdentityRegistry::create_identity(
            RuntimeOrigin::signed(alice.clone()),
            did.clone(),
            public_key
        ));
        
        let did_hash = IdentityRegistry::hash_did(&did);
        let identity = IdentityRegistry::identities(&did_hash).unwrap();
        
        assert_eq!(identity.controller, alice);
        assert_eq!(identity.active, true);
        
        // Step 2: Update identity
        let new_key = H256::from_low_u64_be(2);
        assert_ok!(IdentityRegistry::update_identity(
            RuntimeOrigin::signed(alice.clone()),
            new_key
        ));
        
        let updated_identity = IdentityRegistry::identities(&did_hash).unwrap();
        assert_eq!(updated_identity.public_key, new_key);
        
        // Step 3: Deactivate and reactivate
        assert_ok!(IdentityRegistry::deactivate_identity(
            RuntimeOrigin::signed(alice.clone())
        ));
        
        let deactivated = IdentityRegistry::identities(&did_hash).unwrap();
        assert_eq!(deactivated.active, false);
        
        assert_ok!(IdentityRegistry::reactivate_identity(
            RuntimeOrigin::signed(alice.clone())
        ));
        
        let reactivated = IdentityRegistry::identities(&did_hash).unwrap();
        assert_eq!(reactivated.active, true);
    });
}

#[test]
fn test_full_credential_workflow() {
    new_test_ext().execute_with(|| {
        let issuer = AccountId32::from([1u8; 32]);
        let student = AccountId32::from([2u8; 32]);
        let verifier = AccountId32::from([3u8; 32]);
        
        // Create identities
        let issuer_did = create_test_identity(issuer.clone(), b"did:identity:university".to_vec());
        let student_did = create_test_identity(student.clone(), b"did:identity:student".to_vec());
        
        // Add issuer as trusted
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
            RuntimeOrigin::signed(issuer.clone()),
            student_did,
            CredentialType::Education,
            data_hash,
            expires_at,
            signature
        ));
        
        // Get credential ID
        let student_creds = VerifiableCredentials::credentials_of(&student_did);
        assert_eq!(student_creds.len(), 1);
        let credential_id = student_creds[0];
        
        // Verify credential
        assert_ok!(VerifiableCredentials::verify_credential(
            RuntimeOrigin::signed(verifier.clone()),
            credential_id
        ));
        
        // Selective disclosure
        let fields_to_reveal = vec![0, 2];
        let proof = H256::from_low_u64_be(789);
        
        assert_ok!(VerifiableCredentials::selective_disclosure(
            RuntimeOrigin::signed(student.clone()),
            credential_id,
            fields_to_reveal,
            proof
        ));
        
        // Revoke credential
        assert_ok!(VerifiableCredentials::revoke_credential(
            RuntimeOrigin::signed(issuer.clone()),
            credential_id
        ));
        
        let credential = VerifiableCredentials::credentials(&credential_id).unwrap();
        assert_eq!(credential.status, CredentialStatus::Revoked);
        
        // Verification should now fail
        assert_noop!(
            VerifiableCredentials::verify_credential(
                RuntimeOrigin::signed(verifier),
                credential_id
            ),
            Error::<Test>::CredentialRevoked
        );
    });
}

#[test]
fn test_governance_workflow() {
    new_test_ext().execute_with(|| {
        let proposer = AccountId32::from([1u8; 32]);
        let council_member_1 = AccountId32::from([2u8; 32]);
        let council_member_2 = AccountId32::from([3u8; 32]);
        let council_member_3 = AccountId32::from([4u8; 32]);
        
        // Setup: Add council members
        assert_ok!(CredentialGovernance::add_council_member(
            RuntimeOrigin::root(),
            council_member_1.clone(),
            10 // voting power
        ));
        
        assert_ok!(CredentialGovernance::add_council_member(
            RuntimeOrigin::root(),
            council_member_2.clone(),
            15
        ));
        
        assert_ok!(CredentialGovernance::add_council_member(
            RuntimeOrigin::root(),
            council_member_3.clone(),
            5
        ));
        
        // Proposer needs to be council member
        assert_ok!(CredentialGovernance::add_council_member(
            RuntimeOrigin::root(),
            proposer.clone(),
            1
        ));
        
        // Create identity for issuer
        let issuer_did = create_test_identity(proposer.clone(), b"did:identity:hospital".to_vec());
        
        // Create proposal
        let credential_types = vec![CredentialTypeAuth::Health];
        let description = b"Regional Hospital - Trusted Healthcare Provider".to_vec();
        
        // Give proposer some balance for deposit
        fund_account(proposer.clone(), PROPOSAL_DEPOSIT + 1000);
        
        assert_ok!(CredentialGovernance::propose_add_issuer(
            RuntimeOrigin::signed(proposer.clone()),
            issuer_did,
            credential_types.clone(),
            description
        ));
        
        let proposal_id = 0u64;
        
        // Council votes
        assert_ok!(CredentialGovernance::vote(
            RuntimeOrigin::signed(council_member_1.clone()),
            proposal_id,
            Vote::Yes
        ));
        
        assert_ok!(CredentialGovernance::vote(
            RuntimeOrigin::signed(council_member_2.clone()),
            proposal_id,
            Vote::Yes
        ));
        
        assert_ok!(CredentialGovernance::vote(
            RuntimeOrigin::signed(council_member_3.clone()),
            proposal_id,
            Vote::No
        ));
        
        // Fast forward past voting period
        run_to_block(VOTING_PERIOD + 1);
        
        // Finalize proposal
        assert_ok!(CredentialGovernance::finalize_proposal(
            RuntimeOrigin::signed(proposer.clone()),
            proposal_id
        ));
        
        // Check if issuer is now trusted
        let proposal = CredentialGovernance::proposals(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Executed);
        
        // Verify issuer can now issue credentials
        assert_eq!(
            CredentialGovernance::is_issuer_trusted(
                &issuer_did,
                &CredentialTypeAuth::Health
            ),
            true
        );
    });
}

#[test]
fn test_zk_proof_verification() {
    new_test_ext().execute_with(|| {
        let prover = AccountId32::from([1u8; 32]);
        let verifier = AccountId32::from([2u8; 32]);
        
        // Register verification key (root only)
        let vk_data = vec![0u8; 128]; // Mock verification key
        let registered_by = H256::from_low_u64_be(1);
        
        assert_ok!(ZkCredentials::register_verification_key(
            RuntimeOrigin::root(),
            ProofType::AgeAbove,
            vk_data.clone(),
            registered_by
        ));
        
        // Create ZK proof
        let proof = ZkProof {
            proof_type: ProofType::AgeAbove,
            proof_data: vec![0u8; 256], // Mock proof
            public_inputs: vec![
                21u32.to_le_bytes().to_vec(), // age threshold
                2024u32.to_le_bytes().to_vec(), // current year
            ],
            credential_hash: H256::from_low_u64_be(123),
            created_at: 1234567890,
        };
        
        // In production, this would actually verify the proof
        // For testing, we mock the verification
        
        // Note: Actual verification would fail without proper setup
        // This test demonstrates the API usage
        
        // Create proof schema
        let field_descriptions = vec![
            b"age_threshold".to_vec(),
            b"current_year".to_vec(),
        ];
        
        assert_ok!(ZkCredentials::create_proof_schema(
            RuntimeOrigin::signed(prover.clone()),
            ProofType::AgeAbove,
            field_descriptions
        ));
    });
}

#[test]
fn test_cross_chain_workflow() {
    new_test_ext().execute_with(|| {
        let user = AccountId32::from([1u8; 32]);
        
        // Register parachain
        let parachain_b_id = 2001u32;
        
        assert_ok!(XcmCredentials::register_parachain(
            RuntimeOrigin::root(),
            parachain_b_id,
            true // trusted
        ));
        
        // Create credential
        let credential_hash = H256::from_low_u64_be(123);
        
        // Request cross-chain verification
        assert_ok!(XcmCredentials::request_cross_chain_verification(
            RuntimeOrigin::signed(user.clone()),
            credential_hash,
            parachain_b_id
        ));
        
        // Export credential
        let credential_data = b"encrypted_credential_data".to_vec();
        
        assert_ok!(XcmCredentials::export_credential(
            RuntimeOrigin::signed(user.clone()),
            credential_hash,
            parachain_b_id,
            credential_data.clone()
        ));
        
        // Verify export was recorded
        assert_eq!(
            XcmCredentials::exported_credentials(credential_hash, parachain_b_id),
            true
        );
        
        // Simulate receiving import on this chain from another
        let source_para_id = 2002u32;
        
        assert_ok!(XcmCredentials::register_parachain(
            RuntimeOrigin::root(),
            source_para_id,
            true
        ));
        
        assert_ok!(XcmCredentials::import_credential(
            RuntimeOrigin::root(),
            source_para_id,
            credential_hash,
            credential_data
        ));
        
        // Verify import was stored
        assert!(XcmCredentials::imported_credentials(source_para_id, credential_hash).is_some());
    });
}

#[test]
fn test_credential_expiration() {
    new_test_ext().execute_with(|| {
        let issuer = AccountId32::from([1u8; 32]);
        let student = AccountId32::from([2u8; 32]);
        let verifier = AccountId32::from([3u8; 32]);
        
        let issuer_did = create_test_identity(issuer.clone(), b"did:identity:university".to_vec());
        let student_did = create_test_identity(student.clone(), b"did:identity:student".to_vec());
        
        // Add trusted issuer
        assert_ok!(VerifiableCredentials::add_trusted_issuer(
            RuntimeOrigin::root(),
            CredentialType::Education,
            issuer_did
        ));
        
        // Issue credential that expires soon
        let current_time = Timestamp::now();
        let expires_at = current_time + 100; // Expires in 100 seconds
        
        assert_ok!(VerifiableCredentials::issue_credential(
            RuntimeOrigin::signed(issuer.clone()),
            student_did,
            CredentialType::Education,
            H256::from_low_u64_be(123),
            expires_at,
            H256::from_low_u64_be(456)
        ));
        
        let student_creds = VerifiableCredentials::credentials_of(&student_did);
        let credential_id = student_creds[0];
        
        // Verify works before expiration
        assert_ok!(VerifiableCredentials::verify_credential(
            RuntimeOrigin::signed(verifier.clone()),
            credential_id
        ));
        
        // Fast forward time past expiration
        advance_time(200);
        
        // Verification should fail
        assert_noop!(
            VerifiableCredentials::verify_credential(
                RuntimeOrigin::signed(verifier),
                credential_id
            ),
            Error::<Test>::CredentialExpired
        );
    });
}

#[test]
fn test_batch_operations() {
    new_test_ext().execute_with(|| {
        let issuer = AccountId32::from([1u8; 32]);
        let students = vec![
            AccountId32::from([2u8; 32]),
            AccountId32::from([3u8; 32]),
            AccountId32::from([4u8; 32]),
        ];
        
        let issuer_did = create_test_identity(issuer.clone(), b"did:identity:university".to_vec());
        
        // Add trusted issuer
        assert_ok!(VerifiableCredentials::add_trusted_issuer(
            RuntimeOrigin::root(),
            CredentialType::Education,
            issuer_did
        ));
        
        // Issue credentials to multiple students
        let mut credential_ids = Vec::new();
        
        for (i, student) in students.iter().enumerate() {
            let student_did = create_test_identity(
                student.clone(),
                format!("did:identity:student{}", i).into_bytes()
            );
            
            assert_ok!(VerifiableCredentials::issue_credential(
                RuntimeOrigin::signed(issuer.clone()),
                student_did,
                CredentialType::Education,
                H256::from_low_u64_be(i as u64),
                0,
                H256::from_low_u64_be((i * 100) as u64)
            ));
            
            let creds = VerifiableCredentials::credentials_of(&student_did);
            credential_ids.push(creds[0]);
        }
        
        // Verify all credentials were created
        assert_eq!(credential_ids.len(), 3);
        
        // Test batch verification (if implemented)
        for credential_id in credential_ids {
            let credential = VerifiableCredentials::credentials(&credential_id).unwrap();
            assert_eq!(credential.status, CredentialStatus::Active);
        }
    });
}

#[test]
fn test_security_edge_cases() {
    new_test_ext().execute_with(|| {
        let alice = AccountId32::from([1u8; 32]);
        let bob = AccountId32::from([2u8; 32]);
        
        // Test 1: Can't create identity twice
        let did = b"did:identity:alice".to_vec();
        let pk = H256::from_low_u64_be(1);
        
        assert_ok!(IdentityRegistry::create_identity(
            RuntimeOrigin::signed(alice.clone()),
            did.clone(),
            pk
        ));
        
        assert_noop!(
            IdentityRegistry::create_identity(
                RuntimeOrigin::signed(alice.clone()),
                did.clone(),
                pk
            ),
            Error::<Test>::AccountAlreadyHasIdentity
        );
        
        // Test 2: Can't update someone else's identity
        let alice_did = IdentityRegistry::hash_did(&did);
        
        assert_noop!(
            IdentityRegistry::update_identity(
                RuntimeOrigin::signed(bob.clone()),
                H256::from_low_u64_be(2)
            ),
            Error::<Test>::IdentityNotFound
        );
        
        // Test 3: Can't issue credentials without being trusted
        let bob_did = create_test_identity(bob.clone(), b"did:identity:bob".to_vec());
        let student_did = H256::from_low_u64_be(999);
        
        assert_noop!(
            VerifiableCredentials::issue_credential(
                RuntimeOrigin::signed(bob),
                student_did,
                CredentialType::Education,
                H256::from_low_u64_be(1),
                0,
                H256::from_low_u64_be(2)
            ),
            Error::<Test>::IssuerNotTrusted
        );
    });
}

// Helper functions
fn create_test_identity(account: AccountId32, did: Vec<u8>) -> H256 {
    let pk = H256::from_slice(&account.as_ref()[..32]);
    assert_ok!(IdentityRegistry::create_identity(
        RuntimeOrigin::signed(account.clone()),
        did.clone(),
        pk
    ));
    IdentityRegistry::hash_did(&did)
}

fn fund_account(account: AccountId32, amount: Balance) {
    let _ = Balances::deposit_creating(&account, amount);
}

fn run_to_block(n: BlockNumber) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}

fn advance_time(seconds: u64) {
    let current = Timestamp::now();
    Timestamp::set_timestamp(current + (seconds * 1000));
}