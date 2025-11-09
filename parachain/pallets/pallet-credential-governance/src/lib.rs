#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

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
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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
    }

    /// Proposal types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProposalType {
        /// Add a trusted issuer
        AddTrustedIssuer,
        /// Remove a trusted issuer
        RemoveTrustedIssuer,
        /// Update issuer permissions
        UpdateIssuerPermissions,
        /// Emergency revoke issuer
        EmergencyRevoke,
    }

    /// Credential type for issuer authorization
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
        /// Proposer
        pub proposer: T::AccountId,
        /// Proposal type
        pub proposal_type: ProposalType,
        /// Issuer DID being proposed
        pub issuer_did: H256,
        /// Credential types authorized
        pub credential_types: Vec<CredentialTypeAuth>,
        /// Proposal description/justification
        pub description: Vec<u8>,
        /// Deposit locked
        pub deposit: BalanceOf<T>,
        /// When proposal was created
        pub created_at: BlockNumberFor<T>,
        /// When voting ends
        pub voting_ends_at: BlockNumberFor<T>,
        /// Current status
        pub status: ProposalStatus,
        /// Yes votes
        pub yes_votes: u32,
        /// No votes
        pub no_votes: u32,
        /// Total voting power
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
        /// Proposal created [proposal_id, proposer, issuer_did]
        ProposalCreated {
            proposal_id: u64,
            proposer: T::AccountId,
            issuer_did: H256,
        },
        /// Vote cast [proposal_id, voter, vote]
        VoteCast {
            proposal_id: u64,
            voter: T::AccountId,
            vote: Vote,
        },
        /// Proposal approved [proposal_id]
        ProposalApproved { proposal_id: u64 },
        /// Proposal rejected [proposal_id]
        ProposalRejected { proposal_id: u64 },
        /// Proposal executed [proposal_id]
        ProposalExecuted { proposal_id: u64 },
        /// Proposal cancelled [proposal_id]
        ProposalCancelled { proposal_id: u64 },
        /// Trusted issuer added [issuer_did, credential_types]
        TrustedIssuerAdded {
            issuer_did: H256,
            credential_types: Vec<CredentialTypeAuth>,
        },
        /// Trusted issuer removed [issuer_did]
        TrustedIssuerRemoved { issuer_did: H256 },
        /// Council member added [member, voting_power]
        CouncilMemberAdded {
            member: T::AccountId,
            voting_power: u32,
        },
        /// Council member removed [member]
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
        #[pallet::weight(10_000)]
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
        #[pallet::weight(10_000)]
        pub fn vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            vote: Vote,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if council member
            let voting_power = CouncilMembers::<T>::get(&who)
                .ok_or(Error::<T>::NotCouncilMember)?;

            // Check if already voted
            ensure!(
                !Votes::<T>::contains_key(proposal_id, &who),
                Error::<T>::AlreadyVoted
            );

            // Get proposal
            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Check if voting is still active
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block <= proposal.voting_ends_at,
                Error::<T>::VotingPeriodEnded
            );
            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            // Record vote
            let voter_info = VoterInfo {
                account: who.clone(),
                vote: vote.clone(),
                voting_power,
                voted_at: current_block,
            };

            Votes::<T>::insert(proposal_id, &who, voter_info);

            // Update vote counts
            match vote {
                Vote::Yes => proposal.yes_votes += voting_power,
                Vote::No => proposal.no_votes += voting_power,
                Vote::Abstain => {},
            }
            proposal.total_votes += voting_power;

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
        #[pallet::weight(10_000)]
        pub fn finalize_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Check voting period ended
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block > proposal.voting_ends_at,
                Error::<T>::VotingPeriodNotEnded
            );

            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            // Calculate approval percentage
            let approval_percentage = if proposal.total_votes > 0 {
                (proposal.yes_votes * 100) / proposal.total_votes
            } else {
                0
            };

            // Check if approved
            if approval_percentage >= T::ApprovalThreshold::get() as u32 {
                proposal.status = ProposalStatus::Approved;
                Self::deposit_event(Event::ProposalApproved { proposal_id });

                // Execute proposal
                Self::execute_proposal(&proposal)?;
                proposal.status = ProposalStatus::Executed;
                Self::deposit_event(Event::ProposalExecuted { proposal_id });

                // Return deposit
                T::Currency::unreserve(&proposal.proposer, proposal.deposit);
            } else {
                proposal.status = ProposalStatus::Rejected;
                Self::deposit_event(Event::ProposalRejected { proposal_id });

                // Slash deposit (or return based on your preference)
                // T::Currency::slash_reserved(&proposal.proposer, proposal.deposit);
            }

            Proposals::<T>::insert(proposal_id, proposal);

            Ok(())
        }

        /// Add council member (requires root)
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
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
        #[pallet::weight(10_000)]
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

        /// Emergency remove trusted issuer (requires root)
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
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
        #[pallet::weight(10_000)]
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
                        TrustedIssuers::<T>::insert(
                            proposal.issuer_did,
                            cred_type,
                            true,
                        );
                    }

                    Self::deposit_event(Event::TrustedIssuerAdded {
                        issuer_did: proposal.issuer_did,
                        credential_types: proposal.credential_types.clone(),
                    });
                }
                ProposalType::RemoveTrustedIssuer => {
                    let _ = TrustedIssuers::<T>::clear_prefix(
                        proposal.issuer_did,
                        u32::MAX,
                        None,
                    );

                    Self::deposit_event(Event::TrustedIssuerRemoved {
                        issuer_did: proposal.issuer_did,
                    });
                }
                _ => {}
            }

            Ok(())
        }

        /// Check if an issuer is trusted for a credential type
        pub fn is_issuer_trusted(
            issuer_did: &H256,
            credential_type: &CredentialTypeAuth,
        ) -> bool {
            TrustedIssuers::<T>::get(issuer_did, credential_type) ||
            TrustedIssuers::<T>::get(issuer_did, &CredentialTypeAuth::All)
        }

        /// Get total council voting power
        pub fn total_voting_power() -> u32 {
            CouncilMembers::<T>::iter()
                .fold(0u32, |acc, (_, power)| acc.saturating_add(power))
        }
    }
}