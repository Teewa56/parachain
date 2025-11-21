#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use pallet_verifiable_credentials::CredentialType;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
use crate::weights::WeightInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Time},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_core::H256;
    use sp_runtime::traits::StaticLookup;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: ReservableCurrency<Self::AccountId>;
        type TimeProvider: Time;
        
        /// Minimum deposit to create a proposal
        #[pallet::constant]
        type ProposalDeposit: Get<BalanceOf<Self>>;
        
        /// Voting period in blocks
        #[pallet::constant]
        type VotingPeriod: Get<BlockNumberFor<Self>>;
        
        /// Minimum percentage of yes votes to pass (0-100)
        #[pallet::constant]
        type ApprovalThreshold: Get<u8>;

        type WeightInfo: WeightInfo;
    }

    /// Proposal types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProposalType {
        AddTrustedIssuer,
        RemoveTrustedIssuer,
        UpdateIssuerPermissions,
        EmergencyRevoke,
    }

    /// Credential type for issuer authorization
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum CredentialTypeAuth {
        Education,
        Health,
        Employment,
        Age,
        Address,
        Custom,
        All, // Can issue any type
    }

    /// Proposal status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProposalStatus {
        Active,
        Approved,
        Rejected,
        Executed,
        Cancelled,
    }

    /// A governance proposal
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Proposal<T: Config> {
        pub proposer: T::AccountId,
        pub proposal_type: ProposalType,
        pub issuer_did: H256,
        pub credential_types: Vec<CredentialType>,
        pub description: Vec<u8>,
        pub deposit: BalanceOf<T>,
        pub created_at: BlockNumberFor<T>,
        pub voting_ends_at: BlockNumberFor<T>,
        pub status: ProposalStatus,
        pub yes_votes: u32,
        pub no_votes: u32,
        pub total_votes: u32,
    }

    /// Vote on a proposal
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum Vote {
        Yes,
        No,
        Abstain,
    }

    /// Voter information
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct VoterInfo<T: Config> {
        pub account: T::AccountId,
        pub vote: Vote,
        pub voting_power: u32,
        pub voted_at: BlockNumberFor<T>,
    }

    /// Storage: Proposals by ID
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        Proposal<T>,
        OptionQuery,
    >;

    /// Storage: Next proposal ID
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Storage: Votes on proposals
    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64, // proposal_id
        Blake2_128Concat,
        T::AccountId,
        VoterInfo<T>,
        OptionQuery,
    >;

    /// Storage: Council members (have voting power)
    #[pallet::storage]
    #[pallet::getter(fn council_members)]
    pub type CouncilMembers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // voting power
        OptionQuery,
    >;

    /// Storage: Approved trusted issuers
    #[pallet::storage]
    #[pallet::getter(fn trusted_issuers)]
    pub type TrustedIssuers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        H256, // issuer DID
        Blake2_128Concat,
        CredentialTypeAuth,
        bool,
        ValueQuery,
    >;

    /// Storage: Issuer metadata
    #[pallet::storage]
    #[pallet::getter(fn issuer_metadata)]
    pub type IssuerMetadata<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,
        Vec<u8>, // JSON metadata
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProposalCreated {
            proposal_id: u64,
            proposer: T::AccountId,
            issuer_did: H256,
        },
        VoteCast {
            proposal_id: u64,
            voter: T::AccountId,
            vote: Vote,
        },
        ProposalApproved { proposal_id: u64 },
        ProposalRejected { proposal_id: u64 },
        ProposalExecuted { proposal_id: u64 },
        ProposalCancelled { proposal_id: u64 },
        TrustedIssuerAdded {
            issuer_did: H256,
            credential_types: Vec<CredentialType>,
        },
        TrustedIssuerRemoved { issuer_did: H256 },
        CouncilMemberAdded {
            member: T::AccountId,
            voting_power: u32,
        },
        CouncilMemberRemoved { member: T::AccountId },
    }


    #[pallet::error]
    pub enum Error<T> {
        /// Proposal not found
        ProposalNotFound,
        /// Not a council member
        NotCouncilMember,
        /// Already voted
        AlreadyVoted,
        /// Voting period ended
        VotingPeriodEnded,
        /// Voting period not ended
        VotingPeriodNotEnded,
        /// Proposal not active
        ProposalNotActive,
        /// Insufficient deposit
        InsufficientDeposit,
        /// Invalid proposal
        InvalidProposal,
        /// Not authorized
        NotAuthorized,
        /// Issuer already trusted
        IssuerAlreadyTrusted,
        /// Issuer not found
        IssuerNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a proposal to add a trusted issuer
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_add_issuer())]
        pub fn propose_add_issuer(
            origin: OriginFor<T>,
            issuer_did: H256,
            credential_types: Vec<CredentialTypeAuth>,
            description: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Reserve deposit
            T::Currency::reserve(&who, T::ProposalDeposit::get())
                .map_err(|_| Error::<T>::InsufficientDeposit)?;

            let proposal_id = NextProposalId::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();
            let voting_ends_at = current_block + T::VotingPeriod::get();

            let proposal = Proposal {
                proposer: who.clone(),
                proposal_type: ProposalType::AddTrustedIssuer,
                issuer_did,
                credential_types,
                description,
                deposit: T::ProposalDeposit::get(),
                created_at: current_block,
                voting_ends_at,
                status: ProposalStatus::Active,
                yes_votes: 0,
                no_votes: 0,
                total_votes: 0,
            };

            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);

            Self::deposit_event(Event::ProposalCreated {
                proposal_id,
                proposer: who,
                issuer_did,
            });

            Ok(())
        }

        /// Vote on a proposal
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::vote())]
        pub fn vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            vote: Vote,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let voting_power = CouncilMembers::<T>::get(&who)
                .ok_or(Error::<T>::NotCouncilMember)?;

            ensure!(
                !Votes::<T>::contains_key(proposal_id, &who),
                Error::<T>::AlreadyVoted
            );

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block <= proposal.voting_ends_at,
                Error::<T>::VotingPeriodEnded
            );
            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            let voter_info = VoterInfo {
                account: who.clone(),
                vote: vote.clone(),
                voting_power,
                voted_at: current_block,
            };

            Votes::<T>::insert(proposal_id, &who, voter_info);

            match vote {
                Vote::Yes => {
                    proposal.yes_votes = proposal.yes_votes.saturating_add(voting_power);
                },
                Vote::No => {
                    proposal.no_votes = proposal.no_votes.saturating_add(voting_power);
                },
                Vote::Abstain => {},
            }

            proposal.total_votes = proposal.yes_votes
                .saturating_add(proposal.no_votes);

            Proposals::<T>::insert(proposal_id, proposal);

            Self::deposit_event(Event::VoteCast {
                proposal_id,
                voter: who,
                vote,
            });

            Ok(())
        }

        /// Finalize a proposal after voting period
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::finalize_proposal())]
        pub fn finalize_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block > proposal.voting_ends_at,
                Error::<T>::VotingPeriodNotEnded
            );

            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            let approval_percentage = if proposal.total_votes > 0 {
                (proposal.yes_votes.saturating_mul(100)) / proposal.total_votes
            } else {
                0
            };

            if approval_percentage >= T::ApprovalThreshold::get() as u32 {
                proposal.status = ProposalStatus::Approved;
                Self::deposit_event(Event::ProposalApproved { proposal_id });

                Self::execute_proposal(&proposal)?;
                proposal.status = ProposalStatus::Executed;
                Self::deposit_event(Event::ProposalExecuted { proposal_id });

                T::Currency::unreserve(&proposal.proposer, proposal.deposit);
            } else {
                proposal.status = ProposalStatus::Rejected;
                Self::deposit_event(Event::ProposalRejected { proposal_id });

                // Slash 50% of deposit on rejection (anti-spam)
                let (slashed, _remaining) = T::Currency::slash_reserved(&proposal.proposer, proposal.deposit);
            }

            Proposals::<T>::insert(proposal_id, proposal);

            Ok(())
        }

        /// Add council member (requires root)
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::add_council_member())]
        pub fn add_council_member(
            origin: OriginFor<T>,
            member: <T::Lookup as StaticLookup>::Source,
            voting_power: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let member_account = T::Lookup::lookup(member)?;
            CouncilMembers::<T>::insert(&member_account, voting_power);

            Self::deposit_event(Event::CouncilMemberAdded {
                member: member_account,
                voting_power,
            });

            Ok(())
        }

        /// Remove council member (requires root)
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_council_member())]
        pub fn remove_council_member(
            origin: OriginFor<T>,
            member: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let member_account = T::Lookup::lookup(member)?;
            CouncilMembers::<T>::remove(&member_account);

            Self::deposit_event(Event::CouncilMemberRemoved {
                member: member_account,
            });

            Ok(())
        }

        /// Emergency remove trusted issuer
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::emergency_remove_issuer())]
        pub fn emergency_remove_issuer(
            origin: OriginFor<T>,
            issuer_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Remove all authorizations
            let _ = TrustedIssuers::<T>::clear_prefix(issuer_did, u32::MAX, None);
            IssuerMetadata::<T>::remove(issuer_did);

            Self::deposit_event(Event::TrustedIssuerRemoved { issuer_did });

            Ok(())
        }

        /// Cancel own proposal (before voting ends)
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_proposal())]
        pub fn cancel_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(proposal.proposer == who, Error::<T>::NotAuthorized);
            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            proposal.status = ProposalStatus::Cancelled;
            Proposals::<T>::insert(proposal_id, proposal.clone());

            // Return deposit
            T::Currency::unreserve(&proposal.proposer, proposal.deposit);

            Self::deposit_event(Event::ProposalCancelled { proposal_id });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Execute an approved proposal
        fn execute_proposal(proposal: &Proposal<T>) -> DispatchResult {
            match proposal.proposal_type {
                ProposalType::AddTrustedIssuer => {
                    for cred_type in &proposal.credential_types {
                        pallet_verifiable_credentials::Pallet::<T>::add_trusted_issuer_internal(
                            proposal.issuer_did,
                            cred_type.clone(),
                        )?;
                    }

                    Self::deposit_event(Event::TrustedIssuerAdded {
                        issuer_did: proposal.issuer_did,
                        credential_types: proposal.credential_types.clone(),
                    });
                }
                ProposalType::RemoveTrustedIssuer => {
                    pallet_verifiable_credentials::Pallet::<T>::remove_trusted_issuer_internal(
                        proposal.issuer_did
                    )?;

                    Self::deposit_event(Event::TrustedIssuerRemoved {
                        issuer_did: proposal.issuer_did,
                    });
                }
                ProposalType::UpdateIssuerPermissions => {
                    // Update existing issuer's credential types
                    for cred_type in &proposal.credential_types {
                        pallet_verifiable_credentials::Pallet::<T>::add_trusted_issuer_internal(
                            proposal.issuer_did,
                            cred_type.clone(),
                        )?;
                    }

                    Self::deposit_event(Event::TrustedIssuerAdded {
                        issuer_did: proposal.issuer_did,
                        credential_types: proposal.credential_types.clone(),
                    });
                }
                ProposalType::EmergencyRevoke => {
                    // Emergency revoke: immediately remove all permissions
                    pallet_verifiable_credentials::Pallet::<T>::remove_trusted_issuer_internal(
                        proposal.issuer_did
                    )?;

                    Self::deposit_event(Event::TrustedIssuerRemoved {
                        issuer_did: proposal.issuer_did,
                    });
                }
            }

            Ok(())
        }

        /// Check if an issuer is trusted for a credential type
        pub fn is_issuer_trusted(
            issuer_did: &H256,
            credential_type: &CredentialType,
        ) -> bool {
            TrustedIssuers::<T>::get((credential_type, issuer_did))
        }

        /// Get total council voting power
        pub fn total_voting_power() -> u32 {
            CouncilMembers::<T>::iter()
                .fold(0u32, |acc, (_, power)| acc.saturating_add(power))
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn add_trusted_issuer_internal(
            issuer_did: H256,
            credential_type: CredentialType,
        ) -> DispatchResult {
            TrustedIssuers::<T>::insert((&credential_type, &issuer_did), true);
            Ok(())
        }

        pub fn remove_trusted_issuer_internal(
            issuer_did: H256,
        ) -> DispatchResult {
            let credential_types = vec![
                CredentialType::Education,
                CredentialType::Health,
                CredentialType::Employment,
                CredentialType::Age,
                CredentialType::Address,
                CredentialType::Custom,
            ];
            
            for cred_type in credential_types {
                TrustedIssuers::<T>::remove((&cred_type, &issuer_did));
            }
            Ok(())
        }
    }

}