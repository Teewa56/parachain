#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    
    app_crypto!(sr25519, KEY_TYPE);
    
    pub struct TestAuthId;
    
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::traits::Saturating;
    use pallet_identity_registry::pallet::Identities;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Time},
    };
    use sp_runtime::SaturatedConversion;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_core::H256;
    use pallet_identity_registry;
    use crate::weights::WeightInfo;
    use frame_support::BoundedVec;
    use pallet_zk_credentials;
    use codec::DecodeWithMemTracking;
    use itertools::Itertools;
    use sp_io::crypto::{ sr25519_verify, KeyTypeId };
    use sp_runtime::offchain::{
        http,
        Duration,
    };

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"bbio"); // Behavioral Biometrics

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

    /// Maximum age for fraud proofs (7 days)
    const MAX_FRAUD_PROOF_AGE: u64 = 7 * 24 * 60 * 60;

    /// Suspicious guardian approval threshold
    const MAX_GUARDIAN_APPROVALS: usize = 5;

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
        /// Authority identifier for off-chain worker
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    }

    /// Personhood proof structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, DecodeWithMemTracking)]
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
        pub zk_binding_proof: BoundedVec<u8, ConstU32<8192>>,
        /// Session token (prevents replay attacks)
        pub session_id: H256,
        /// Timestamp when biometrics were captured
        pub captured_at: u64,
    }

    /// Recovery request structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

    /// Historical signature entry for proof verification
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct HistoricalSignature {
        pub timestamp: u64,
        pub signature: [u8; 64],
        pub public_key: [u8; 32],
        pub message_hash: H256,
    }

    /// Behavioral pattern features
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct BehavioralFeatures {
        pub typing_speed_wpm: u32,        // Words per minute
        pub avg_key_hold_time_ms: u32,    // Average key hold time
        pub avg_transition_time_ms: u32,  // Time between key presses
        pub error_rate_percent: u8,       // Typing error rate
        pub common_patterns_hash: H256,   // Hash of common word sequences
        pub activity_hour_preference: u8,  // Preferred hour of day (0-23)
    }

    /// Statistical envelope for pattern matching (2-sigma bounds)
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct BehavioralEnvelope {
        // Mean values
        pub mean_typing_speed: u32,
        pub mean_key_hold_time: u32,
        pub mean_transition_time: u32,
        pub mean_error_rate: u8,
        
        // Standard deviations (fixed-point: value * 100)
        pub std_dev_typing_speed: u32,
        pub std_dev_key_hold_time: u32,
        pub std_dev_transition_time: u32,
        pub std_dev_error_rate: u16,
        
        // 2-sigma thresholds (95% confidence bounds)
        pub min_typing_speed: u32,
        pub max_typing_speed: u32,
        pub min_key_hold_time: u32,
        pub max_key_hold_time: u32,
        pub min_transition_time: u32,
        pub max_transition_time: u32,
        
        pub samples_count: u32,
        pub last_updated: u64,
    }

    /// Feature weights based on research (total = 100)
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct FeatureWeights {
        pub typing_speed: u8,        // 15 - moderate discriminability
        pub key_hold_time: u8,        // 20 - good consistency
        pub transition_time: u8,      // 30 - most distinctive (unconscious)
        pub error_rate: u8,           // 10 - most variable
        pub pattern_hash: u8,         // 15 - common sequences
        pub time_preference: u8,      // 10 - activity patterns
    }

    impl Default for FeatureWeights {
        fn default() -> Self {
            Self {
                typing_speed: 15,
                key_hold_time: 20,
                transition_time: 30,
                error_rate: 10,
                pattern_hash: 15,
                time_preference: 10,
            }
        }
    }

    /// Full behavioral pattern with all features (not just hash)
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct StoredBehavioralPattern {
        /// Actual feature values for direct comparison
        pub features: BehavioralFeatures,
        pub recorded_at: u64,
        pub sample_count: u32,
        pub confidence_score: u8,  // 0-100, increases with more samples
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
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

    /// Store historical public keys for signature verification
    #[pallet::storage]
    #[pallet::getter(fn historical_keys)]
    pub type HistoricalKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        BoundedVec<([u8; 32], u64), ConstU32<20>>, // (public_key, registered_at)
        ValueQuery,
    >;

    /// Storage: DID -> Statistical Envelope
    #[pallet::storage]
    #[pallet::getter(fn behavioral_envelopes)]
    pub type BehavioralEnvelopes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        BehavioralEnvelope,
        OptionQuery,
    >;

    /// Storage: DID -> Vec<Full Patterns> (last 10 samples)
    #[pallet::storage]
    #[pallet::getter(fn behavioral_pattern_samples)]
    pub type BehavioralPatternSamples<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        BoundedVec<StoredBehavioralPattern, ConstU32<10>>,
        ValueQuery,
    >;

    /// Storage for patterns pending ML scoring
    #[pallet::storage]
    #[pallet::getter(fn pending_ml_patterns)]
    pub type PendingMLPatterns<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        BehavioralFeatures,
        OptionQuery,
    >;

    /// Storage for ML scores received from off-chain worker
    #[pallet::storage]
    #[pallet::getter(fn ml_scores)]
    pub type MLScores<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        (u8, u64), // (score, timestamp)
        OptionQuery,
    >;

    // Configuration for ML service endpoint
    #[pallet::storage]
    #[pallet::getter(fn ml_service_url)]
    pub type MLServiceUrl<T: Config> = StorageValue<
        _, 
        BoundedVec<u8, ConstU32<256>>, 
        ValueQuery
    >;

    /// Drift analysis result
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum DriftAnalysis {
        InsufficientData,
        NormalVariation { distance: u32 },
        GradualDrift { distance: u32, accept_update: bool },
        SuddenChange { distance: u32, confidence: u8 },
    }

    /// Biometric modality types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, DecodeWithMemTracking, MaxEncodedLen)]
    pub enum BiometricModality {
        Fingerprint,
        Iris,
        FaceGeometry,
        Voice,
        Gait,
        Retina,
    }

    /// Evidence types for progressive recovery
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, DecodeWithMemTracking)]
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
        DoubleRegistrationAttempt { nullifier: H256, existing_did: H256 },
        HistoricalKeyRegistered { did: H256, key_hash: H256 },
        BehavioralPatternRecorded { did: H256, pattern_hash: H256, sample_count: u32 },
        /// Anomalous behavioral pattern detected (potential takeover)
        AnomalousPatternDetected {
            did: H256,
            distance: u32,
            confidence: u8,
        },
        
        /// Behavioral envelope updated with new sample
        EnvelopeUpdated {
            did: H256,
            samples_count: u32,
        },
        
        /// Pattern rejected (outside 2-sigma envelope)
        PatternRejected {
            did: H256,
            violations: Vec<u8>,
        },
        /// ML confidence score stored
        MLScoreStored {
            did: H256,
            score: u8,
            timestamp: u64,
        },        
        /// Pattern queued for ML scoring
        PatternQueuedForML { did: H256 },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidProof,
        NullifierAlreadyUsed,
        DidNotFound,
        NotAuthorized,
        RecoveryRequestNotFound,
        RecoveryPeriodNotElapsed,
        NotAGuardian,
        InsufficientGuardianApprovals,
        PersonhoodProofNotFound,
        InvalidRecoveryProof,
        InvalidUniquenessProof,
        RegistrationTooSoon,
        InsufficientDeposit,
        RecoveryAlreadyActive,
        InvalidNullifier,
        InvalidCommitment,
        GuardianAlreadyExists,
        InvalidRelationshipStrength,
        InsufficientGuardianBond,
        GuardianNotFound,
        ExceededVotingPower,
        ProgressiveRecoveryNotFound,
        RecoveryScoreInsufficient,
        InvalidBehavioralProof,
        InvalidHistoricalProof,
        RecoveryInProgress,
        NullifierAlreadyBound,
        InvalidCrossBiometricProof,
        SessionTokenUsed,
        SessionTokenExpired,
        ModalityAlreadyRegistered,
        InvalidBiometricModality,
        BindingNotFound,
        MaxBiometricsReached,
        InvalidSignature,
        InvalidPublicKey,
        SignatureTooOld,
        InvalidFeatureData,
        PatternMismatch,
        InvalidMLServiceUrl,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            // Run ML inference every 10 blocks
            if (block_number % 10u32.into()).is_zero() {
                log::info!(" Running ML inference at block {:?}", block_number);
                
                if let Err(e) = Self::run_ml_inference(block_number) {
                    log::error!(" ML inference failed: {:?}", e);
                }
            }
        }
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
            let identity = pallet_identity_registry::pallet::Identities::<T>::get(&did)
                .ok_or(Error::<T>::DidNotFound)?;
            ensure!(who == identity.controller, Error::<T>::NotAuthorized);
            ensure!(identity.active, Error::<T>::NotAuthorized);

            // Check nullifier is unique
            ensure!(
                !PersonhoodRegistry::<T>::contains_key(&nullifier),
                Error::<T>::NullifierAlreadyUsed
            );

            // Check cooldown period
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
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
                uniqueness_proof: uniqueness_proof.try_into().map_err(|_| Error::<T>::InvalidUniquenessProof)?,
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

            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            let active_at = now.saturating_add(RECOVERY_DELAY_SECONDS);

            let guardians_bounded: BoundedVec<T::AccountId, ConstU32<10>> = 
                guardians.clone().try_into().map_err(|_| Error::<T>::NotAuthorized)?;

            let request = RecoveryRequest {
                did: old_did,
                old_nullifier,
                new_nullifier,
                new_commitment,
                recovery_proof: recovery_proof.try_into().map_err(|_| Error::<T>::InvalidRecoveryProof)?,
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
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            ensure!(now >= request.active_at, Error::<T>::RecoveryPeriodNotElapsed);

            // Check guardian approvals (require 2/3 majority)
            let approvals = GuardianApprovals::<T>::get(&did);
            let required = (request.guardians.len() * 2 / 3).saturating_add(1);
            ensure!(
                approvals.len() >= required,
                Error::<T>::InsufficientGuardianApprovals
            );

            // Get old proof
            let _old_proof = PersonhoodRegistry::<T>::get(&request.old_nullifier)
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
            let (did, identity) = pallet_identity_registry::pallet::Pallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::DidNotFound)?;

            ensure!(identity.active, Error::<T>::NotAuthorized);

            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
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
            let identity = Identities::<T>::get(&did)
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
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
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
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
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
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
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
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
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
            
            ensure!(
                Self::verify_fraud_proof(&did, &fraudulent_guardian, &fraud_proof),
                Error::<T>::InvalidRecoveryProof
            );
            
            let relationship = GuardianRelationships::<T>::get(&did, &fraudulent_guardian)
                .ok_or(Error::<T>::GuardianNotFound)?;
            
            // Slash guardian's bond
            let slashed = T::Currency::slash_reserved(&fraudulent_guardian, relationship.bonded_stake);
            let slashed_balance = slashed.0;
            
            // Calculate reward (50% of slashed amount)
            let divisor: BalanceOf<T> = 2u32.into();
            let reward = slashed_balance / divisor;
            
            T::Currency::deposit_creating(&challenger, reward);
            
            GuardianRelationships::<T>::remove(&did, &fraudulent_guardian);
            
            if let Some(mut recovery) = ProgressiveRecoveries::<T>::get(&did) {
                recovery.guardian_votes.retain(|(g, _)| *g != fraudulent_guardian);
                
                let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
                recovery.recovery_score = Self::calculate_recovery_score(&recovery, now);
                
                ProgressiveRecoveries::<T>::insert(&did, recovery);
            }
            
            Self::deposit_event(Event::GuardianSlashed {
                did,
                guardian: fraudulent_guardian,
                amount: slashed_balance,
            });
            
            Ok(())
        }
        
        /// Record behavioral pattern for future verification
        /// Pattern data should be encoded BehavioralFeatures struct
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::record_behavioral_pattern())]
        pub fn record_behavioral_pattern(
            origin: OriginFor<T>,
            pattern_data: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let (did, identity) = pallet_identity_registry::pallet::Pallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::DidNotFound)?;
            
            ensure!(identity.active, Error::<T>::NotAuthorized);
            
            // Decode behavioral features
            let features = BehavioralFeatures::decode(&mut &pattern_data[..])
                .map_err(|_| Error::<T>::InvalidFeatureData)?;
            
            // Validate features
            ensure!(features.typing_speed_wpm > 0, Error::<T>::InvalidFeatureData);
            ensure!(features.error_rate_percent <= 100, Error::<T>::InvalidFeatureData);
            ensure!(features.activity_hour_preference < 24, Error::<T>::InvalidFeatureData);
            
            Self::record_behavioral_pattern_internal(&did, &features)?;
            
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
            let identity = pallet_identity_registry::pallet::Identities::<T>::get(&did)
                .ok_or(Error::<T>::DidNotFound)?;
            ensure!(who == identity.controller, Error::<T>::NotAuthorized);
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
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
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
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
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

        /// Register a historical public key for future signature verification
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::register_historical_key())]
        pub fn register_historical_key(
            origin: OriginFor<T>,
            public_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Get DID from account
            let (did, identity) = pallet_identity_registry::pallet::Pallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::DidNotFound)?;
            
            ensure!(identity.active, Error::<T>::NotAuthorized);
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            HistoricalKeys::<T>::try_mutate(&did, |keys| -> DispatchResult {
                keys.try_push((public_key, now))
                    .map_err(|_| Error::<T>::InvalidPublicKey)?;
                Ok(())
            })?;
            
            let key_hash: H256 = sp_io::hashing::blake2_256(&public_key).into();
            Self::deposit_event(Event::HistoricalKeyRegistered {
                did,
                key_hash,
            });
            
            Ok(())
        }

        /// Store ML confidence score (called by off-chain worker)
        #[pallet::call_index(15)]
        #[pallet::weight(<T as Config>::WeightInfo::store_ml_score())]
        pub fn store_ml_score(
            origin: OriginFor<T>,
            did: H256,
            score: u8,
        ) -> DispatchResult {
            // Can only be called by off-chain worker
            ensure_none(origin)?;
            
            // Validate score
            ensure!(score <= 100, Error::<T>::InvalidFeatureData);
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            // Store ML score
            MLScores::<T>::insert(&did, (score, now));
            
            // Remove from pending queue
            PendingMLPatterns::<T>::remove(&did);
            
            Self::deposit_event(Event::MLScoreStored {
                did,
                score,
                timestamp: now,
            });
            
            Ok(())
        }
        
        /// Set ML service URL (governance only)
        #[pallet::call_index(16)]
        #[pallet::weight(<T as Config>::WeightInfo::set_ml_service_url())]
        pub fn set_ml_service_url(
            origin: OriginFor<T>,
            url: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let bounded_url: BoundedVec<u8, ConstU32<256>> = url
                .try_into()
                .map_err(|_| Error::<T>::InvalidFeatureData)?;
            
            MLServiceUrl::<T>::put(bounded_url);
            
            Ok(())
        }
        
        /// Queue pattern for ML scoring
        #[pallet::call_index(17)]
        #[pallet::weight(<T as Config>::WeightInfo::queue_for_ml_scoring())]
        pub fn queue_for_ml_scoring(
            origin: OriginFor<T>,
            pattern_data: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Get DID from account
            let (did, identity) = pallet_identity_registry::pallet::Pallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::DidNotFound)?;
            
            ensure!(identity.active, Error::<T>::NotAuthorized);
            
            // Decode features
            let features = BehavioralFeatures::decode(&mut &pattern_data[..])
                .map_err(|_| Error::<T>::InvalidFeatureData)?;
            
            // Store in pending queue
            PendingMLPatterns::<T>::insert(&did, features);
            
            Self::deposit_event(Event::PatternQueuedForML { did });
            
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Main off-chain worker function
        fn run_ml_inference(block_number: T::BlockNumber) -> Result<(), &'static str> {
            // Check if we have signing keys
            let signer = Self::get_signer().ok_or("No signing key available")?;
            
            // Fetch pending patterns
            let pending_patterns = Self::get_pending_ml_patterns();
            
            if pending_patterns.is_empty() {
                log::debug!("No pending patterns to process");
                return Ok(());
            }
            
            log::info!("Processing {} pending patterns", pending_patterns.len());
            
            // Process each pattern
            for (did, features) in pending_patterns.iter() {
                match Self::call_ml_service(features) {
                    Ok(score) => {
                        log::info!("ML score for DID {:?}: {}", did, score);
                        
                        // Submit signed transaction with result
                        if let Err(e) = Self::submit_ml_score_transaction(&signer, *did, score) {
                            log::error!(" Failed to submit ML score for {:?}: {:?}", did, e);
                        }
                    },
                    Err(e) => {
                        log::error!(" ML service call failed for {:?}: {:?}", did, e);
                    }
                }
            }
            
            Ok(())
        }

        /// Get pending patterns that need ML scoring
        fn get_pending_ml_patterns() -> Vec<(H256, BehavioralFeatures)> {
            let mut patterns = Vec::new();
            
            // Iterate through pending patterns
            for (did, features) in PendingMLPatterns::<T>::iter() {
                // Only process if we don't have a recent ML score
                if !Self::has_recent_ml_score(&did) {
                    patterns.push((did, features));
                }
                
                // Limit batch size to avoid timeout
                if patterns.len() >= 10 {
                    break;
                }
            }
            
            patterns
        }
        
        /// Check if DID has a recent ML score (within last 100 blocks)
        fn has_recent_ml_score(did: &H256) -> bool {
            if let Some((_, timestamp)) = MLScores::<T>::get(did) {
                let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
                let age = now.saturating_sub(timestamp);
                let max_age = 100 * 6; // ~10 minutes (assuming 6s blocks)
                
                return age < max_age;
            }
            false
        }
        
        /// Call external ML service via HTTP
        fn call_ml_service(features: &BehavioralFeatures) -> Result<u8, &'static str> {
            // Get ML service URL from storage
            let url_bytes = MLServiceUrl::<T>::get();
            let url = sp_std::str::from_utf8(&url_bytes)
                .map_err(|_| "Invalid ML service URL")?;
            
            log::debug!(" Calling ML service at: {}", url);
            
            // Build request payload
            let payload = Self::build_ml_request_payload(features)?;
            
            // Create HTTP request
            let request = http::Request::post(url, vec![payload]);
            
            // Set timeout (5 seconds)
            let timeout = Duration::from_millis(5000);
            let pending = request
                .deadline(sp_io::offchain::timestamp().add(timeout))
                .send()
                .map_err(|_| "Failed to send HTTP request")?;
            
            // Wait for response
            let response = pending
                .try_wait(timeout)
                .map_err(|_| "Request timeout")?
                .map_err(|_| "Request failed")?;
            
            // Check status code
            if response.code != 200 {
                log::error!(" ML service returned status: {}", response.code);
                return Err("ML service error");
            }
            
            // Parse response
            let body = response.body().collect::<Vec<u8>>();
            Self::parse_ml_response(&body)
        }
        
        /// Build JSON payload for ML service
        fn build_ml_request_payload(features: &BehavioralFeatures) -> Result<u8, &'static str> {
            // Encode features to JSON manually (no_std compatible)
            let mut json = Vec::new();
            
            json.extend_from_slice(b"{");
            json.extend_from_slice(b"\"did\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",");
            json.extend_from_slice(b"\"features\":{");
            
            // typing_speed_wpm
            json.extend_from_slice(b"\"typing_speed_wpm\":");
            json.extend_from_slice(features.typing_speed_wpm.to_string().as_bytes());
            json.extend_from_slice(b",");
            
            // avg_key_hold_time_ms
            json.extend_from_slice(b"\"avg_key_hold_time_ms\":");
            json.extend_from_slice(features.avg_key_hold_time_ms.to_string().as_bytes());
            json.extend_from_slice(b",");
            
            // avg_transition_time_ms
            json.extend_from_slice(b"\"avg_transition_time_ms\":");
            json.extend_from_slice(features.avg_transition_time_ms.to_string().as_bytes());
            json.extend_from_slice(b",");
            
            // error_rate_percent
            json.extend_from_slice(b"\"error_rate_percent\":");
            json.extend_from_slice(features.error_rate_percent.to_string().as_bytes());
            json.extend_from_slice(b",");
            
            // activity_hour_preference
            json.extend_from_slice(b"\"activity_hour_preference\":");
            json.extend_from_slice(features.activity_hour_preference.to_string().as_bytes());
            
            json.extend_from_slice(b"}}");
            
            Ok(json[0]) // Return first byte as placeholder
        }
        
        /// Parse ML service JSON response
        fn parse_ml_response(body: &[u8]) -> Result<u8, &'static str> {
            // Parse JSON response: {"confidence_score": 87, ...}
            // Simple parser for no_std environment
            
            let body_str = sp_std::str::from_utf8(body)
                .map_err(|_| "Invalid UTF-8 response")?;
            
            // Find "confidence_score": field
            if let Some(start) = body_str.find("\"confidence_score\":") {
                let score_str = &body_str[start + 19..]; // Skip key
                
                // Find the number
                let score_str = score_str.trim_start();
                let end = score_str.find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(score_str.len());
                
                let score_str = &score_str[..end];
                
                // Parse to u8
                let score = score_str.parse::<u8>()
                    .map_err(|_| "Invalid score format")?;
                
                if score > 100 {
                    return Err("Score out of range");
                }
                
                return Ok(score);
            }
            
            Err("confidence_score not found in response")
        }
        
        /// Get signer for submitting transactions
        fn get_signer() -> Option<Signer<T, T::AuthorityId, ForAny>> {
            Signer::<T, T::AuthorityId, ForAny>::any_account()
        }
        
        /// Submit ML score via signed transaction
        fn submit_ml_score_transaction(
            signer: &Signer<T, T::AuthorityId, ForAny>,
            did: H256,
            score: u8,
        ) -> Result<(), &'static str> {
            // Submit signed transaction
            let results = signer.send_signed_transaction(|_account| {
                Call::store_ml_score { did, score }
            });
            
            // Check if transaction was submitted
            for (_, result) in &results {
                if result.is_err() {
                    return Err("Failed to submit transaction");
                }
            }
            
            if results.len() == 0 {
                return Err("No account available for signing");
            }
            
            Ok(())
        }

        /// Update behavioral envelope with new sample (Welford's online algorithm)
        pub fn update_behavioral_envelope(
            did: &H256,
            new_features: &BehavioralFeatures,
        ) -> Result<(), Error<T>> {
            BehavioralEnvelopes::<T>::try_mutate(did, |envelope_opt| -> Result<(), Error<T>> {
                let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
                
                match envelope_opt {
                    None => {
                        // First sample - initialize envelope with conservative bounds
                        *envelope_opt = Some(BehavioralEnvelope {
                            mean_typing_speed: new_features.typing_speed_wpm,
                            mean_key_hold_time: new_features.avg_key_hold_time_ms,
                            mean_transition_time: new_features.avg_transition_time_ms,
                            mean_error_rate: new_features.error_rate_percent,
                            std_dev_typing_speed: 1000, // Conservative initial std dev (10 WPM * 100)
                            std_dev_key_hold_time: 2000, // 20ms * 100
                            std_dev_transition_time: 1500, // 15ms * 100
                            std_dev_error_rate: 300, // 3% * 100
                            min_typing_speed: new_features.typing_speed_wpm.saturating_sub(10),
                            max_typing_speed: new_features.typing_speed_wpm.saturating_add(10),
                            min_key_hold_time: new_features.avg_key_hold_time_ms.saturating_sub(20),
                            max_key_hold_time: new_features.avg_key_hold_time_ms.saturating_add(20),
                            min_transition_time: new_features.avg_transition_time_ms.saturating_sub(15),
                            max_transition_time: new_features.avg_transition_time_ms.saturating_add(15),
                            samples_count: 1,
                            last_updated: now,
                        });
                    },
                    Some(envelope) => {
                        let n = envelope.samples_count;
                        let n_plus_1 = n + 1;
                        
                        // Update typing speed mean and variance (Welford's algorithm)
                        let old_mean_typing = envelope.mean_typing_speed;
                        envelope.mean_typing_speed = 
                            ((old_mean_typing as u64 * n as u64 + new_features.typing_speed_wpm as u64) 
                            / n_plus_1 as u64) as u32;
                        
                        // Update variance incrementally
                        if n > 1 {
                            let delta = new_features.typing_speed_wpm as i64 - old_mean_typing as i64;
                            let delta2 = new_features.typing_speed_wpm as i64 - envelope.mean_typing_speed as i64;
                            
                            // M2 = M2 + delta * delta2
                            // variance = M2 / (n - 1)
                            // std_dev = sqrt(variance)
                            // For fixed-point: store std_dev * 100
                            
                            // Simplified: recalculate from samples
                            envelope.std_dev_typing_speed = Self::calculate_std_dev_from_samples(
                                did, 
                                envelope.mean_typing_speed,
                                0 // feature index for typing speed
                            )?;
                        }
                        
                        // Similar updates for other features
                        envelope.mean_key_hold_time = 
                            ((envelope.mean_key_hold_time as u64 * n as u64 + new_features.avg_key_hold_time_ms as u64) 
                            / n_plus_1 as u64) as u32;
                        
                        envelope.mean_transition_time = 
                            ((envelope.mean_transition_time as u64 * n as u64 + new_features.avg_transition_time_ms as u64) 
                            / n_plus_1 as u64) as u32;
                        
                        envelope.mean_error_rate = 
                            ((envelope.mean_error_rate as u32 * n + new_features.error_rate_percent as u32) 
                            / n_plus_1) as u8;
                        
                        // Update 2-sigma bounds
                        let std_typing = envelope.std_dev_typing_speed / 100; // Convert from fixed-point
                        envelope.min_typing_speed = envelope.mean_typing_speed.saturating_sub(2 * std_typing);
                        envelope.max_typing_speed = envelope.mean_typing_speed.saturating_add(2 * std_typing);
                        
                        let std_hold = envelope.std_dev_key_hold_time / 100;
                        envelope.min_key_hold_time = envelope.mean_key_hold_time.saturating_sub(2 * std_hold);
                        envelope.max_key_hold_time = envelope.mean_key_hold_time.saturating_add(2 * std_hold);
                        
                        let std_transition = envelope.std_dev_transition_time / 100;
                        envelope.min_transition_time = envelope.mean_transition_time.saturating_sub(2 * std_transition);
                        envelope.max_transition_time = envelope.mean_transition_time.saturating_add(2 * std_transition);
                        
                        envelope.samples_count = n_plus_1;
                        envelope.last_updated = now;
                    }
                }
                
                Self::deposit_event(Event::EnvelopeUpdated {
                    did: *did,
                    samples_count: envelope_opt.as_ref().unwrap().samples_count,
                });
                
                Ok(())
            })
        }
        
        /// Calculate standard deviation from stored samples
        fn calculate_std_dev_from_samples(
            did: &H256,
            mean: u32,
            feature_index: u8,
        ) -> Result<u32, Error<T>> {
            let samples = BehavioralPatternSamples::<T>::get(did);
            if samples.len() < 2 {
                return Ok(1000); // Conservative default
            }
            
            let mut sum_squared_diff = 0u64;
            
            for sample in samples.iter() {
                let value = match feature_index {
                    0 => sample.features.typing_speed_wpm,
                    1 => sample.features.avg_key_hold_time_ms,
                    2 => sample.features.avg_transition_time_ms,
                    _ => return Ok(1000),
                };
                
                let diff = if value > mean {
                    value - mean
                } else {
                    mean - value
                };
                
                sum_squared_diff += (diff as u64).pow(2);
            }
            
            let variance = sum_squared_diff / (samples.len() as u64 - 1);
            let std_dev = Self::integer_sqrt_u64(variance) as u32;
            
            // Return fixed-point (std_dev * 100)
            Ok(std_dev * 100)
        }
        
        /// 64-bit integer square root
        fn integer_sqrt_u64(n: u64) -> u64 {
            if n < 2 {
                return n;
            }
            let mut x = n;
            let mut y = (x + 1) / 2;
            while y < x {
                x = y;
                y = (x + n / x) / 2;
            }
            x
        }
        
        /// Detect gradual drift vs sudden takeover
        pub fn detect_pattern_drift(
            did: &H256,
            new_features: &BehavioralFeatures,
            historical_samples: &[StoredBehavioralPattern],
        ) -> DriftAnalysis {
            if historical_samples.len() < 3 {
                return DriftAnalysis::InsufficientData;
            }
            
            // Calculate trend over last 5 samples
            let recent_samples = if historical_samples.len() > 5 {
                &historical_samples[historical_samples.len() - 5..]
            } else {
                historical_samples
            };
            
            let distance_from_mean = Self::calculate_distance_from_mean(new_features, historical_samples);
            let follows_trend = Self::is_consistent_with_trend(recent_samples, new_features);
            
            if distance_from_mean > 200 && !follows_trend {
                // Sudden large deviation = potential takeover
                DriftAnalysis::SuddenChange { 
                    distance: distance_from_mean,
                    confidence: 95,
                }
            } else if distance_from_mean > 100 && follows_trend {
                // Gradual change = natural evolution
                DriftAnalysis::GradualDrift {
                    distance: distance_from_mean,
                    accept_update: true,
                }
            } else {
                // Normal variation
                DriftAnalysis::NormalVariation {
                    distance: distance_from_mean,
                }
            }
        }
        
        /// Calculate distance from mean of historical samples
        fn calculate_distance_from_mean(
            features: &BehavioralFeatures,
            samples: &[StoredBehavioralPattern],
        ) -> u32 {
            if samples.is_empty() {
                return 0;
            }
            
            // Calculate means
            let mut sum_typing = 0u64;
            let mut sum_hold = 0u64;
            let mut sum_transition = 0u64;
            
            for sample in samples.iter() {
                sum_typing += sample.features.typing_speed_wpm as u64;
                sum_hold += sample.features.avg_key_hold_time_ms as u64;
                sum_transition += sample.features.avg_transition_time_ms as u64;
            }
            
            let mean_typing = (sum_typing / samples.len() as u64) as u32;
            let mean_hold = (sum_hold / samples.len() as u64) as u32;
            let mean_transition = (sum_transition / samples.len() as u64) as u32;
            
            // Calculate Euclidean distance
            let typing_diff = Self::absolute_diff(features.typing_speed_wpm, mean_typing);
            let hold_diff = Self::absolute_diff(features.avg_key_hold_time_ms, mean_hold);
            let transition_diff = Self::absolute_diff(features.avg_transition_time_ms, mean_transition);
            
            let distance_sq = 
                typing_diff.pow(2) + 
                hold_diff.pow(2) + 
                transition_diff.pow(2);
            
            Self::integer_sqrt(distance_sq)
        }
        
        /// Check if new features follow recent trend
        fn is_consistent_with_trend(
            recent_samples: &[StoredBehavioralPattern],
            new_features: &BehavioralFeatures,
        ) -> bool {
            if recent_samples.len() < 2 {
                return true; // Not enough data
            }
            
            // Calculate linear trend for typing speed
            let oldest = &recent_samples[0].features;
            let newest = &recent_samples[recent_samples.len() - 1].features;
            
            let typing_trend = newest.typing_speed_wpm as i64 - oldest.typing_speed_wpm as i64;
            let predicted_typing = newest.typing_speed_wpm as i64 + typing_trend / 2;
            
            let actual_diff = (new_features.typing_speed_wpm as i64 - predicted_typing).abs();
            
            // Allow 20% deviation from trend
            actual_diff < (predicted_typing.abs() / 5)
        }

        /// Calculate weighted distance between two behavioral feature sets
        pub fn calculate_weighted_distance(
            current: &BehavioralFeatures,
            stored: &BehavioralFeatures,
            weights: &FeatureWeights,
        ) -> u32 {
            let w = weights;
            
            // Normalize features to 0-100 scale
            let typing_diff = Self::normalize_typing_speed_diff(
                current.typing_speed_wpm, 
                stored.typing_speed_wpm
            );
            let hold_diff = Self::normalize_time_diff(
                current.avg_key_hold_time_ms, 
                stored.avg_key_hold_time_ms, 
                200
            );
            let transition_diff = Self::normalize_time_diff(
                current.avg_transition_time_ms, 
                stored.avg_transition_time_ms, 
                150
            );
            let error_diff = Self::absolute_diff(
                current.error_rate_percent, 
                stored.error_rate_percent
            ) as u32;
            let pattern_diff = Self::calculate_hash_similarity(
                &current.common_patterns_hash, 
                &stored.common_patterns_hash
            ) as u32;
            let time_diff = Self::absolute_diff(
                current.activity_hour_preference, 
                stored.activity_hour_preference
            ) as u32;
            
            // Weighted sum of squared differences
            let distance_squared = 
                (typing_diff * typing_diff * w.typing_speed as u32) +
                (hold_diff * hold_diff * w.key_hold_time as u32) +
                (transition_diff * transition_diff * w.transition_time as u32) +
                (error_diff * error_diff * w.error_rate as u32) +
                (pattern_diff * pattern_diff * w.pattern_hash as u32) +
                (time_diff * time_diff * w.time_preference as u32);
            
            // Return square root (fixed-point integer sqrt)
            Self::integer_sqrt(distance_squared)
        }
        
        /// Normalize typing speed difference to 0-100 scale
        /// ±10 WPM = 90% similarity (distance of 10)
        fn normalize_typing_speed_diff(current: u32, stored: u32) -> u32 {
            let diff = Self::absolute_diff(current, stored);
            (diff * 10).min(100)  // 10 WPM = 100 distance points
        }
        
        /// Normalize time differences (ms)
        fn normalize_time_diff(current: u32, stored: u32, max_expected: u32) -> u32 {
            let diff = Self::absolute_diff(current, stored);
            ((diff * 100) / max_expected).min(100)
        }
        
        /// Calculate absolute difference (safe subtraction)
        fn absolute_diff(a: u32, b: u32) -> u32 {
            if a > b {
                a.saturating_sub(b)
            } else {
                b.saturating_sub(a)
            }
        }
        
        /// Integer square root using Newton's method
        fn integer_sqrt(n: u32) -> u32 {
            if n < 2 {
                return n;
            }
            let mut x = n;
            let mut y = (x + 1) / 2;
            while y < x {
                x = y;
                y = (x + n / x) / 2;
            }
            x
        }

        /// Calculate match confidence (0-100)
        pub fn calculate_match_confidence(
            distance: u32,
            envelope: &BehavioralEnvelope,
            samples_available: u32,
        ) -> u8 {
            // Convert distance to similarity (inverse relationship)
            // Distance of 0 = 100% similarity, distance of 100 = 0% similarity
            let base_similarity = if distance >= 100 {
                0
            } else {
                100 - distance
            };
            
            // Confidence boost based on number of samples
            let sample_confidence = match samples_available {
                0..=1 => 50,   // Low confidence with 1 sample
                2..=3 => 70,   // Medium confidence
                4..=5 => 85,   // Good confidence
                6..=10 => 95,  // High confidence
                _ => 100,      // Maximum confidence
            };
            
            // Combined confidence: weighted average (70% similarity, 30% samples)
            let confidence = ((base_similarity as u32 * 70) + (sample_confidence as u32 * 30)) / 100;
            
            confidence.min(100) as u8
        }
        
        /// Check if pattern falls within 2-sigma envelope
        pub fn is_within_envelope(
            features: &BehavioralFeatures,
            envelope: &BehavioralEnvelope,
        ) -> (bool, Vec<u8>) {
            let mut violations = Vec::new();
            
            // Check typing speed
            if features.typing_speed_wpm < envelope.min_typing_speed 
                || features.typing_speed_wpm > envelope.max_typing_speed {
                violations.push(0); // Feature index 0
            }
            
            // Check key hold time
            if features.avg_key_hold_time_ms < envelope.min_key_hold_time 
                || features.avg_key_hold_time_ms > envelope.max_key_hold_time {
                violations.push(1); // Feature index 1
            }
            
            // Check transition time
            if features.avg_transition_time_ms < envelope.min_transition_time 
                || features.avg_transition_time_ms > envelope.max_transition_time {
                violations.push(2); // Feature index 2
            }
            
            let is_within = violations.is_empty();
            (is_within, violations)
        }

        /// Check if nullifier is part of any personhood
        pub fn get_personhood_for_nullifier(nullifier: &H256) -> Option<H256> {
            BiometricBindings::<T>::get(nullifier)
        }
        
        /// Verify cross-biometric ZK proof
        fn verify_cross_biometric_proof(
            existing_nullifier: &H256,
            new_nullifier: &H256,
            proof: &CrossBiometricProof,
        ) -> Result<(), Error<T>> {
            ensure!(
                proof.nullifier_a == *existing_nullifier && proof.nullifier_b == *new_nullifier,
                Error::<T>::InvalidCrossBiometricProof
            );
            
            let bounded_proof: BoundedVec<u8, ConstU32<8192>> = proof.zk_binding_proof.clone();
            
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
            
            let zk_proof = pallet_zk_credentials::ZkProof {
                proof_type: pallet_zk_credentials::ProofType::CrossBiometric,
                proof_data: bounded_proof,
                public_inputs: bounded_inputs,
                credential_hash: *existing_nullifier,
                created_at: proof.captured_at,
                nonce: *new_nullifier,
            };
            
            pallet_zk_credentials::Pallet::<T::ZkCredentials>::verify_proof_internal(&zk_proof)
                .map_err(|_| Error::<T>::InvalidCrossBiometricProof)?;
            
            Ok(())
        }
        
        /// SECURITY CHECK: Verify credential issuance doesn't create duplicate personhoods
        pub fn verify_single_personhood_for_credential(
            _issuer_did: &H256,
            subject_did: &H256,
        ) -> Result<(), Error<T>> {
            let subject_nullifier = DidToNullifier::<T>::get(subject_did)
                .ok_or(Error::<T>::DidNotFound)?;
            
            if let Some(primary_did) = Self::get_personhood_for_nullifier(&subject_nullifier) {
                ensure!(
                    primary_did == *subject_did,
                    Error::<T>::NullifierAlreadyBound
                );
            }
            Ok(())
        }

        /// Verify uniqueness proof
        fn verify_uniqueness_proof(
            nullifier: &H256,
            commitment: &H256,
            proof_bytes: &[u8],
        ) -> Result<(), Error<T>> {
            ensure!(proof_bytes.len() >= 64, Error::<T>::InvalidUniquenessProof);
            
            let salt = &proof_bytes[0..32];
            
            let mut preimage = Vec::new();
            preimage.extend_from_slice(nullifier.as_bytes());
            preimage.extend_from_slice(salt);
            let computed_commitment = sp_io::hashing::blake2_256(&preimage);
            
            ensure!(
                computed_commitment == commitment.as_bytes(),
                Error::<T>::InvalidCommitment
            );
            
            ensure!(
                !PersonhoodRegistry::<T>::contains_key(nullifier),
                Error::<T>::NullifierAlreadyUsed
            );

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
            let bounded_proof: BoundedVec<u8, ConstU32<4096>> = proof_bytes
                .to_vec()
                .try_into()
                .map_err(|_| Error::<T>::InvalidUniquenessProof)?;
            
            let mut public_inputs = Vec::new();
            
            public_inputs.push(
                nullifier.as_bytes().to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidUniquenessProof)?
            );
            
            public_inputs.push(
                commitment.as_bytes().to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidUniquenessProof)?
            );
            
            let bounded_inputs: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<16>> = 
                public_inputs
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidUniquenessProof)?;
            
            // Fixed: Pad proof data to 8192 bytes without using itertools
            let mut proof_vec = bounded_proof.to_vec();
            proof_vec.resize(8192, 0u8);
            let padded_proof: BoundedVec<u8, ConstU32<8192>> = proof_vec
                .try_into()
                .map_err(|_| Error::<T>::InvalidProof)?;
            
            let zk_proof = pallet_zk_credentials::ZkProof {
                proof_type: pallet_zk_credentials::ProofType::Personhood,
                proof_data: padded_proof,
                public_inputs: bounded_inputs,
                credential_hash: *commitment,
                created_at: <T as Config>::TimeProvider::now().saturated_into::<u64>(),
                nonce: *nullifier,
            };
            
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
            let root: H256 = sp_io::storage::root(sp_runtime::StateVersion::V1).into();
            
            // 3. Build storage key for this nullifier
            let storage_key = Self::storage_key_for_nullifier(nullifier);
            
            // 4. Generate trie proof: This creates a minimal proof that this key exists in the state trie
            let backend = <dyn sp_state_machine::Backend::<Blake2Hasher>>::as_trie_backend()
                .ok_or(Error::<T>::InvalidUniquenessProof)?;
            
            let proof = generate_trie_proof::<LayoutV1<Blake2Hasher>, _, _, _>(
                backend,
                root,
                &[&storage_key[..]]
            ).map_err(|_| Error::<T>::InvalidUniquenessProof)?;
            
            Ok(proof)
        }

        /// Register historical key for signature verification
        pub fn register_historical_key(
            did: &H256,
            public_key: [u8; 32],
        ) -> DispatchResult {
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            HistoricalKeys::<T>::try_mutate(did, |keys| -> DispatchResult {
                keys.try_push((public_key, now))
                    .map_err(|_| Error::<T>::InvalidPublicKey)?;
                Ok(())
            })?;
            
            let key_hash: H256 = sp_io::hashing::blake2_256(&public_key).into();
            Self::deposit_event(Event::HistoricalKeyRegistered {
                did: *did,
                key_hash,
            });
            
            Ok(())
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
                &[(storage_key.as_slice(), None::<&[u8]>)],
            );
            
            match result {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        
        /// Helper: Generate storage key for a nullifier
        fn storage_key_for_nullifier(nullifier: &H256) -> Vec<u8> {
            use sp_io::hashing::twox_128;
            
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

        fn verify_recovery_proof(
            old_did: &H256,
            new_nullifier: &H256,
            proof_bytes: &[u8],
        ) -> Result<(), Error<T>> {
            ensure!(
                !proof_bytes.is_empty() && proof_bytes.len() <= 4096,
                Error::<T>::InvalidRecoveryProof
            );

            let old_nullifier = DidToNullifier::<T>::get(old_did)
                .ok_or(Error::<T>::DidNotFound)?;

            if proof_bytes.len() > 64 {
                let bounded_proof: BoundedVec<u8, ConstU32<8192>> = proof_bytes
                    .to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidRecoveryProof)?;
                
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
                    proof_type: pallet_zk_credentials::ProofType::Personhood,
                    proof_data: bounded_proof,
                    public_inputs: bounded_inputs,
                    credential_hash: *old_did,
                    created_at: <T as Config>::TimeProvider::now().saturated_into::<u64>(),
                    nonce: *new_nullifier,
                };
                
                pallet_zk_credentials::Pallet::<T::ZkCredentials>::verify_proof_internal(&zk_proof)
                    .map_err(|_| Error::<T>::InvalidRecoveryProof)?;
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
                Ok(_) => {
                    // All keys verified, return true for each
                    Ok(vec![true; nullifiers.len()])
                },
                Err(_) => Ok(vec![false; nullifiers.len()]),
            }
        }

        fn calculate_recovery_score(
            recovery: &ProgressiveRecoveryRequest<T>,
            now: u64,
        ) -> u32 {
            let mut score: u32 = 0;
            
            let guardian_score: u32 = recovery.guardian_votes.iter()
                .map(|(guardian, vote_strength)| {
                    GuardianRelationships::<T>::get(&recovery.did, guardian)
                        .map(|rel| {
                            let base = (*vote_strength as u32) * (rel.relationship_strength as u32);
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
            
            let behavioral_score = (recovery.behavioral_confidence as u32 * 30) / 100;
            score = score.saturating_add(behavioral_score);
            
            let historical_score = (recovery.historical_proof_strength as u32 * 20) / 100;
            score = score.saturating_add(historical_score);
            
            let stake_score = {
                let stake_u128 = recovery.economic_stake.saturated_into::<u128>();
                ((stake_u128 / 1000) as u32).min(20)
            };
            score = score.saturating_add(stake_score);
            
            let elapsed = now.saturating_sub(recovery.requested_at);
            let time_score = if elapsed >= recovery.finalization_delay {
                30
            } else {
                ((elapsed as u128 * 30) / recovery.finalization_delay as u128) as u32
            };
            score = score.saturating_add(time_score);
            
            score
        }
        
        /// Verify behavioral pattern with feature analysis
        pub fn verify_behavioral_pattern(
            did: &H256,
            pattern_data: &[u8],
        ) -> Result<u8, Error<T>> {
            // Decode features
            let features = BehavioralFeatures::decode(&mut &pattern_data[..])
                .map_err(|_| Error::<T>::InvalidFeatureData)?;
            
            // Validate features
            ensure!(features.typing_speed_wpm > 0, Error::<T>::InvalidFeatureData);
            ensure!(features.error_rate_percent <= 100, Error::<T>::InvalidFeatureData);
            ensure!(features.activity_hour_preference < 24, Error::<T>::InvalidFeatureData);
            
            // Get stored samples and envelope
            let samples = BehavioralPatternSamples::<T>::get(did);
            let envelope = BehavioralEnvelopes::<T>::get(did);
            
            if samples.is_empty() {
                // No baseline - store this as first sample
                Self::store_behavioral_sample(did, &features)?;
                return Ok(0); // Return 0 confidence (no baseline to compare)
            }
            
            // STEP 1: Quick rejection - check if within statistical envelope
            if let Some(env) = &envelope {
                let (within_bounds, violations) = Self::is_within_envelope(&features, env);
                if !within_bounds && violations.len() > 2 {
                    // Multiple feature violations = likely not the same person
                    Self::deposit_event(Event::PatternRejected {
                        did: *did,
                        violations: violations.clone(),
                    });
                    return Ok(0);
                }
            }
            
            // STEP 2: Calculate weighted distance to each stored sample
            let weights = FeatureWeights::default();
            let mut min_distance = u32::MAX;
            let mut best_match_age = 0u64;
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            for stored_pattern in samples.iter() {
                let distance = Self::calculate_weighted_distance(
                    &features,
                    &stored_pattern.features,
                    &weights,
                );
                
                if distance < min_distance {
                    min_distance = distance;
                    best_match_age = now.saturating_sub(stored_pattern.recorded_at);
                }
            }
            
            // STEP 3: Calculate base confidence score
            let base_confidence = Self::calculate_match_confidence(
                min_distance,
                envelope.as_ref().unwrap(),
                samples.len() as u32,
            );
            
            // STEP 4: Apply temporal decay (patterns older than 90 days lose confidence)
            let ninety_days = 90 * 24 * 60 * 60u64;
            let decay_factor = if best_match_age < ninety_days {
                100u32
            } else {
                let excess_age = best_match_age.saturating_sub(ninety_days);
                let decay = (excess_age * 50) / (365 * 24 * 60 * 60); // 50% decay over a year
                100u32.saturating_sub(decay as u32)
            };
            
            let final_confidence = ((base_confidence as u32 * decay_factor) / 100) as u8;
            
            // STEP 5: Detect drift (for adaptive learning)
            let drift = Self::detect_pattern_drift(did, &features, &samples[..]);
            match drift {
                DriftAnalysis::SuddenChange { distance, confidence } => {
                    // Log potential takeover attempt
                    Self::deposit_event(Event::AnomalousPatternDetected {
                        did: *did,
                        distance,
                        confidence,
                    });
                    // Don't update envelope for sudden changes
                },
                DriftAnalysis::GradualDrift { accept_update, .. } => {
                    if accept_update && final_confidence > 70 {
                        // Natural evolution - update envelope and store sample
                        let _ = Self::update_behavioral_envelope(did, &features);
                        let _ = Self::store_behavioral_sample(did, &features);
                    }
                },
                DriftAnalysis::NormalVariation { .. } => {
                    // Normal variation - store sample if confidence is high
                    if final_confidence > 80 {
                        let _ = Self::store_behavioral_sample(did, &features);
                    }
                },
                DriftAnalysis::InsufficientData => {
                    // Not enough historical data yet
                    let _ = Self::store_behavioral_sample(did, &features);
                }
            }
            
            Ok(final_confidence)
        }

        /// Store a new behavioral sample (maintains rolling window of 10)
        fn store_behavioral_sample(
            did: &H256,
            features: &BehavioralFeatures,
        ) -> Result<(), Error<T>> {
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            BehavioralPatternSamples::<T>::try_mutate(did, |samples| -> Result<(), Error<T>> {
                let new_sample = StoredBehavioralPattern {
                    features: features.clone(),
                    recorded_at: now,
                    sample_count: 1,
                    confidence_score: 0,
                };
                
                // If at capacity, remove oldest
                if samples.len() >= 10 {
                    samples.remove(0);
                }
                
                samples.try_push(new_sample)
                    .map_err(|_| Error::<T>::InvalidBehavioralProof)?;
                
                Ok(())
            })?;
            
            // Update envelope
            Self::update_behavioral_envelope(did, features)?;
            
            Ok(())
        }
        
        /// Calculate hash similarity (Hamming distance based)
        fn calculate_hash_similarity(hash1: &H256, hash2: &H256) -> u8 {
            let bytes1 = hash1.as_bytes();
            let bytes2 = hash2.as_bytes();
            
            let mut matching_bits = 0u32;
            for (b1, b2) in bytes1.iter().zip(bytes2.iter()) {
                matching_bits += (b1 ^ b2).count_zeros();
            }
            
            ((matching_bits * 100) / 256) as u8
        }
        
        /// Verify historical access proof with real cryptographic signatures
        fn verify_historical_proof(
            did: &H256,
            proof_data: &[u8],
        ) -> Result<u8, Error<T>> {
            // Proof format: [count: 1][signatures: Vec<HistoricalSignature>]
            if proof_data.is_empty() {
                return Ok(0);
            }
            
            let signature_count = proof_data[0] as usize;
            if signature_count == 0 {
                return Ok(0);
            }
            
            // Each signature: 8 (timestamp) + 64 (signature) + 32 (pubkey) + 32 (msg_hash) = 136 bytes
            let required_len = 1 + (signature_count * 136);
            if proof_data.len() < required_len {
                return Err(Error::<T>::InvalidHistoricalProof);
            }
            
            // Get stored historical keys for this DID
            let historical_keys = HistoricalKeys::<T>::get(did);
            if historical_keys.is_empty() {
                return Ok(0);
            }
            
            let mut verified_count = 0u32;
            let mut oldest_verified_timestamp = 0u64;
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            for i in 0..signature_count {
                let offset = 1 + (i * 136);
                
                let timestamp = u64::from_le_bytes(
                    proof_data[offset..offset + 8].try_into()
                        .map_err(|_| Error::<T>::InvalidHistoricalProof)?
                );
                
                let signature_bytes = &proof_data[offset + 8..offset + 72];
                let public_key_bytes = &proof_data[offset + 72..offset + 104];
                let message_hash_bytes = &proof_data[offset + 104..offset + 136];
                
                // Signature must be from the past
                if timestamp >= now {
                    continue;
                }
                
                // Verify public key exists in historical keys
                let key_valid = historical_keys.iter().any(|(key, registered_at)| {
                    key == public_key_bytes && *registered_at <= timestamp
                });
                
                if !key_valid {
                    continue;
                }
                
                // Parse cryptographic types
                let public_key = match sr25519::Public::try_from(public_key_bytes) {
                    Ok(pk) => pk,
                    Err(_) => continue,
                };
                
                let signature = match sr25519::Signature::try_from(signature_bytes) {
                    Ok(sig) => sig,
                    Err(_) => continue,
                };
                
                let message_hash: [u8; 32] = match message_hash_bytes.try_into() {
                    Ok(hash) => hash,
                    Err(_) => continue,
                };
                
                // Verify signature
                if sr25519_verify(&signature, &message_hash, &public_key) {
                    verified_count += 1;
                    
                    let age = now.saturating_sub(timestamp);
                    if age > oldest_verified_timestamp {
                        oldest_verified_timestamp = age;
                    }
                }
            }
            
            // Calculate confidence score
            // Component 1: Number of verified signatures (max 50 points)
            let signature_score = (verified_count * 10).min(50);
            
            // Component 2: Age of oldest signature (max 50 points)
            let one_year = 365 * 24 * 60 * 60u64;
            let age_score = ((oldest_verified_timestamp as u128 * 50) / one_year as u128)
                .min(50) as u32;
            
            let total_score = (signature_score + age_score).min(100);
            Ok(total_score as u8)
        }
        
        /// Verify fraud proof with cryptographic signatures
        fn verify_fraud_proof(
            did: &H256,
            guardian: &T::AccountId,
            proof: &[u8],
        ) -> bool {
            // Proof format: [sig: 64 bytes][timestamp: 8 bytes][evidence_hash: 32 bytes][public_key: 32 bytes]
            if proof.len() < 136 {
                return false;
            }
            
            let signature_bytes = &proof[0..64];
            let timestamp_bytes = &proof[64..72];
            let evidence_hash = &proof[72..104];
            let public_key_bytes = &proof[104..136];
            
            // Parse timestamp
            let timestamp = u64::from_le_bytes(
                match timestamp_bytes.try_into() {
                    Ok(bytes) => bytes,
                    Err(_) => return false,
                }
            );
            
            // Verify proof is recent (within 7 days)
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            if now.saturating_sub(timestamp) > MAX_FRAUD_PROOF_AGE {
                return false;
            }
            
            // Verify guardian exists
            let relationship = match GuardianRelationships::<T>::get(did, guardian) {
                Some(rel) => rel,
                None => return false,
            };
            
            // Build message for signature verification
            let mut message = Vec::new();
            message.extend_from_slice(b"FRAUD:");
            message.extend_from_slice(did.as_bytes());
            message.extend_from_slice(&guardian.encode());
            message.extend_from_slice(evidence_hash);
            message.extend_from_slice(&timestamp.to_le_bytes());
            
            let message_hash = sp_io::hashing::blake2_256(&message);
            
            // Convert to sr25519 types
            let public_key = match sr25519::Public::try_from(public_key_bytes) {
                Ok(pk) => pk,
                Err(_) => return false,
            };
            
            let signature = match sr25519::Signature::try_from(signature_bytes) {
                Ok(sig) => sig,
                Err(_) => return false,
            };
            
            // Verify signature using sr25519
            if !sr25519_verify(&signature, &message_hash, &public_key) {
                return false;
            }
            
            // Verify evidence hash is substantive
            if evidence_hash == &[0u8; 32] {
                return false;
            }
            
            // Check guardian's approval history for suspicious patterns
            let guardian_approval_count = GuardianApprovals::<T>::iter()
                .filter(|(_, approvals)| approvals.contains(guardian))
                .count();
            
            // Suspicious if too many recent approvals
            if guardian_approval_count > MAX_GUARDIAN_APPROVALS {
                return true;
            }
            
            // All checks passed
            relationship.relationship_strength > 0
        }

        /// Record behavioral pattern with features
        pub fn record_behavioral_pattern_internal(
            did: &H256,
            features: &BehavioralFeatures,
        ) -> DispatchResult {
            let features_encoded = features.encode();
            let features_hash: H256 = sp_io::hashing::blake2_256(&features_encoded).into();
            let pattern_hash: H256 = sp_io::hashing::blake2_256(&features.encode()).into();
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            BehavioralPatterns::<T>::try_mutate(did, |patterns| -> DispatchResult {
                // Check if pattern already exists
                if let Some(existing) = patterns.iter_mut().find(|p| p.features_hash == features_hash) {
                    existing.sample_count = existing.sample_count.saturating_add(1);
                    existing.recorded_at = now;
                    
                    Self::deposit_event(Event::BehavioralPatternRecorded {
                        did: *did,
                        pattern_hash,
                        sample_count: existing.sample_count,
                    });
                } else {
                    let new_pattern = StoredBehavioralPattern {
                        pattern_hash,
                        features_hash,
                        recorded_at: now,
                        sample_count: 1,
                    };
                    
                    patterns.try_push(new_pattern)
                        .map_err(|_| Error::<T>::InvalidBehavioralProof)?;
                    
                    Self::deposit_event(Event::BehavioralPatternRecorded {
                        did: *did,
                        pattern_hash,
                        sample_count: 1,
                    });
                }
                
                Ok(())
            })
        }

        /// Validate nullifier format
        fn validate_nullifier(nullifier: &H256) -> bool {
            *nullifier != H256::zero()
        }

        /// Validate commitment format
        fn validate_commitment(commitment: &H256) -> bool {
            *commitment != H256::zero()
        }
    }

    /// Check if personhood is registered
    pub fn is_personhood_registered<T: Config>(did: &H256) -> bool {
        if let Some(nullifier) = DidToNullifier::<T>::get(did) {
            PersonhoodRegistry::<T>::contains_key(&nullifier)
        } else {
            false
        }
    }

    /// Check if account is dormant (no activity for 12 months)
    pub fn is_account_dormant<T: Config>(did: &H256) -> bool {
        let last_active = LastActivity::<T>::get(did);
        let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
        let twelve_months = 12 * 30 * 24 * 60 * 60u64;
        
        now.saturating_sub(last_active) > twelve_months
    }

    /// Get nullifier for DID
    pub fn get_nullifier_for_did<T: Config>(did: &H256) -> Result<H256, Error<T>> {
        DidToNullifier::<T>::get(did).ok_or(Error::<T>::DidNotFound)
    }
}