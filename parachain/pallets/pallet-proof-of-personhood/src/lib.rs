#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Time},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_core::H256;
    use sp_runtime::traits::StaticLookup;
    use pallet_identity_registry;
    use crate::weights::WeightInfo;
    use frame_support::BoundedVec;
    use pallet_zk_credentials;
    use sp_trie::{generate_trie_proof, verify_trie_proof, LayoutV1, TrieDBBuilder};

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 6 months in seconds
    const RECOVERY_DELAY_SECONDS: u64 = 6 * 30 * 24 * 60 * 60;
    
    /// Cooldown between registrations (6 months)
    const REGISTRATION_COOLDOWN_SECONDS: u64 = 6 * 30 * 24 * 60 * 60;

    /// Base recovery delay: 6 months
    const BASE_RECOVERY_DELAY: u64 = 6 * 30 * 24 * 60 * 60;

    /// Minimum recovery delay even with all evidence: 7 days
    const MIN_RECOVERY_DELAY: u64 = 7 * 24 * 60 * 60;

    /// Required recovery score to finalize (0-100+)
    const REQUIRED_RECOVERY_SCORE: u32 = 100;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_identity_registry::pallet::Config {
        type Currency: ReservableCurrency<Self::AccountId>;
        type TimeProvider: Time;
        /// Deposit required for registration (anti-spam)
        #[pallet::constant]
        type RegistrationDeposit: Get<BalanceOf<Self>>;
        /// Deposit for recovery request
        #[pallet::constant]
        type RecoveryDeposit: Get<BalanceOf<Self>>;
        type ZkCredentials: pallet_zk_credentials::Config;
        type WeightInfo: WeightInfo;
    }

    /// Personhood proof structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct PersonhoodProof<T: Config> {
        /// Commitment to biometric (NOT the biometric itself)
        pub biometric_commitment: H256,
        /// Nullifier derived from biometric (prevents duplicates)
        pub nullifier: H256,
        /// ZK proof of uniqueness
        pub uniqueness_proof: BoundedVec<u8, ConstU32<4096>>,
        /// Timestamp of registration
        pub registered_at: u64,
        /// Associated DID
        pub did: H256,
        /// Account that registered
        pub controller: T::AccountId,
    }

    /// Progressive recovery request with multi-layered evidence
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct ProgressiveRecoveryRequest<T: Config> {
        /// Original DID being recovered
        pub did: H256,
        /// Old nullifier to be replaced
        pub old_nullifier: H256,
        /// New nullifier from new biometric (or none if total loss)
        pub new_nullifier: Option<H256>,
        /// New commitment
        pub new_commitment: Option<H256>,
        /// Guardian votes (guardian -> vote_strength)
        pub guardian_votes: BoundedVec<(T::AccountId, u8), ConstU32<10>>,
        /// Behavioral biometric confidence (0-100)
        pub behavioral_confidence: u8,
        /// Historical access proof strength (0-100)
        pub historical_proof_strength: u8,
        /// Economic stake deposited
        pub economic_stake: BalanceOf<T>,
        /// When recovery was initiated
        pub requested_at: u64,
        /// Current finalization delay (reduced by evidence)
        pub finalization_delay: u64,
        /// Base delay before any evidence (6 months)
        pub base_delay: u64,
        /// Requester account
        pub requester: T::AccountId,
        /// Recovery score (0-100+)
        pub recovery_score: u32,
    }

    /// Cross-biometric proof structure
    /// Proves: "nullifier_A and nullifier_B belong to the same person"
    /// WITHOUT revealing the biometrics
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct CrossBiometricProof {
        /// First biometric nullifier
        pub nullifier_a: H256,
        /// Second biometric nullifier
        pub nullifier_b: H256,
        /// Modality of first biometric
        pub modality_a: BiometricModality,
        /// Modality of second biometric
        pub modality_b: BiometricModality,
        /// ZK proof that both biometrics come from same physical person
        /// Private inputs: biometric_a, biometric_b
        /// Public inputs: nullifier_a, nullifier_b
        /// Circuit proves: Hash(bio_a) == nullifier_a AND Hash(bio_b) == nullifier_b
        ///                 AND bio_a and bio_b pass liveness checks
        ///                 AND bio_a and bio_b are from same capture session
        pub zk_binding_proof: BoundedVec<u8, ConstU32<4096>>,
        /// Session token (prevents replay attacks)
        pub session_id: H256,
        /// Timestamp when biometrics were captured
        pub captured_at: u64,
    }

    /// Recovery request structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct RecoveryRequest<T: Config> {
        /// Original DID being recovered
        pub did: H256,
        /// Old nullifier to be replaced
        pub old_nullifier: H256,
        /// New nullifier from new biometric
        pub new_nullifier: H256,
        /// New commitment
        pub new_commitment: H256,
        /// ZK proof linking old and new identity
        pub recovery_proof: BoundedVec<u8, ConstU32<4096>>,
        /// Social recovery guardians
        pub guardians: BoundedVec<T::AccountId, ConstU32<10>>,
        /// Timestamp when requested
        pub requested_at: u64,
        /// Becomes active after this time
        pub active_at: u64,
        /// Deposit paid
        pub deposit: BalanceOf<T>,
        /// Requester account
        pub requester: T::AccountId,
    }

    /// Storage: Personhood registry by nullifier
    #[pallet::storage]
    #[pallet::getter(fn personhood_registry)]
    pub type PersonhoodRegistry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // nullifier
        PersonhoodProof<T>,
        OptionQuery,
    >;

    /// Guardian relationship with reputation weighting
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct GuardianRelationship<T: Config> {
        pub guardian: T::AccountId,
        /// Relationship strength (1-10: family=10, friend=7, colleague=3)
        pub relationship_strength: u8,
        /// When relationship was established (older = more trusted)
        pub established_at: u64,
        /// Number of interactions (prevents fresh relationships)
        pub interaction_count: u32,
        /// Bonded stake (slashed if fraudulent approval)
        pub bonded_stake: BalanceOf<T>,
    }

    /// A biometric binding links multiple biometric nullifiers to one personhood
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct BiometricBinding<T: Config> {
        /// Primary personhood DID (anchor)
        pub primary_did: H256,
        /// Primary nullifier (first registered biometric)
        pub primary_nullifier: H256,
        /// Additional biometric nullifiers bound to this personhood
        pub bound_nullifiers: BoundedVec<(H256, BiometricModality), ConstU32<10>>,
        /// When binding was created
        pub created_at: u64,
        /// Last binding update
        pub updated_at: u64,
        /// Controller account
        pub controller: T::AccountId,
    }

    /// Storage: Map DID to nullifier
    #[pallet::storage]
    #[pallet::getter(fn did_to_nullifier)]
    pub type DidToNullifier<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        H256, // nullifier
        OptionQuery,
    >;

    /// Storage: Pending recovery requests
    #[pallet::storage]
    #[pallet::getter(fn pending_recoveries)]
    pub type PendingRecoveries<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        RecoveryRequest<T>,
        OptionQuery,
    >;

    /// Storage: Guardian approvals for recovery
    #[pallet::storage]
    #[pallet::getter(fn guardian_approvals)]
    pub type GuardianApprovals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        BoundedVec<T::AccountId, ConstU32<10>>,
        ValueQuery,
    >;

    /// Storage: Registration cooldown
    #[pallet::storage]
    #[pallet::getter(fn registration_cooldown)]
    pub type RegistrationCooldown<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // nullifier
        u64, // can register again after this timestamp
        ValueQuery,
    >;

    /// Storage: Last activity timestamp for each DID
    #[pallet::storage]
    #[pallet::getter(fn last_activity)]
    pub type LastActivity<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        u64, // timestamp
        ValueQuery,
    >;

    /// Guardian relationships by DID
    #[pallet::storage]
    #[pallet::getter(fn guardian_relationships)]
    pub type GuardianRelationships<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        H256, // DID
        Blake2_128Concat,
        T::AccountId, // Guardian
        GuardianRelationship<T>,
        OptionQuery,
    >;

    /// Progressive recovery requests
    #[pallet::storage]
    #[pallet::getter(fn progressive_recoveries)]
    pub type ProgressiveRecoveries<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        ProgressiveRecoveryRequest<T>,
        OptionQuery,
    >;

    /// Behavioral biometric models (hash of pattern data)
    #[pallet::storage]
    #[pallet::getter(fn behavioral_patterns)]
    pub type BehavioralPatterns<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        BoundedVec<H256, ConstU32<10>>, // Multiple pattern hashes
        ValueQuery,
    >;

    /// Master personhood registry: nullifier -> binding
    /// This replaces single-nullifier approach
    #[pallet::storage]
    #[pallet::getter(fn biometric_bindings)]
    pub type BiometricBindings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // Any nullifier (primary or bound)
        H256, // Points to primary_did
        OptionQuery,
    >;

    /// Personhood bindings by primary DID
    #[pallet::storage]
    #[pallet::getter(fn personhood_bindings)]
    pub type PersonhoodBindings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // Primary DID
        BiometricBinding<T>,
        OptionQuery,
    >;

    /// Prevents binding same nullifier to multiple personhoods
    #[pallet::storage]
    #[pallet::getter(fn nullifier_claims)]
    pub type NullifierClaims<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // Nullifier
        bool, // Claimed?
        ValueQuery,
    >;

    /// Session tokens for binding operations (prevents replay)
    #[pallet::storage]
    #[pallet::getter(fn used_session_tokens)]
    pub type UsedSessionTokens<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // Session ID
        u64, // Timestamp
        OptionQuery,
    >;

    /// Biometric modality types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum BiometricModality {
        Fingerprint,
        Iris,
        FaceGeometry,
        Voice,
        Gait,
        Retina,
    }

    /// Evidence types for progressive recovery
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum EvidenceType {
        /// Guardian approval with vote strength
        GuardianApproval { vote_strength: u8 },
        /// Behavioral biometric (typing pattern, gait, etc.)
        BehavioralBiometric,
        /// Proof of access to historical data/keys
        HistoricalAccess,
        /// Economic stake as confidence signal
        EconomicStake,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Personhood registered [did, nullifier]
        PersonhoodRegistered { did: H256, nullifier: H256 },
        /// Recovery requested [did, guardians, active_at]
        RecoveryRequested {
            did: H256,
            guardians: Vec<T::AccountId>,
            active_at: u64,
        },
        /// Recovery approved by guardian [did, guardian]
        RecoveryApproved { did: H256, guardian: T::AccountId },
        /// Recovery finalized [did]
        RecoveryFinalized { did: H256 },
        /// Recovery cancelled [did]
        RecoveryCancelled { did: H256 },
        /// Activity recorded [did, timestamp]
        ActivityRecorded { did: H256, timestamp: u64 },
        /// Guardian relationship established [did, guardian, strength]
        GuardianRelationshipEstablished {
            did: H256,
            guardian: T::AccountId,
            strength: u8,
        },
        /// Progressive recovery initiated [did, base_delay]
        ProgressiveRecoveryInitiated {
            did: H256,
            base_delay: u64,
        },
        /// Recovery evidence submitted [did, evidence_type, score_increase]
        RecoveryEvidenceSubmitted {
            did: H256,
            evidence_type: EvidenceType,
            score_increase: u32,
        },
        /// Recovery score updated [did, new_score, delay_remaining]
        RecoveryScoreUpdated {
            did: H256,
            new_score: u32,
            delay_remaining: u64,
        },
        /// Recovery ready for finalization [did, final_score]
        RecoveryReadyForFinalization {
            did: H256,
            final_score: u32,
        },
        /// Guardian slashed for fraud [did, guardian, amount]
        GuardianSlashed {
            did: H256,
            guardian: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// Primary personhood registered [did, nullifier, modality]
        PrimaryPersonhoodRegistered {
            did: H256,
            nullifier: H256,
            modality: BiometricModality,
        },
        
        /// Additional biometric bound to personhood [did, nullifier, modality]
        BiometricBound {
            did: H256,
            nullifier: H256,
            modality: BiometricModality,
        },
        
        /// Attempted double registration detected [nullifier, existing_did]
        DoubleRegistrationAttempt {
            nullifier: H256,
            existing_did: H256,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Nullifier already used (duplicate registration)
        NullifierAlreadyUsed,
        /// DID not found
        DidNotFound,
        /// Not authorized
        NotAuthorized,
        /// Recovery request not found
        RecoveryRequestNotFound,
        /// Recovery period not elapsed
        RecoveryPeriodNotElapsed,
        /// Not a guardian
        NotAGuardian,
        /// Insufficient guardian approvals
        InsufficientGuardianApprovals,
        /// Personhood proof not found
        PersonhoodProofNotFound,
        /// Invalid recovery proof
        InvalidRecoveryProof,
        /// Invalid uniqueness proof
        InvalidUniquenessProof,
        /// Registration too soon (cooldown active)
        RegistrationTooSoon,
        /// Insufficient deposit
        InsufficientDeposit,
        /// Recovery already active
        RecoveryAlreadyActive,
        /// Invalid nullifier format
        InvalidNullifier,
        /// Invalid commitment format
        InvalidCommitment,
        /// Guardian relationship already exists
        GuardianAlreadyExists,
        /// Invalid relationship strength (must be 1-10)
        InvalidRelationshipStrength,
        /// Insufficient guardian bond
        InsufficientGuardianBond,
        /// Guardian not found
        GuardianNotFound,
        /// Exceeded voting power
        ExceededVotingPower,
        /// Progressive recovery not found
        ProgressiveRecoveryNotFound,
        /// Recovery score insufficient
        RecoveryScoreInsufficient,
        /// Invalid behavioral proof
        InvalidBehavioralProof,
        /// Invalid historical proof
        InvalidHistoricalProof,
        /// Recovery already in progress
        RecoveryInProgress,
        /// Nullifier already bound to different personhood
        NullifierAlreadyBound,
        /// Invalid cross-biometric proof
        InvalidCrossBiometricProof,
        /// Session token already used
        SessionTokenUsed,
        /// Session token expired
        SessionTokenExpired,
        /// Biometric modality already registered
        ModalityAlreadyRegistered,
        /// Invalid biometric modality
        InvalidBiometricModality,
        /// Binding not found
        BindingNotFound,
        /// Maximum bound biometrics reached
        MaxBiometricsReached,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register personhood with biometric nullifier
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_personhood())]
        pub fn register_personhood(
            origin: OriginFor<T>,
            did: H256,
            nullifier: H256,
            commitment: H256,
            uniqueness_proof: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate nullifier format
            ensure!(
                Self::validate_nullifier(&nullifier),
                Error::<T>::InvalidNullifier
            );
            ensure!(
                Self::validate_commitment(&commitment),
                Error::<T>::InvalidCommitment
            );

            // Check DID exists and belongs to caller
            let identity = pallet_identity_registry::Identities::<T>::get(&did)
                .ok_or(Error::<T>::DidNotFound)?;
            ensure!(identity.controller == who, Error::<T>::NotAuthorized);
            ensure!(identity.active, Error::<T>::NotAuthorized);

            // Check nullifier is unique
            ensure!(
                !PersonhoodRegistry::<T>::contains_key(&nullifier),
                Error::<T>::NullifierAlreadyUsed
            );

            // Check cooldown period
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>().saturated_into::<u64>();
            let cooldown_end = RegistrationCooldown::<T>::get(&nullifier);
            ensure!(now > cooldown_end, Error::<T>::RegistrationTooSoon);

            // Verify uniqueness proof (ZK proof)
            Self::verify_uniqueness_proof(&nullifier, &commitment, &uniqueness_proof)?;

            // Reserve deposit
            T::Currency::reserve(&who, T::RegistrationDeposit::get())
                .map_err(|_| Error::<T>::InsufficientDeposit)?;

            // Create personhood proof
            let proof = PersonhoodProof {
                biometric_commitment: commitment,
                nullifier,
                uniqueness_proof,
                registered_at: now,
                did,
                controller: who.clone(),
            };

            // Store in registry
            PersonhoodRegistry::<T>::insert(&nullifier, proof);
            DidToNullifier::<T>::insert(&did, nullifier);
            
            // Set cooldown for next registration
            let cooldown_until = now.saturating_add(REGISTRATION_COOLDOWN_SECONDS);
            RegistrationCooldown::<T>::insert(&nullifier, cooldown_until);

            // Record activity
            LastActivity::<T>::insert(&did, now);

            Self::deposit_event(Event::PersonhoodRegistered { did, nullifier });

            Ok(())
        }

        /// Request identity recovery
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::request_recovery())]
        pub fn request_recovery(
            origin: OriginFor<T>,
            old_did: H256,
            new_nullifier: H256,
            new_commitment: H256,
            recovery_proof: Vec<u8>,
            guardians: Vec<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate inputs
            ensure!(
                Self::validate_nullifier(&new_nullifier),
                Error::<T>::InvalidNullifier
            );
            ensure!(
                !guardians.is_empty() && guardians.len() <= 10,
                Error::<T>::NotAuthorized
            );

            // Get old nullifier
            let old_nullifier = DidToNullifier::<T>::get(&old_did)
                .ok_or(Error::<T>::DidNotFound)?;

            // Verify old personhood proof exists
            ensure!(
                PersonhoodRegistry::<T>::contains_key(&old_nullifier),
                Error::<T>::PersonhoodProofNotFound
            );

            // Ensure new nullifier is unique
            ensure!(
                !PersonhoodRegistry::<T>::contains_key(&new_nullifier),
                Error::<T>::NullifierAlreadyUsed
            );

            // Check no active recovery
            ensure!(
                !PendingRecoveries::<T>::contains_key(&old_did),
                Error::<T>::RecoveryAlreadyActive
            );

            // Verify recovery proof (ZK proof linking old and new identity)
            Self::verify_recovery_proof(&old_did, &new_nullifier, &recovery_proof)?;

            // Reserve deposit
            T::Currency::reserve(&who, T::RecoveryDeposit::get())
                .map_err(|_| Error::<T>::InsufficientDeposit)?;

            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>().saturated_into::<u64>();
            let active_at = now.saturating_add(RECOVERY_DELAY_SECONDS);

            let guardians_bounded: BoundedVec<T::AccountId, ConstU32<10>> = 
                guardians.clone().try_into().map_err(|_| Error::<T>::NotAuthorized)?;

            let request = RecoveryRequest {
                did: old_did,
                old_nullifier,
                new_nullifier,
                new_commitment,
                recovery_proof,
                guardians: guardians_bounded,
                requested_at: now,
                active_at,
                deposit: T::RecoveryDeposit::get(),
                requester: who,
            };

            PendingRecoveries::<T>::insert(&old_did, request);

            Self::deposit_event(Event::RecoveryRequested {
                did: old_did,
                guardians,
                active_at,
            });

            Ok(())
        }

        /// Guardian approves recovery
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::approve_recovery())]
        pub fn approve_recovery(
            origin: OriginFor<T>,
            did: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let request = PendingRecoveries::<T>::get(&did)
                .ok_or(Error::<T>::RecoveryRequestNotFound)?;

            // Check caller is a guardian
            ensure!(
                request.guardians.contains(&who),
                Error::<T>::NotAGuardian
            );

            // Add approval
            GuardianApprovals::<T>::mutate(&did, |approvals| {
                if !approvals.contains(&who) {
                    let _ = approvals.try_push(who.clone());
                }
            });

            Self::deposit_event(Event::RecoveryApproved { did, guardian: who });

            Ok(())
        }

        /// Finalize recovery after time lock
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::finalize_recovery())]
        pub fn finalize_recovery(
            origin: OriginFor<T>,
            did: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let request = PendingRecoveries::<T>::get(&did)
                .ok_or(Error::<T>::RecoveryRequestNotFound)?;

            ensure!(request.requester == who, Error::<T>::NotAuthorized);

            // Check time lock elapsed
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>().saturated_into::<u64>();
            ensure!(now >= request.active_at, Error::<T>::RecoveryPeriodNotElapsed);

            // Check guardian approvals (require 2/3 majority)
            let approvals = GuardianApprovals::<T>::get(&did);
            let required = (request.guardians.len() * 2 / 3).saturating_add(1);
            ensure!(
                approvals.len() >= required,
                Error::<T>::InsufficientGuardianApprovals
            );

            // Get old proof
            let old_proof = PersonhoodRegistry::<T>::get(&request.old_nullifier)
                .ok_or(Error::<T>::PersonhoodProofNotFound)?;

            // Remove old nullifier
            PersonhoodRegistry::<T>::remove(&request.old_nullifier);

            // Create new proof
            let new_proof = PersonhoodProof {
                biometric_commitment: request.new_commitment,
                nullifier: request.new_nullifier,
                uniqueness_proof: request.recovery_proof.clone(),
                registered_at: now,
                did,
                controller: who.clone(),
            };

            // Update registry
            PersonhoodRegistry::<T>::insert(&request.new_nullifier, new_proof);
            DidToNullifier::<T>::insert(&did, request.new_nullifier);

            // Set cooldown
            let cooldown_until = now.saturating_add(REGISTRATION_COOLDOWN_SECONDS);
            RegistrationCooldown::<T>::insert(&request.new_nullifier, cooldown_until);

            // Clean up
            PendingRecoveries::<T>::remove(&did);
            GuardianApprovals::<T>::remove(&did);

            // Return deposit
            T::Currency::unreserve(&request.requester, request.deposit);

            // Record activity
            LastActivity::<T>::insert(&did, now);

            Self::deposit_event(Event::RecoveryFinalized { did });

            Ok(())
        }

        /// Cancel recovery request
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_recovery())]
        pub fn cancel_recovery(
            origin: OriginFor<T>,
            did: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let request = PendingRecoveries::<T>::get(&did)
                .ok_or(Error::<T>::RecoveryRequestNotFound)?;

            ensure!(request.requester == who, Error::<T>::NotAuthorized);

            // Return deposit
            T::Currency::unreserve(&request.requester, request.deposit);

            // Clean up
            PendingRecoveries::<T>::remove(&did);
            GuardianApprovals::<T>::remove(&did);

            Self::deposit_event(Event::RecoveryCancelled { did });

            Ok(())
        }

        /// Record activity (prevents dormant account takeover)
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::record_activity())]
        pub fn record_activity(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get DID from account
            let (did, identity) = pallet_identity_registry::Pallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::DidNotFound)?;

            ensure!(identity.active, Error::<T>::NotAuthorized);

            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>().saturated_into::<u64>();
            LastActivity::<T>::insert(&did, now);

            // Auto-cancel recovery if user becomes active
            if PendingRecoveries::<T>::contains_key(&did) {
                let request = PendingRecoveries::<T>::get(&did).unwrap();
                T::Currency::unreserve(&request.requester, request.deposit);
                PendingRecoveries::<T>::remove(&did);
                GuardianApprovals::<T>::remove(&did);
                Self::deposit_event(Event::RecoveryCancelled { did });
            }

            Self::deposit_event(Event::ActivityRecorded { did, timestamp: now });

            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::add_guardian())]
        pub fn add_guardian(
            origin: OriginFor<T>,
            did: H256,
            guardian: T::AccountId,
            relationship_strength: u8,
            bond_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Verify DID ownership
            let identity = pallet_identity_registry::Identities::<T>::get(&did)
                .ok_or(Error::<T>::DidNotFound)?;
            ensure!(identity.controller == who, Error::<T>::NotAuthorized);
            
            // Validate strength (1-10)
            ensure!(
                relationship_strength >= 1 && relationship_strength <= 10,
                Error::<T>::InvalidRelationshipStrength
            );
            
            // Ensure guardian doesn't already exist
            ensure!(
                !GuardianRelationships::<T>::contains_key(&did, &guardian),
                Error::<T>::GuardianAlreadyExists
            );
            
            // Require minimum bond (prevents sybil guardians)
            let min_bond = T::RecoveryDeposit::get();
            ensure!(bond_amount >= min_bond, Error::<T>::InsufficientGuardianBond);
            
            // Reserve bond from guardian
            T::Currency::reserve(&guardian, bond_amount)?;
            
            let now = T::TimeProvider::now().saturated_into::<u64>();
            
            let relationship = GuardianRelationship {
                guardian: guardian.clone(),
                relationship_strength,
                established_at: now,
                interaction_count: 0,
                bonded_stake: bond_amount,
            };
            
            GuardianRelationships::<T>::insert(&did, &guardian, relationship);
            
            Self::deposit_event(Event::GuardianRelationshipEstablished {
                did,
                guardian,
                strength: relationship_strength,
            });
            
            Ok(())
        }
        
        /// Initiate progressive recovery (catastrophic loss scenario)
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::initiate_progressive_recovery())]
        pub fn initiate_progressive_recovery(
            origin: OriginFor<T>,
            old_did: H256,
            new_nullifier: Option<H256>,
            new_commitment: Option<H256>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Verify old DID exists
            let old_nullifier = DidToNullifier::<T>::get(&old_did)
                .ok_or(Error::<T>::DidNotFound)?;
            
            ensure!(
                PersonhoodRegistry::<T>::contains_key(&old_nullifier),
                Error::<T>::PersonhoodProofNotFound
            );
            
            // Ensure no active recovery
            ensure!(
                !ProgressiveRecoveries::<T>::contains_key(&old_did),
                Error::<T>::RecoveryInProgress
            );
            
            // If providing new nullifier, ensure it's unique
            if let Some(new_null) = new_nullifier {
                ensure!(
                    !PersonhoodRegistry::<T>::contains_key(&new_null),
                    Error::<T>::NullifierAlreadyUsed
                );
            }
            
            let now = T::TimeProvider::now().saturated_into::<u64>();
            
            let request = ProgressiveRecoveryRequest {
                did: old_did,
                old_nullifier,
                new_nullifier,
                new_commitment,
                guardian_votes: BoundedVec::default(),
                behavioral_confidence: 0,
                historical_proof_strength: 0,
                economic_stake: Zero::zero(),
                requested_at: now,
                finalization_delay: BASE_RECOVERY_DELAY,
                base_delay: BASE_RECOVERY_DELAY,
                requester: who,
                recovery_score: 0,
            };
            
            ProgressiveRecoveries::<T>::insert(&old_did, request);
            
            Self::deposit_event(Event::ProgressiveRecoveryInitiated {
                did: old_did,
                base_delay: BASE_RECOVERY_DELAY,
            });
            
            Ok(())
        }
        
        /// Submit recovery evidence (progressive approach)
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_recovery_evidence())]
        pub fn submit_recovery_evidence(
            origin: OriginFor<T>,
            did: H256,
            evidence_type: EvidenceType,
            evidence_data: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let mut recovery = ProgressiveRecoveries::<T>::get(&did)
                .ok_or(Error::<T>::ProgressiveRecoveryNotFound)?;
            
            let mut score_increase: u32 = 0;
            let now = T::TimeProvider::now().saturated_into::<u64>();
            
            match evidence_type {
                EvidenceType::GuardianApproval { vote_strength } => {
                    // Verify caller is a guardian
                    let relationship = GuardianRelationships::<T>::get(&did, &who)
                        .ok_or(Error::<T>::GuardianNotFound)?;
                    
                    // Vote strength cannot exceed relationship strength
                    ensure!(
                        vote_strength <= relationship.relationship_strength,
                        Error::<T>::ExceededVotingPower
                    );
                    
                    // Quadratic voting cost
                    let cost = (vote_strength as u32).saturating_pow(2);
                    ensure!(
                        relationship.interaction_count >= cost,
                        Error::<T>::ExceededVotingPower
                    );
                    
                    // Add vote (or update if already voted)
                    let mut found = false;
                    for (guardian, strength) in recovery.guardian_votes.iter_mut() {
                        if *guardian == who {
                            *strength = vote_strength;
                            found = true;
                            break;
                        }
                    }
                    
                    if !found {
                        recovery.guardian_votes.try_push((who.clone(), vote_strength))
                            .map_err(|_| Error::<T>::NotAuthorized)?;
                    }
                    
                    // Score: weighted votes (max 30 points)
                    let total_weighted_votes: u32 = recovery.guardian_votes.iter()
                        .map(|(g, s)| {
                            GuardianRelationships::<T>::get(&did, g)
                                .map(|r| (*s as u32) * (r.relationship_strength as u32))
                                .unwrap_or(0)
                        })
                        .sum();
                    
                    score_increase = total_weighted_votes.min(30);
                    
                    // Reduce delay: each vote_strength point = 3 days reduction
                    let delay_reduction = (vote_strength as u64) * 3 * 24 * 60 * 60;
                    recovery.finalization_delay = recovery.finalization_delay
                        .saturating_sub(delay_reduction)
                        .max(MIN_RECOVERY_DELAY);
                },
                
                EvidenceType::BehavioralBiometric => {
                    // Verify behavioral pattern matches stored patterns
                    let confidence = Self::verify_behavioral_pattern(&did, &evidence_data)?;
                    recovery.behavioral_confidence = confidence;
                    
                    // Score: 0-30 points based on confidence
                    score_increase = (confidence as u32 * 30) / 100;
                    
                    // High confidence (>80%) reduces delay by 60 days
                    if confidence > 80 {
                        recovery.finalization_delay = recovery.finalization_delay
                            .saturating_sub(60 * 24 * 60 * 60)
                            .max(MIN_RECOVERY_DELAY);
                    }
                },
                
                EvidenceType::HistoricalAccess => {
                    // Verify access to historical keys/data
                    let strength = Self::verify_historical_proof(&did, &evidence_data)?;
                    recovery.historical_proof_strength = strength;
                    
                    // Score: 0-20 points
                    score_increase = (strength as u32 * 20) / 100;
                    
                    // Strong proof (>90%) reduces delay by 45 days
                    if strength > 90 {
                        recovery.finalization_delay = recovery.finalization_delay
                            .saturating_sub(45 * 24 * 60 * 60)
                            .max(MIN_RECOVERY_DELAY);
                    }
                },
                
                EvidenceType::EconomicStake => {
                    // Decode stake amount
                    let stake_amount = BalanceOf::<T>::decode(&mut &evidence_data[..])
                        .map_err(|_| Error::<T>::InvalidRecoveryProof)?;
                    
                    // Reserve additional stake
                    T::Currency::reserve(&who, stake_amount)?;
                    recovery.economic_stake = recovery.economic_stake.saturating_add(stake_amount);
                    
                    // Score: 1 point per 1000 tokens (max 20 points)
                    let stake_u128 = recovery.economic_stake.saturated_into::<u128>();
                    score_increase = ((stake_u128 / 1000) as u32).min(20);
                    
                    // Large stake (>10000) reduces delay by 90 days
                    if stake_u128 > 10_000 {
                        recovery.finalization_delay = recovery.finalization_delay
                            .saturating_sub(90 * 24 * 60 * 60)
                            .max(MIN_RECOVERY_DELAY);
                    }
                },
            }
            
            // Calculate total recovery score
            recovery.recovery_score = Self::calculate_recovery_score(&recovery, now);
            
            ProgressiveRecoveries::<T>::insert(&did, recovery.clone());
            
            Self::deposit_event(Event::RecoveryEvidenceSubmitted {
                did,
                evidence_type: evidence_type.clone(),
                score_increase,
            });
            
            Self::deposit_event(Event::RecoveryScoreUpdated {
                did,
                new_score: recovery.recovery_score,
                delay_remaining: recovery.finalization_delay,
            });
            
            // Check if ready for finalization
            if recovery.recovery_score >= REQUIRED_RECOVERY_SCORE 
                && now >= recovery.requested_at.saturating_add(recovery.finalization_delay) {
                Self::deposit_event(Event::RecoveryReadyForFinalization {
                    did,
                    final_score: recovery.recovery_score,
                });
            }
            
            Ok(())
        }
        
        /// Finalize progressive recovery
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::finalize_progressive_recovery())]
        pub fn finalize_progressive_recovery(
            origin: OriginFor<T>,
            did: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let recovery = ProgressiveRecoveries::<T>::get(&did)
                .ok_or(Error::<T>::ProgressiveRecoveryNotFound)?;
            
            ensure!(recovery.requester == who, Error::<T>::NotAuthorized);
            
            let now = T::TimeProvider::now().saturated_into::<u64>();
            
            // Recalculate final score
            let final_score = Self::calculate_recovery_score(&recovery, now);
            
            // Check requirements
            ensure!(
                final_score >= REQUIRED_RECOVERY_SCORE,
                Error::<T>::RecoveryScoreInsufficient
            );
            
            ensure!(
                now >= recovery.requested_at.saturating_add(recovery.finalization_delay),
                Error::<T>::RecoveryPeriodNotElapsed
            );
            
            // Remove old nullifier
            PersonhoodRegistry::<T>::remove(&recovery.old_nullifier);
            
            // If new biometric provided, register it
            if let (Some(new_nullifier), Some(new_commitment)) = 
                (recovery.new_nullifier, recovery.new_commitment) {
                
                let new_proof = PersonhoodProof {
                    biometric_commitment: new_commitment,
                    nullifier: new_nullifier,
                    uniqueness_proof: BoundedVec::default(),
                    registered_at: now,
                    did,
                    controller: who.clone(),
                };
                
                PersonhoodRegistry::<T>::insert(&new_nullifier, new_proof);
                DidToNullifier::<T>::insert(&did, new_nullifier);
            }
            
            // Return economic stake
            if recovery.economic_stake > Zero::zero() {
                T::Currency::unreserve(&recovery.requester, recovery.economic_stake);
            }
            
            // Clean up
            ProgressiveRecoveries::<T>::remove(&did);
            
            Self::deposit_event(Event::RecoveryFinalized { did });
            
            Ok(())
        }
        
        /// Challenge fraudulent recovery and slash guardian
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::challenge_recovery())]
        pub fn challenge_recovery(
            origin: OriginFor<T>,
            did: H256,
            fraudulent_guardian: T::AccountId,
            fraud_proof: Vec<u8>,
        ) -> DispatchResult {
            let challenger = ensure_signed(origin)?;
            
            // Verify fraud proof (simplified - in production, we will use governance)
            ensure!(
                Self::verify_fraud_proof(&did, &fraudulent_guardian, &fraud_proof),
                Error::<T>::InvalidRecoveryProof
            );
            
            // Get guardian relationship
            let relationship = GuardianRelationships::<T>::get(&did, &fraudulent_guardian)
                .ok_or(Error::<T>::GuardianNotFound)?;
            
            // Slash guardian's bond
            let slashed = T::Currency::slash_reserved(&fraudulent_guardian, relationship.bonded_stake);
            
            // Reward challenger with 50% of slashed amount
            let reward = slashed.0.saturating_div(2u32.into());
            T::Currency::deposit_creating(&challenger, reward);
            
            // Remove guardian
            GuardianRelationships::<T>::remove(&did, &fraudulent_guardian);
            
            // Cancel recovery if active
            if let Some(mut recovery) = ProgressiveRecoveries::<T>::get(&did) {
                // Remove fraudulent guardian's votes
                recovery.guardian_votes.retain(|(g, _)| *g != fraudulent_guardian);
                
                // Recalculate score
                let now = T::TimeProvider::now().saturated_into::<u64>();
                recovery.recovery_score = Self::calculate_recovery_score(&recovery, now);
                
                ProgressiveRecoveries::<T>::insert(&did, recovery);
            }
            
            Self::deposit_event(Event::GuardianSlashed {
                did,
                guardian: fraudulent_guardian,
                amount: slashed.0,
            });
            
            Ok(())
        }
        
        /// Record behavioral pattern for future verification
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::record_behavioral_pattern())]
        pub fn record_behavioral_pattern(
            origin: OriginFor<T>,
            pattern_data: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Get DID from account
            let (did, identity) = pallet_identity_registry::Pallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::DidNotFound)?;
            
            ensure!(identity.active, Error::<T>::NotAuthorized);
            
            // Hash pattern data
            let pattern_hash = sp_io::hashing::blake2_256(&pattern_data).into();
            
            // Store pattern hash
            BehavioralPatterns::<T>::mutate(&did, |patterns| {
                if !patterns.contains(&pattern_hash) {
                    let _ = patterns.try_push(pattern_hash);
                }
            });
            
            Ok(())
        }

        /// Register PRIMARY personhood with first biometric
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::register_primary_personhood())]
        pub fn register_primary_personhood(
            origin: OriginFor<T>,
            did: H256,
            nullifier: H256,
            commitment: H256,
            modality: BiometricModality,
            uniqueness_proof: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Validate inputs
            ensure!(
                Self::validate_nullifier(&nullifier),
                Error::<T>::InvalidNullifier
            );
            
            // Check DID exists and belongs to caller
            let identity = pallet_identity_registry::Identities::<T>::get(&did)
                .ok_or(Error::<T>::DidNotFound)?;
            ensure!(identity.controller == who, Error::<T>::NotAuthorized);
            ensure!(identity.active, Error::<T>::NotAuthorized);
            
            // Check if nullifier is already bound to ANY personhood
            if NullifierClaims::<T>::get(&nullifier) {
                // Get existing personhood
                if let Some(existing_did) = BiometricBindings::<T>::get(&nullifier) {
                    Self::deposit_event(Event::DoubleRegistrationAttempt {
                        nullifier,
                        existing_did,
                    });
                    return Err(Error::<T>::NullifierAlreadyBound.into());
                }
            }
            
            // Verify uniqueness proof
            Self::verify_uniqueness_proof(&nullifier, &commitment, &uniqueness_proof)?;
            
            // Reserve deposit
            T::Currency::reserve(&who, T::RegistrationDeposit::get())
                .map_err(|_| Error::<T>::InsufficientDeposit)?;
            
            let now = T::TimeProvider::now().saturated_into::<u64>();
            
            // Create biometric binding
            let binding = BiometricBinding {
                primary_did: did,
                primary_nullifier: nullifier,
                bound_nullifiers: BoundedVec::default(),
                created_at: now,
                updated_at: now,
                controller: who.clone(),
            };
            
            // Store binding
            PersonhoodBindings::<T>::insert(&did, binding);
            BiometricBindings::<T>::insert(&nullifier, did);
            NullifierClaims::<T>::insert(&nullifier, true);
            
            // Update legacy storage for compatibility
            let proof = PersonhoodProof {
                biometric_commitment: commitment,
                nullifier,
                uniqueness_proof: uniqueness_proof.try_into()
                    .map_err(|_| Error::<T>::InvalidUniquenessProof)?,
                registered_at: now,
                did,
                controller: who.clone(),
            };
            PersonhoodRegistry::<T>::insert(&nullifier, proof);
            DidToNullifier::<T>::insert(&did, nullifier);
            
            Self::deposit_event(Event::PrimaryPersonhoodRegistered {
                did,
                nullifier,
                modality,
            });
            
            Ok(())
        }
        
        /// Bind additional biometric to EXISTING personhood
        /// This is how a user proves his/her iris and thumbprint belong to same person
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::bind_additional_biometric())]
        pub fn bind_additional_biometric(
            origin: OriginFor<T>,
            did: H256,
            new_nullifier: H256,
            new_commitment: H256,
            new_modality: BiometricModality,
            cross_biometric_proof: CrossBiometricProof,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Get existing binding
            let mut binding = PersonhoodBindings::<T>::get(&did)
                .ok_or(Error::<T>::BindingNotFound)?;
            
            ensure!(binding.controller == who, Error::<T>::NotAuthorized);
            
            // Check nullifier not already used
            ensure!(
                !NullifierClaims::<T>::get(&new_nullifier),
                Error::<T>::NullifierAlreadyBound
            );
            
            // Check modality not already registered
            if new_modality == BiometricModality::Fingerprint && 
            binding.primary_nullifier != new_nullifier {
                // Check if already in bound list
                for (_, modality) in binding.bound_nullifiers.iter() {
                    ensure!(
                        *modality != new_modality,
                        Error::<T>::ModalityAlreadyRegistered
                    );
                }
            }
            
            // Check session token not used
            ensure!(
                !UsedSessionTokens::<T>::contains_key(&cross_biometric_proof.session_id),
                Error::<T>::SessionTokenUsed
            );
            
            let now = T::TimeProvider::now().saturated_into::<u64>();
            
            // Session token valid for 5 minutes
            ensure!(
                now.saturating_sub(cross_biometric_proof.captured_at) < 300,
                Error::<T>::SessionTokenExpired
            );
            
            // Verify cross-biometric ZK proof
            // This proves: "I have BOTH biometrics from the SAME capture session"
            Self::verify_cross_biometric_proof(
                &binding.primary_nullifier,
                &new_nullifier,
                &cross_biometric_proof,
            )?;
            
            // Mark session as used
            UsedSessionTokens::<T>::insert(&cross_biometric_proof.session_id, now);
            
            // Add to binding
            binding.bound_nullifiers.try_push((new_nullifier, new_modality.clone()))
                .map_err(|_| Error::<T>::MaxBiometricsReached)?;
            binding.updated_at = now;
            
            // Update storage
            PersonhoodBindings::<T>::insert(&did, binding);
            BiometricBindings::<T>::insert(&new_nullifier, did);
            NullifierClaims::<T>::insert(&new_nullifier, true);
            
            Self::deposit_event(Event::BiometricBound {
                did,
                nullifier: new_nullifier,
                modality: new_modality,
            });
            
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Verify uniqueness proof
        fn verify_uniqueness_proof(
            nullifier: &H256,
            commitment: &H256,
            proof_bytes: &[u8],
        ) -> Result<(), Error<T>> {
            // 1. Verify commitment structure
            // Expected: commitment = Hash(biometric_hash || salt)
            ensure!(
                proof_bytes.len() >= 64,
                Error::<T>::InvalidUniquenessProof
            );
            
            let salt = &proof_bytes[0..32];
            
            // 2. Recompute commitment
            let mut preimage = Vec::new();
            preimage.extend_from_slice(nullifier.as_bytes());
            preimage.extend_from_slice(salt);
            let computed_commitment = sp_io::hashing::blake2_256(&preimage);
            
            ensure!(
                computed_commitment == commitment.as_bytes(),
                Error::<T>::InvalidCommitment
            );
            
            /// 4. LAYER 2: Verify nullifier is UNIQUE (not already registered)
            // This is the PRIMARY uniqueness check - simple storage lookup
            ensure!(
                !PersonhoodRegistry::<T>::contains_key(nullifier),
                Error::<T>::NullifierAlreadyUsed
            );

            // 5. LAYER 3: Verify ZK proof (if provided)
            // Proves "I have a valid biometric" without revealing it
            if proof_bytes.len() > 32 {
                let zk_proof_data = &proof_bytes[32..];
                Self::verify_biometric_zk_proof(nullifier, commitment, zk_proof_data)?;
            }

            Ok(())
        }

        /// Verify ZK proof that biometric is valid and unique
        fn verify_biometric_zk_proof(
            nullifier: &H256,
            commitment: &H256,
            proof_bytes: &[u8],
        ) -> Result<(), Error<T>> {
            // Convert to bounded format
            let bounded_proof: BoundedVec<u8, ConstU32<4096>> = proof_bytes
                .to_vec()
                .try_into()
                .map_err(|_| Error::<T>::InvalidUniquenessProof)?;
            
            // Public inputs: nullifier and commitment
            // These are visible to everyone, but don't reveal biometric
            let mut public_inputs = Vec::new();
            
            // Add nullifier as public input
            public_inputs.push(
                nullifier.as_bytes().to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidUniquenessProof)?
            );
            
            // Add commitment as public input
            public_inputs.push(
                commitment.as_bytes().to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidUniquenessProof)?
            );
            
            let bounded_inputs: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<16>> = 
                public_inputs
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidUniquenessProof)?;
            
            // Create ZK proof structure
            let zk_proof = pallet_zk_credentials::ZkProof {
                proof_type: pallet_zk_credentials::ProofType::Personhood,
                proof_data: bounded_proof,
                public_inputs: bounded_inputs,
                credential_hash: *commitment, // Use commitment as credential hash
                created_at: <T as Config>::TimeProvider::now().saturated_into::<u64>(),
                nonce: *nullifier, // Use nullifier as nonce to prevent replay
            };
            
            // Verify via ZK credentials pallet
            pallet_zk_credentials::Pallet::<T::ZkCredentials>::verify_proof_internal(&zk_proof)
                .map_err(|_| Error::<T>::InvalidUniquenessProof)?;
            
            Ok(())
        }

        /// CROSS-CHAIN VERIFICATION: Generate Merkle proof of registration
        /// This allows other parachains to verify someone is registered
        /// without needing to query this chain's full state
        pub fn generate_existence_proof(nullifier: &H256) -> Result<Vec<Vec<u8>>, Error<T>> {
            use sp_trie::{generate_trie_proof, LayoutV1};
            use sp_core::Blake2Hasher;
            
            // 1. Ensure nullifier exists
            ensure!(
                PersonhoodRegistry::<T>::contains_key(nullifier),
                Error::<T>::PersonhoodProofNotFound
            );
            
            // 2. Get storage root hash
            let root = sp_io::storage::root(sp_runtime::StateVersion::V1);
            
            // 3. Build storage key for this nullifier
            // This is how Substrate stores the PersonhoodRegistry map
            let storage_key = Self::storage_key_for_nullifier(nullifier);
            
            // 4. Generate trie proof
            // This creates a minimal proof that this key exists in the state trie
            let backend = sp_state_machine::Backend::<Blake2Hasher>::as_trie_backend()
                .ok_or(Error::<T>::InvalidUniquenessProof)?;
            
            let proof = generate_trie_proof::<LayoutV1<Blake2Hasher>, _, _, _>(
                backend,
                root,
                &[&storage_key[..]]
            ).map_err(|_| Error::<T>::InvalidUniquenessProof)?;
            
            Ok(proof.into_iter_nodes().map(|n| n.to_vec()).collect())
        }
        
        /// CROSS-CHAIN VERIFICATION: Verify Merkle proof from another chain
        /// Allows this chain to verify someone is registered on another parachain
        pub fn verify_existence_proof(
            nullifier: &H256,
            state_root: H256,
            proof_nodes: Vec<Vec<u8>>,
        ) -> Result<bool, Error<T>> {
            use sp_trie::{verify_trie_proof, LayoutV1};
            use sp_core::Blake2Hasher;
            
            // Build storage key
            let storage_key = Self::storage_key_for_nullifier(nullifier);
            
            // Verify the proof
            let result = verify_trie_proof::<LayoutV1<Blake2Hasher>, _, _, _>(
                &state_root,
                &proof_nodes,
                &[(storage_key.as_slice(), None)], // Checking key exists
            );
            
            match result {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        
        /// Helper: Generate storage key for a nullifier
        /// This matches Substrate's storage key format
        fn storage_key_for_nullifier(nullifier: &H256) -> Vec<u8> {
            use sp_io::hashing::twox_128;
            use frame_support::storage::generator::StorageMap as _;
            
            // Format: twox128("ProofOfPersonhood") + twox128("PersonhoodRegistry") + blake2_128(nullifier) + nullifier
            let mut key = Vec::new();
            
            // Pallet prefix
            key.extend_from_slice(&twox_128(b"ProofOfPersonhood"));
            
            // Storage item prefix
            key.extend_from_slice(&twox_128(b"PersonhoodRegistry"));
            
            // Key hash (Blake2_128Concat uses Blake2_128 + original key)
            key.extend_from_slice(&sp_io::hashing::blake2_128(nullifier.as_bytes()));
            key.extend_from_slice(nullifier.as_bytes());
            
            key
        }

        /// Enhanced recovery proof verification with ZK
        fn verify_recovery_proof(
            old_did: &H256,
            new_nullifier: &H256,
            proof_bytes: &[u8],
        ) -> Result<(), Error<T>> {
            // Basic validation
            ensure!(
                !proof_bytes.is_empty() && proof_bytes.len() <= 4096,
                Error::<T>::InvalidRecoveryProof
            );

            // Get old nullifier
            let old_nullifier = DidToNullifier::<T>::get(old_did)
                .ok_or(Error::<T>::DidNotFound)?;

            // If ZK proof provided, verify it
            if proof_bytes.len() > 64 {
                let bounded_proof: BoundedVec<u8, ConstU32<4096>> = proof_bytes
                    .to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidRecoveryProof)?;
                
                // Public inputs: old_nullifier and new_nullifier
                let mut public_inputs = Vec::new();
                public_inputs.push(
                    old_nullifier.as_bytes().to_vec()
                        .try_into()
                        .map_err(|_| Error::<T>::InvalidRecoveryProof)?
                );
                public_inputs.push(
                    new_nullifier.as_bytes().to_vec()
                        .try_into()
                        .map_err(|_| Error::<T>::InvalidRecoveryProof)?
                );
                
                let bounded_inputs: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<16>> = 
                    public_inputs
                        .try_into()
                        .map_err(|_| Error::<T>::InvalidRecoveryProof)?;
                
                let zk_proof = pallet_zk_credentials::ZkProof {
                    proof_type: pallet_zk_credentials::ProofType::Recovery,
                    proof_data: bounded_proof,
                    public_inputs: bounded_inputs,
                    credential_hash: *old_did,
                    created_at: <T as Config>::TimeProvider::now().saturated_into::<u64>(),
                    nonce: *new_nullifier,
                };
                
                // Verify the recovery proof links old and new identities
                pallet_zk_credentials::Pallet::<T::ZkCredentials>::verify_proof_internal(&zk_proof)
                    .map_err(|_| Error::<T>::InvalidRecoveryProof)?;
            } else {
                // Fallback: Simple hash-based verification (less secure)
                let mut data = Vec::new();
                data.extend_from_slice(old_did.as_bytes());
                data.extend_from_slice(new_nullifier.as_bytes());
                data.extend_from_slice(proof_bytes);
                
                let proof_hash = sp_io::hashing::blake2_256(&data);
                
                ensure!(
                    proof_hash != [0u8; 32],
                    Error::<T>::InvalidRecoveryProof
                );
            }

            Ok(())
        }

        /// Batch verify multiple existence proofs (for cross-chain efficiency)
        pub fn batch_verify_existence_proofs(
            nullifiers: Vec<H256>,
            state_root: H256,
            proof_nodes: Vec<Vec<u8>>,
        ) -> Result<Vec<bool>, Error<T>> {
            use sp_trie::{verify_trie_proof, LayoutV1};
            use sp_core::Blake2Hasher;
            
            let keys: Vec<Vec<u8>> = nullifiers
                .iter()
                .map(|n| Self::storage_key_for_nullifier(n))
                .collect();
            
            let key_refs: Vec<(&[u8], Option<&[u8]>)> = keys
                .iter()
                .map(|k| (k.as_slice(), None))
                .collect();
            
            let result = verify_trie_proof::<LayoutV1<Blake2Hasher>, _, _, _>(
                &state_root,
                &proof_nodes,
                &key_refs,
            );
            
            match result {
                Ok(items) => Ok(items.into_iter().map(|(_, v)| v.is_some()).collect()),
                Err(_) => Ok(vec![false; nullifiers.len()]),
            }
        }

        fn calculate_recovery_score(
            recovery: &ProgressiveRecoveryRequest<T>,
            now: u64,
        ) -> u32 {
            let mut score: u32 = 0;
            
            // Component 1: Guardian votes (0-30 points)
            let guardian_score: u32 = recovery.guardian_votes.iter()
                .map(|(guardian, vote_strength)| {
                    GuardianRelationships::<T>::get(&recovery.did, guardian)
                        .map(|rel| {
                            let base = (*vote_strength as u32) * (rel.relationship_strength as u32);
                            // Older relationships get bonus
                            let age_bonus = if now.saturating_sub(rel.established_at) > (365 * 24 * 60 * 60) {
                                2
                            } else {
                                0
                            };
                            base + age_bonus
                        })
                        .unwrap_or(0)
                })
                .sum();
            score = score.saturating_add(guardian_score.min(30));
            
            // Component 2: Behavioral biometric (0-30 points)
            let behavioral_score = (recovery.behavioral_confidence as u32 * 30) / 100;
            score = score.saturating_add(behavioral_score);
            
            // Component 3: Historical proof (0-20 points)
            let historical_score = (recovery.historical_proof_strength as u32 * 20) / 100;
            score = score.saturating_add(historical_score);
            
            // Component 4: Economic stake (0-20 points)
            let stake_score = {
                let stake_u128 = recovery.economic_stake.saturated_into::<u128>();
                ((stake_u128 / 1000) as u32).min(20)
            };
            score = score.saturating_add(stake_score);
            
            // Component 5: Time elapsed (0-30 points)
            let elapsed = now.saturating_sub(recovery.requested_at);
            let time_score = if elapsed >= recovery.finalization_delay {
                30
            } else {
                // Linear growth: 30 points over finalization_delay period
                ((elapsed as u128 * 30) / recovery.finalization_delay as u128) as u32
            };
            score = score.saturating_add(time_score);
            
            score
        }
        
        /// Verify behavioral pattern
        fn verify_behavioral_pattern(
            did: &H256,
            pattern_data: &[u8],
        ) -> Result<u8, Error<T>> {
            // Get stored patterns
            let stored_patterns = BehavioralPatterns::<T>::get(did);
            
            if stored_patterns.is_empty() {
                return Ok(0); // No patterns to compare
            }
            
            // Hash provided pattern
            let provided_hash: H256 = sp_io::hashing::blake2_256(pattern_data).into();
            
            // Check if matches any stored pattern
            if stored_patterns.contains(&provided_hash) {
                Ok(100) // Perfect match
            } else {
                // In production, we will use ML model for fuzzy matching
                // For now: simple hamming distance
                let mut best_match: u8 = 0;
                for stored in stored_patterns.iter() {
                    let similarity = Self::calculate_hash_similarity(&provided_hash, stored);
                    if similarity > best_match {
                        best_match = similarity;
                    }
                }
                Ok(best_match)
            }

            /// Check if nullifier is part of ANY personhood
            pub fn get_personhood_for_nullifier(nullifier: &H256) -> Option<H256> {
                BiometricBindings::<T>::get(nullifier)
            }
            
            /// Verify cross-biometric ZK proof
            fn verify_cross_biometric_proof(
                existing_nullifier: &H256,
                new_nullifier: &H256,
                proof: &CrossBiometricProof,
            ) -> Result<(), Error<T>> {
                // Verify proof references correct nullifiers
                ensure!(
                    proof.nullifier_a == *existing_nullifier && proof.nullifier_b == *new_nullifier,
                    Error::<T>::InvalidCrossBiometricProof
                );
                
                // Convert to ZK proof format
                let bounded_proof: BoundedVec<u8, ConstU32<4096>> = proof.zk_binding_proof.clone();
                
                // Public inputs: both nullifiers + session_id
                let mut public_inputs = Vec::new();
                public_inputs.push(
                    existing_nullifier.as_bytes().to_vec()
                        .try_into()
                        .map_err(|_| Error::<T>::InvalidCrossBiometricProof)?
                );
                public_inputs.push(
                    new_nullifier.as_bytes().to_vec()
                        .try_into()
                        .map_err(|_| Error::<T>::InvalidCrossBiometricProof)?
                );
                public_inputs.push(
                    proof.session_id.as_bytes().to_vec()
                        .try_into()
                        .map_err(|_| Error::<T>::InvalidCrossBiometricProof)?
                );
                
                let bounded_inputs: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<16>> = 
                    public_inputs
                        .try_into()
                        .map_err(|_| Error::<T>::InvalidCrossBiometricProof)?;
                
                // Create ZK proof structure
                let zk_proof = pallet_zk_credentials::ZkProof {
                    proof_type: pallet_zk_credentials::ProofType::CrossBiometric,
                    proof_data: bounded_proof,
                    public_inputs: bounded_inputs,
                    credential_hash: *existing_nullifier,
                    created_at: proof.captured_at,
                    nonce: *new_nullifier,
                };
                
                // Verify via ZK credentials pallet
                pallet_zk_credentials::Pallet::<T::ZkCredentials>::verify_proof_internal(&zk_proof)
                    .map_err(|_| Error::<T>::InvalidCrossBiometricProof)?;
                
                Ok(())
            }
            
            /// SECURITY CHECK: Verify credential issuance doesn't create duplicate personhoods
            pub fn verify_single_personhood_for_credential(
                issuer_did: &H256,
                subject_did: &H256,
            ) -> Result<(), Error<T>> {
                // Get subject's nullifier
                let subject_nullifier = DidToNullifier::<T>::get(subject_did)
                    .ok_or(Error::<T>::DidNotFound)?;
                
                // Check if this nullifier is part of a personhood binding
                if let Some(primary_did) = Self::get_personhood_for_nullifier(&subject_nullifier) {
                    // Ensure this is the primary DID or a legitimate bound DID
                    ensure!(
                        primary_did == *subject_did,
                        Error::<T>::NullifierAlreadyBound
                    );
                    Ok(())
                } else {
                    // No personhood registered yet - this is the first credential
                    Ok(())
                }
            }
        }
        
        /// Calculate similarity between two hashes (0-100)
        fn calculate_hash_similarity(hash1: &H256, hash2: &H256) -> u8 {
            let bytes1 = hash1.as_bytes();
            let bytes2 = hash2.as_bytes();
            
            let mut matching_bits = 0u32;
            for (b1, b2) in bytes1.iter().zip(bytes2.iter()) {
                matching_bits += (b1 ^ b2).count_zeros();
            }
            
            ((matching_bits * 100) / 256) as u8
        }
        
        /// Verify historical access proof
        fn verify_historical_proof(
            did: &H256,
            proof_data: &[u8],
        ) -> Result<u8, Error<T>> {
            // Proof format: old_signature || message || timestamp
            if proof_data.len() < 96 {
                return Ok(0);
            }
            
            // In production: verify signature against old keys
            // For now: hash-based verification
            let proof_hash = sp_io::hashing::blake2_256(proof_data);
            
            // Non-zero hash = valid proof (simplified)
            if proof_hash != [0u8; 32] {
                Ok(85) // Good confidence
            } else {
                Ok(0)
            }
        }
        
        /// Verify fraud proof
        fn verify_fraud_proof(
            did: &H256,
            guardian: &T::AccountId,
            proof: &[u8],
        ) -> bool {
            // In production: escalate to governance or oracle network
            // For now: simple check
            !proof.is_empty() && proof.len() >= 32
        }
    }

    /// Validate nullifier format
    fn validate_nullifier(nullifier: &H256) -> bool {
        *nullifier != H256::zero()
    }

    /// Validate commitment format
    fn validate_commitment(commitment: &H256) -> bool {
        *commitment != H256::zero()
    }

    /// Check if personhood is registered
    pub fn is_personhood_registered(did: &H256) -> bool {
        if let Some(nullifier) = DidToNullifier::<T>::get(did) {
            PersonhoodRegistry::<T>::contains_key(&nullifier)
        } else {
            false
        }
    }

    /// Check if account is dormant (no activity for 12 months)
    pub fn is_account_dormant(did: &H256) -> bool {
        let last_active = LastActivity::<T>::get(did);
        let now = <T as Config>::TimeProvider::now().saturated_into::<u64>().saturated_into::<u64>();
        let twelve_months = 12 * 30 * 24 * 60 * 60u64;
        
        now.saturating_sub(last_active) > twelve_months
    }

    /// Get nullifier for DID
    pub fn get_nullifier_for_did(did: &H256) -> Result<H256, Error<T>> {
        DidToNullifier::<T>::get(did).ok_or(Error::<T>::DidNotFound)
    }
}