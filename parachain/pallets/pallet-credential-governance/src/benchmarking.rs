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
    fn propose_add_issuer() {
        let proposer: T::AccountId = whitelisted_caller();
        
        // Setup: Create identity for proposer
        let proposer_did = b"did:identity:proposer".to_vec();
        let proposer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(proposer.clone()).into(),
            proposer_did.clone(),
            proposer_pk
        ).unwrap();
        
        // Add proposer as council member
        CouncilMembers::<T>::insert(&proposer, 10u32);
        
        // Give proposer funds for deposit
        let deposit = T::ProposalDeposit::get();
        T::Currency::make_free_balance_be(&proposer, deposit * 2u32.into());
        
        let issuer_did = H256::from_low_u64_be(999);
        let credential_types = vec![CredentialTypeAuth::Education];
        let description = b"Test Hospital".to_vec();

        #[extrinsic_call]
        propose_add_issuer(
            RawOrigin::Signed(proposer),
            issuer_did,
            credential_types,
            description
        );

        assert_eq!(NextProposalId::<T>::get(), 1);
    }

    #[benchmark]
    fn vote() {
        let proposer: T::AccountId = account("proposer", 0, 0);
        let voter: T::AccountId = whitelisted_caller();
        
        // Setup proposer identity
        let proposer_did = b"did:identity:proposer".to_vec();
        let proposer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(proposer.clone()).into(),
            proposer_did.clone(),
            proposer_pk
        ).unwrap();
        
        // Add as council members
        CouncilMembers::<T>::insert(&proposer, 5u32);
        CouncilMembers::<T>::insert(&voter, 10u32);
        
        // Give proposer funds
        let deposit = T::ProposalDeposit::get();
        T::Currency::make_free_balance_be(&proposer, deposit * 2u32.into());
        
        // Create proposal
        let issuer_did = H256::from_low_u64_be(999);
        Pallet::<T>::propose_add_issuer(
            RawOrigin::Signed(proposer).into(),
            issuer_did,
            vec![CredentialTypeAuth::Education],
            b"Test".to_vec()
        ).unwrap();
        
        let proposal_id = 0u64;

        #[extrinsic_call]
        vote(
            RawOrigin::Signed(voter.clone()),
            proposal_id,
            Vote::Yes
        );

        assert!(Votes::<T>::contains_key(proposal_id, &voter));
    }

    #[benchmark]
    fn finalize_proposal() {
        let proposer: T::AccountId = account("proposer", 0, 0);
        let voter1: T::AccountId = account("voter1", 0, 0);
        let voter2: T::AccountId = account("voter2", 0, 0);
        let finalizer: T::AccountId = whitelisted_caller();
        
        // Setup
        let proposer_did = b"did:identity:proposer".to_vec();
        let proposer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(proposer.clone()).into(),
            proposer_did.clone(),
            proposer_pk
        ).unwrap();
        
        CouncilMembers::<T>::insert(&proposer, 5u32);
        CouncilMembers::<T>::insert(&voter1, 10u32);
        CouncilMembers::<T>::insert(&voter2, 15u32);
        
        let deposit = T::ProposalDeposit::get();
        T::Currency::make_free_balance_be(&proposer, deposit * 2u32.into());
        
        let issuer_did = H256::from_low_u64_be(999);
        Pallet::<T>::propose_add_issuer(
            RawOrigin::Signed(proposer.clone()).into(),
            issuer_did,
            vec![CredentialTypeAuth::Education],
            b"Test".to_vec()
        ).unwrap();
        
        let proposal_id = 0u64;
        
        // Cast votes
        Pallet::<T>::vote(
            RawOrigin::Signed(voter1).into(),
            proposal_id,
            Vote::Yes
        ).unwrap();
        
        Pallet::<T>::vote(
            RawOrigin::Signed(voter2).into(),
            proposal_id,
            Vote::Yes
        ).unwrap();
        
        // Fast forward time past voting period
        let mut proposal = Proposals::<T>::get(proposal_id).unwrap();
        proposal.voting_ends_at = 0u32.into();
        Proposals::<T>::insert(proposal_id, proposal);

        #[extrinsic_call]
        finalize_proposal(
            RawOrigin::Signed(finalizer),
            proposal_id
        );

        let proposal = Proposals::<T>::get(proposal_id).unwrap();
        assert!(proposal.status == ProposalStatus::Executed || proposal.status == ProposalStatus::Approved);
    }

    #[benchmark]
    fn add_council_member() {
        let member: T::AccountId = whitelisted_caller();
        let voting_power = 10u32;

        #[extrinsic_call]
        add_council_member(
            RawOrigin::Root,
            member.clone(),
            voting_power
        );

        assert_eq!(CouncilMembers::<T>::get(&member), Some(voting_power));
    }

    #[benchmark]
    fn remove_council_member() {
        let member: T::AccountId = whitelisted_caller();
        
        // Setup: Add member first
        CouncilMembers::<T>::insert(&member, 10u32);

        #[extrinsic_call]
        remove_council_member(
            RawOrigin::Root,
            member.clone()
        );

        assert!(!CouncilMembers::<T>::contains_key(&member));
    }

    #[benchmark]
    fn emergency_remove_issuer() {
        let issuer_did = H256::from_low_u64_be(123);
        
        // Setup: Add as trusted issuer
        TrustedIssuers::<T>::insert(&issuer_did, CredentialTypeAuth::Education, true);

        #[extrinsic_call]
        emergency_remove_issuer(
            RawOrigin::Root,
            issuer_did
        );
    }

    #[benchmark]
    fn cancel_proposal() {
        let proposer: T::AccountId = whitelisted_caller();
        
        // Setup
        let proposer_did = b"did:identity:proposer".to_vec();
        let proposer_pk = H256::from_low_u64_be(1);
        pallet_identity_registry::Pallet::<T>::create_identity(
            RawOrigin::Signed(proposer.clone()).into(),
            proposer_did.clone(),
            proposer_pk
        ).unwrap();
        
        CouncilMembers::<T>::insert(&proposer, 10u32);
        
        let deposit = T::ProposalDeposit::get();
        T::Currency::make_free_balance_be(&proposer, deposit * 2u32.into());
        
        let issuer_did = H256::from_low_u64_be(999);
        Pallet::<T>::propose_add_issuer(
            RawOrigin::Signed(proposer.clone()).into(),
            issuer_did,
            vec![CredentialTypeAuth::Education],
            b"Test".to_vec()
        ).unwrap();
        
        let proposal_id = 0u64;

        #[extrinsic_call]
        cancel_proposal(
            RawOrigin::Signed(proposer),
            proposal_id
        );

        let proposal = Proposals::<T>::get(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Cancelled);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}