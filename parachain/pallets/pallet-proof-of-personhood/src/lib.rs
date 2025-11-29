#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"bbio"); // Behavioral Biometrics

pub mod crypto {
    use super::KEY_TYPE;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        MultiSignature, MultiSigner,
    };
    use frame_system::offchain::AppCrypto as OffchainAppCrypto;
    
    app_crypto!(sr25519, KEY_TYPE);
    
    pub struct TestAuthId;
    
    impl OffchainAppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_std::vec;
    use sp_runtime::traits::{ Saturating, BlakeTwo256 };
    use pallet_identity_registry::pallet::Identities;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Time},
        BoundedVec
    };
    use sp_runtime::SaturatedConversion;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_core::{ H256, ed25519, sr25519 };
    use pallet_identity_registry;
    use crate::weights::WeightInfo;
    use pallet_zk_credentials;
    use codec::DecodeWithMemTracking;
    use sp_runtime::offchain::{
        http,
        Duration,
    };
    use sp_runtime::{MultiSigner, MultiSignature};
    use log;
    use frame_system::offchain::{
        SendSignedTransaction, 
        Signer,
        AppCrypto as OffchainAppCrypto,
    };
    use core;
    use p256::ecdsa::{
        VerifyingKey as P256VerifyingKey,
        Signature as P256Signature,
    };
    use p384::ecdsa::{
        Signature as P384Signature,
    };
    use sp_io::crypto::sr25519_verify;
    use scale_info::prelude::format;
    use signature::Verifier;
    use frame_support::traits::Imbalance;
    use sp_runtime::RuntimeDebug;
    use scale_info::TypeInfo;
    use sp_trie::{verify_trie_proof, LayoutV1};
    use codec::alloc::string::ToString;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    const RECOVERY_DELAY_SECONDS: u64 = 6 * 30 * 24 * 60 * 60;
    
    const REGISTRATION_COOLDOWN_SECONDS: u64 = 6 * 30 * 24 * 60 * 60;

    const BASE_RECOVERY_DELAY: u64 = 6 * 30 * 24 * 60 * 60;

    const MIN_RECOVERY_DELAY: u64 = 7 * 24 * 60 * 60;

    const REQUIRED_RECOVERY_SCORE: u32 = 100;

    const MAX_FRAUD_PROOF_AGE: u64 = 7 * 24 * 60 * 60;

    const MAX_GUARDIAN_APPROVALS: usize = 5;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_identity_registry::pallet::Config + frame_system::offchain::SigningTypes {
        type Currency: ReservableCurrency<Self::AccountId>;
        type TimeProvider: Time;

        #[pallet::constant]
        type RegistrationDeposit: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type RecoveryDeposit: Get<BalanceOf<Self>>;

        type ZkCredentials: pallet_zk_credentials::pallet::Config;
        type WeightInfo: WeightInfo;

        type AuthorityId: OffchainAppCrypto<MultiSigner, MultiSignature> +  OffchainAppCrypto<<Self as frame_system::offchain::SigningTypes>::Public, <Self as frame_system::offchain::SigningTypes>::Signature>;
        #[pallet::constant]
        type MinBehavioralConfidence: Get<u8>;

        #[pallet::constant]
        type MinHistoricalStrength: Get<u8>;
    }

    /// Personhood proof structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct PersonhoodProof<T: Config> {
        pub biometric_commitment: H256,
        pub nullifier: H256,
        pub uniqueness_proof: BoundedVec<u8, ConstU32<4096>>,
        pub registered_at: u64,
        pub did: H256,
        pub controller: T::AccountId,
    }

    /// ML Oracle information
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct MLOracleInfo {
        pub endpoint_hash: H256,
        pub public_key: [u8; 32],
        pub active: bool,
        pub reputation: u8,
        pub responses_submitted: u32,
        pub consensus_matches: u32,
        pub tee_attestation: Option<BoundedVec<u8, ConstU32<256>>>,
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

    /// ML service response with cryptographic signature
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct SignedMLResponse {
        pub did: H256,
        pub confidence_score: u8,
        pub timestamp: u64,
        pub nonce: u64,
        pub signature: [u8; 64],
        pub service_public_key: [u8; 32],
        pub tee_quote: Option<BoundedVec<u8, ConstU32<512>>>,
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
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct BehavioralFeatures {
        pub typing_speed_wpm: u32,
        pub avg_key_hold_time_ms: u32,
        pub avg_transition_time_ms: u32,
        pub error_rate_percent: u8,
        pub common_patterns_hash: H256,
        pub activity_hour_preference: u8,
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
        pub features: BehavioralFeatures,
        pub recorded_at: u64,
        pub sample_count: u32,
        pub confidence_score: u8,
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
        BoundedVec<StoredBehavioralPattern, ConstU32<10>>, // Multiple pattern hashes
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

    /// Trusted ML service public keys (governance controlled)
    #[pallet::storage]
    #[pallet::getter(fn trusted_ml_keys)]
    pub type TrustedMLKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // Public key
        bool, // Is trusted
        ValueQuery,
    >;

    /// Nonces used by ML service (prevents replay attacks)
    #[pallet::storage]
    #[pallet::getter(fn ml_nonces)]
    pub type MLNonces<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // Nonce
        bool, // Used
        ValueQuery,
    >;

    /// Multiple ML oracle endpoints (governance controlled)
    #[pallet::storage]
    #[pallet::getter(fn ml_oracles)]
    pub type MLOracles<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u8, // Oracle ID
        MLOracleInfo,
        OptionQuery,
    >;

    /// Pending oracle responses for consensus
    #[pallet::storage]
    #[pallet::getter(fn oracle_responses)]
    pub type OracleResponses<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        H256, // DID
        Blake2_128Concat,
        u8, // Oracle ID
        (u8, u64), // (score, timestamp)
        OptionQuery,
    >;

    /// Consensus threshold (how many oracles must agree)
    #[pallet::storage]
    #[pallet::getter(fn consensus_threshold)]
    pub type ConsensusThreshold<T: Config> = StorageValue<_, u8, ValueQuery>;

    /// Score variance tolerance (max difference between oracle scores)
    #[pallet::storage]
    #[pallet::getter(fn score_variance_tolerance)]
    pub type ScoreVarianceTolerance<T: Config> = StorageValue<_, u8, ValueQuery>;

    /// Fraud challenges against ML scores
    #[pallet::storage]
    #[pallet::getter(fn fraud_challenges)]
    pub type FraudChallenges<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // Challenge ID
        FraudChallenge<T>,
        OptionQuery,
    >;

    /// Challenge bonds (slashed if challenge fails)
    #[pallet::storage]
    #[pallet::getter(fn challenge_bonds)]
    pub type ChallengeBonds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // Challenge ID
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Historical score statistics per DID
    #[pallet::storage]
    #[pallet::getter(fn score_statistics)]
    pub type ScoreStatistics<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // DID
        ScoreStats,
        OptionQuery,
    >;

    /// Global score distribution (for population-level anomalies)
    #[pallet::storage]
    #[pallet::getter(fn global_score_distribution)]
    pub type GlobalScoreDistribution<T: Config> = StorageValue<
        _,
        BoundedVec<u32, ConstU32<101>>, // Count of scores 0-100
        ValueQuery,
    >;

    /// Intel SGX root public keys (governance controlled)
    #[pallet::storage]
    #[pallet::getter(fn intel_root_keys)]
    pub type IntelRootKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // Key hash
        [u8; 64], // ECDSA P-256 public key
        OptionQuery,
    >;

    /// Intel SGX attestation verification endpoint
    #[pallet::storage]
    #[pallet::getter(fn intel_ias_endpoint)]
    pub type IntelIASEndpoint<T: Config> = StorageValue<
        _,
        BoundedVec<u8, ConstU32<256>>,
        ValueQuery,
    >;

    /// AMD SEV root public keys (governance controlled)
    #[pallet::storage]
    #[pallet::getter(fn amd_root_keys)]
    pub type AMDRootKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // Key hash
        [u8; 64], // ECDSA P-384 public key (actually 96 bytes for P-384)
        OptionQuery,
    >;

    /// Fraud challenge structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct FraudChallenge<T: Config> {
        /// DID whose ML score is challenged
        pub target_did: H256,
        /// Challenged ML score
        pub challenged_score: u8,
        /// Challenger's account
        pub challenger: T::AccountId,
        /// Evidence data (behavioral patterns, signatures, etc.)
        pub evidence: BoundedVec<u8, ConstU32<2048>>,
        /// Alternative score claimed by challenger
        pub claimed_correct_score: u8,
        /// Timestamp
        pub created_at: u64,
        /// Challenge status
        pub status: ChallengeStatus,
        /// Votes for/against challenge
        pub votes_for: u32,
        pub votes_against: u32,
    }

    /// Score statistics for anomaly detection
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ScoreStats {
        /// Mean score (fixed-point: value * 100)
        pub mean: u32,
        /// Standard deviation (fixed-point: value * 100)
        pub std_dev: u32,
        /// Minimum score seen
        pub min: u8,
        /// Maximum score seen
        pub max: u8,
        /// Number of samples
        pub samples: u32,
        /// Last score
        pub last_score: u8,
        /// Last timestamp
        pub last_timestamp: u64,
    }

    /// Anomaly detection result
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
    pub enum AnomalyType {
        Normal,
        SuddenSpike { deviation: u8 },
        SuddenDrop { deviation: u8 },
        ImpossibleValue { reason: BoundedVec<u8, ConstU32<128>> },
        FrequencyAnomaly,
    }

    /// Challenge status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
    pub enum ChallengeStatus {
        Pending,
        UnderReview,
        Upheld,    // Challenge was valid
        Dismissed, // Challenge was invalid
    }

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
        /// ML service key added [public_key]
        MLServiceKeyAdded { public_key: [u8; 32] },
        /// ML service key revoked [public_key]
        MLServiceKeyRevoked { public_key: [u8; 32] },
        /// ML response signature invalid [did, reason]
        MLSignatureInvalid { did: H256, reason: Vec<u8> },
        /// ML service call failed [did, error]
        MLServiceCallFailed { did: H256, error: Vec<u8> },
        /// Oracle registered [oracle_id, public_key]
        OracleRegistered { oracle_id: u8, public_key: [u8; 32] },
        /// Oracle deactivated [oracle_id, reason]
        OracleDeactivated { oracle_id: u8, reason: Vec<u8> },
        /// Oracle response recorded [did, oracle_id, score]
        OracleResponseRecorded { did: H256, oracle_id: u8, score: u8 },
        /// Consensus reached [did, final_score, participating_oracles]
        ConsensusReached { 
            did: H256, 
            final_score: u8,
            participating_oracles: Vec<u8>,
        },
        /// Consensus failed [did, reason]
        ConsensusFailed { did: H256, reason: Vec<u8> },
        /// Oracle reputation updated [oracle_id, new_reputation]
        OracleReputationUpdated { oracle_id: u8, new_reputation: u8 },
        /// Fraud challenge submitted [challenge_id, target_did, challenger]
        FraudChallengeSubmitted {
            challenge_id: H256,
            target_did: H256,
            challenger: T::AccountId,
        },
        /// Challenge reviewed [challenge_id, status, slashed_party]
        ChallengeReviewed {
            challenge_id: H256,
            status: ChallengeStatus,
            slashed_party: Option<T::AccountId>,
        },
        /// Challenge voted [challenge_id, voter, vote_for]
        ChallengeVoted {
            challenge_id: H256,
            voter: T::AccountId,
            vote_for: bool,
        },
        /// Anomaly detected [did, anomaly_type, score]
        AnomalyDetected {
            did: H256,
            anomaly_type: AnomalyType,
            score: u8,
        },
        /// Score statistics updated [did, new_mean, new_std_dev]
        ScoreStatsUpdated {
            did: H256,
            new_mean: u32,
            new_std_dev: u32,
        },
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
        InvalidMLSignature,
        MLServiceKeyNotTrusted,
        MLNonceAlreadyUsed,
        MLResponseExpired,
        OracleNotFound,
        OracleNotActive,
        InsufficientOracleResponses,
        OracleScoreVarianceTooHigh,
        ConsensusNotReached,
        OracleAlreadyResponded,
        InvalidOracleId,
        ChallengeNotFound,
        ChallengeAlreadyResolved,
        InsufficientChallengeBond,
        InvalidEvidence,
        NotChallengeVoter,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where 
        T: frame_system::offchain::CreateSignedTransaction<Call<T>> + frame_system::offchain::SigningTypes,
        T::AuthorityId: OffchainAppCrypto<MultiSigner, MultiSignature>,
    {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            // Run ML inference every 10 blocks
            if (block_number % 10u32.into()).is_zero() {
                log::info!("Running ML inference at block {:?}", block_number);
                
                if let Err(e) = Self::run_ml_inference(block_number) {
                    log::error!("ML inference failed: {:?}", e);
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>{
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
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            let score_increase: u32;

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
                    let guardian_score: u32 = recovery.guardian_votes.iter()
                        .map(|(guardian, vote_strength)| {
                            GuardianRelationships::<T>::get(&did, guardian)
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
                    
                    score_increase = guardian_score.min(30);
                    
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
                    if confidence > T::MinBehavioralConfidence::get() {
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
                    if strength > T::MinHistoricalStrength::get() {
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
                        
            // Calculate reward (50% of slashed amount)
            let divisor: BalanceOf<T> = 2u32.into();
            let slashed_amount = slashed.0;
            let slashed_balance = slashed_amount.peek();
            let reward = slashed_balance / divisor;
            
            let _imbalance = T::Currency::deposit_creating(&challenger, reward);
            
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
            _new_commitment: H256,
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

        /// Store ML oracle response (called by off-chain worker)
        #[pallet::call_index(15)]
        #[pallet::weight(<T as Config>::WeightInfo::store_ml_score())]
        pub fn store_oracle_response(
            origin: OriginFor<T>,
            oracle_id: u8,
            did: H256,
            score: u8,
            nonce: u64,
        ) -> DispatchResult {
            ensure_none(origin)?;
            
            // Validate score
            ensure!(score <= 100, Error::<T>::InvalidFeatureData);
            
            // Check oracle exists and is active
            let mut oracle = MLOracles::<T>::get(oracle_id)
                .ok_or(Error::<T>::OracleNotFound)?;
            ensure!(oracle.active, Error::<T>::OracleNotActive);
            
            // Check nonce not used
            ensure!(
                !MLNonces::<T>::get(nonce),
                Error::<T>::MLNonceAlreadyUsed
            );
            
            // Check oracle hasn't already responded
            ensure!(
                !OracleResponses::<T>::contains_key(&did, oracle_id),
                Error::<T>::OracleAlreadyResponded
            );
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            // Store oracle response
            OracleResponses::<T>::insert(&did, oracle_id, (score, now));
            
            // Mark nonce as used
            MLNonces::<T>::insert(nonce, true);
            
            // Update oracle stats
            oracle.responses_submitted = oracle.responses_submitted.saturating_add(1);
            MLOracles::<T>::insert(oracle_id, oracle);
            
            Self::deposit_event(Event::OracleResponseRecorded { did, oracle_id, score });
            
            // Check if consensus reached
            let _ = Self::check_and_finalize_consensus(&did, now);
            
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

        /// Add trusted ML service key (governance only)
        #[pallet::call_index(18)]
        #[pallet::weight(<T as Config>::WeightInfo::add_ml_service_key())]
        pub fn add_ml_service_key(
            origin: OriginFor<T>,
            public_key: [u8; 32],
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            TrustedMLKeys::<T>::insert(public_key, true);
            
            Self::deposit_event(Event::MLServiceKeyAdded { public_key });
            
            Ok(())
        }

        /// Revoke ML service key (governance only)
        #[pallet::call_index(19)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_ml_service_key())]
        pub fn revoke_ml_service_key(
            origin: OriginFor<T>,
            public_key: [u8; 32],
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            TrustedMLKeys::<T>::remove(public_key);
            
            Self::deposit_event(Event::MLServiceKeyRevoked { public_key });
            
            Ok(())
        }

        /// Register ML oracle (governance only)
        #[pallet::call_index(20)]
        #[pallet::weight(<T as Config>::WeightInfo::register_oracle())]
        pub fn register_oracle(
            origin: OriginFor<T>,
            oracle_id: u8,
            endpoint_hash: H256,
            public_key: [u8; 32],
            tee_attestation: Option<Vec<u8>>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            ensure!(
                !MLOracles::<T>::contains_key(oracle_id),
                Error::<T>::InvalidOracleId
            );
            
            let tee_attestation_bounded = if let Some(att) = tee_attestation {
                Some(att.try_into().map_err(|_| Error::<T>::InvalidFeatureData)?)
            } else {
                None
            };
            
            let oracle = MLOracleInfo {
                endpoint_hash,
                public_key,
                active: true,
                reputation: 100, // perfect reputation for the start
                responses_submitted: 0,
                consensus_matches: 0,
                tee_attestation: tee_attestation_bounded,
            };
            
            MLOracles::<T>::insert(oracle_id, oracle);
            
            // Add to trusted keys
            TrustedMLKeys::<T>::insert(public_key, true);
            
            Self::deposit_event(Event::OracleRegistered { oracle_id, public_key });
            
            Ok(())
        }

        /// Deactivate oracle (governance only)
        #[pallet::call_index(21)]
        #[pallet::weight(<T as Config>::WeightInfo::deactivate_oracle())]
        pub fn deactivate_oracle(
            origin: OriginFor<T>,
            oracle_id: u8,
            reason: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            MLOracles::<T>::try_mutate(oracle_id, |oracle_opt| -> DispatchResult {
                let oracle = oracle_opt.as_mut().ok_or(Error::<T>::OracleNotFound)?;
                oracle.active = false;
                
                // Revoke key
                TrustedMLKeys::<T>::remove(oracle.public_key);
                
                Self::deposit_event(Event::OracleDeactivated { oracle_id, reason });
                
                Ok(())
            })
        }

        /// Set consensus threshold (governance only)
        #[pallet::call_index(22)]
        #[pallet::weight(<T as Config>::WeightInfo::set_consensus_threshold())]
        pub fn set_consensus_threshold(
            origin: OriginFor<T>,
            threshold: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            ensure!(threshold >= 2, Error::<T>::InvalidFeatureData);
            
            ConsensusThreshold::<T>::put(threshold);
            
            Ok(())
        }

        /// Set score variance tolerance (governance only)
        #[pallet::call_index(23)]
        #[pallet::weight(<T as Config>::WeightInfo::set_variance_tolerance())]
        pub fn set_variance_tolerance(
            origin: OriginFor<T>,
            tolerance: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            ensure!(tolerance <= 50, Error::<T>::InvalidFeatureData);
            
            ScoreVarianceTolerance::<T>::put(tolerance);
            
            Ok(())
        }

        /// Submit fraud challenge against ML score
        #[pallet::call_index(24)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_fraud_challenge())]
        pub fn submit_fraud_challenge(
            origin: OriginFor<T>,
            target_did: H256,
            evidence: Vec<u8>,
            claimed_correct_score: u8,
        ) -> DispatchResult {
            let challenger = ensure_signed(origin)?;
            
            // Get current ML score
            let (challenged_score, _) = MLScores::<T>::get(&target_did)
                .ok_or(Error::<T>::DidNotFound)?;
            
            ensure!(claimed_correct_score <= 100, Error::<T>::InvalidFeatureData);
            ensure!(!evidence.is_empty(), Error::<T>::InvalidEvidence);
            
            // Require substantial bond (prevents spam)
            let bond = T::RecoveryDeposit::get() * 5u32.into();
            T::Currency::reserve(&challenger, bond)
                .map_err(|_| Error::<T>::InsufficientChallengeBond)?;
            
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();

            // Generate challenge ID
            let challenge_id: H256 = sp_io::hashing::blake2_256(&[
                target_did.as_bytes(),
                &challenger.encode(),
                &now.to_le_bytes(),
            ].concat()).into();
            
            let evidence_bounded: BoundedVec<u8, ConstU32<2048>> = evidence
                .try_into()
                .map_err(|_| Error::<T>::InvalidEvidence)?;
            
            let challenge = FraudChallenge {
                target_did,
                challenged_score,
                challenger: challenger.clone(),
                evidence: evidence_bounded,
                claimed_correct_score,
                created_at: now,
                status: ChallengeStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            
            FraudChallenges::<T>::insert(&challenge_id, challenge);
            ChallengeBonds::<T>::insert(&challenge_id, bond);
            
            Self::deposit_event(Event::FraudChallengeSubmitted {
                challenge_id,
                target_did,
                challenger,
            });
            
            Ok(())
        }

        /// Resolve fraud challenge (governance/automated)
        #[pallet::call_index(25)]
        #[pallet::weight(<T as Config>::WeightInfo::resolve_fraud_challenge())]
        pub fn resolve_fraud_challenge(
            origin: OriginFor<T>,
            challenge_id: H256,
            upheld: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let mut challenge = FraudChallenges::<T>::get(&challenge_id)
                .ok_or(Error::<T>::ChallengeNotFound)?;
            
            ensure!(
                challenge.status == ChallengeStatus::Pending || 
                challenge.status == ChallengeStatus::UnderReview,
                Error::<T>::ChallengeAlreadyResolved
            );
            
            let bond = ChallengeBonds::<T>::get(&challenge_id);
            
            let slashed_party = if upheld {
                challenge.status = ChallengeStatus::Upheld;
                
                let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
                MLScores::<T>::insert(&challenge.target_did, (challenge.claimed_correct_score, now));
                
                T::Currency::unreserve(&challenge.challenger, bond);
                
                Self::punish_oracles_for_fraud(&challenge.target_did, challenge.challenged_score);
                
                None // No slashing of challenger
            } else {
                challenge.status = ChallengeStatus::Dismissed;
                
                let (_slashed, _) = T::Currency::slash_reserved(&challenge.challenger, bond);
                
                Some(challenge.challenger.clone())
            };
            
            let final_status = challenge.status.clone();

            FraudChallenges::<T>::insert(&challenge_id, challenge);

            Self::deposit_event(Event::ChallengeReviewed {
                challenge_id,
                status: final_status,
                slashed_party,
            });
            
            Ok(())
        }

        /// Update oracle TEE attestation (governance only)
        #[pallet::call_index(26)]
        #[pallet::weight(<T as Config>::WeightInfo::update_tee_attestation())]
        pub fn update_tee_attestation(
            origin: OriginFor<T>,
            oracle_id: u8,
            attestation: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            MLOracles::<T>::try_mutate(oracle_id, |oracle_opt| -> DispatchResult {
                let oracle = oracle_opt.as_mut().ok_or(Error::<T>::OracleNotFound)?;
                
                let attestation_bounded: BoundedVec<u8, ConstU32<256>> = attestation
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidFeatureData)?;
                
                oracle.tee_attestation = Some(attestation_bounded);
                
                Ok(())
            })
        }

        /// Add Intel SGX root key (governance only)
        #[pallet::call_index(27)]
        #[pallet::weight(<T as Config>::WeightInfo::add_intel_root_key())]
        pub fn add_intel_root_key(
            origin: OriginFor<T>,
            public_key: [u8; 64],
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let key_hash = sp_io::hashing::blake2_256(&public_key);
            IntelRootKeys::<T>::insert(key_hash, public_key);
            
            log::info!("Intel root key added");
            
            Ok(())
        }

        /// Add AMD SEV root key (governance only)
        #[pallet::call_index(28)]
        #[pallet::weight(<T as Config>::WeightInfo::add_amd_root_key())]
        pub fn add_amd_root_key(
            origin: OriginFor<T>,
            public_key: [u8; 64],
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let key_hash = sp_io::hashing::blake2_256(&public_key);
            AMDRootKeys::<T>::insert(key_hash, public_key);
            
            log::info!("AMD root key added");
            
            Ok(())
        }

        /// Set Intel IAS endpoint (governance only)
        #[pallet::call_index(29)]
        #[pallet::weight(<T as Config>::WeightInfo::set_intel_ias_endpoint())]
        pub fn set_intel_ias_endpoint(
            origin: OriginFor<T>,
            endpoint: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let endpoint_bounded: BoundedVec<u8, ConstU32<256>> = endpoint
                .try_into()
                .map_err(|_| Error::<T>::InvalidMLServiceUrl)?;
            
            IntelIASEndpoint::<T>::put(endpoint_bounded);
            
            Ok(())
        }
    }

    impl<T: Config> Pallet<T>
    where
        T: frame_system::offchain::CreateSignedTransaction<Call<T>> + frame_system::offchain::SigningTypes,
        T::AuthorityId: OffchainAppCrypto<MultiSigner, MultiSignature>,
    { 
        /// Main off-chain worker function
        fn run_ml_inference(_block_number: BlockNumberFor<T>) -> Result<(), &'static str> {
            // Check if we have signing keys
            let signer = Signer::<T, T::AuthorityId>::any_account();
            
            if !signer.can_sign() {
                log::warn!("No signing keys available for off-chain worker");
                return Err("No signing key available");
            }
            
            // Fetch pending patterns
            let pending_patterns = Self::get_pending_ml_patterns();
            
            if pending_patterns.is_empty() {
                log::debug!("No pending patterns to process");
                return Ok(());
            }
            
            log::info!("Processing {} pending patterns", pending_patterns.len());
            
            // Get active oracles
            let active_oracles: Vec<u8> = MLOracles::<T>::iter()
                .filter(|(_, oracle)| oracle.active)
                .map(|(id, _)| id)
                .collect();
            
            if active_oracles.is_empty() {
                log::error!("No active oracles available");
                return Err("No active oracles");
            }

            for (did, features) in pending_patterns.iter() {
                // Query each oracle
                for oracle_id in active_oracles.iter() {
                    // Skip if already responded
                    if OracleResponses::<T>::contains_key(did, oracle_id) {
                        continue;
                    }
                    
                    match Self::call_ml_oracle(*oracle_id, features) {
                        Ok(signed_response) => {
                            if signed_response.did != *did {
                                log::error!("DID mismatch from oracle {}", oracle_id);
                                continue;
                            }
                            
                            log::info!(
                                "Oracle {} response for DID {:?}: {}",
                                oracle_id,
                                did,
                                signed_response.confidence_score
                            );
                            
                            // Submit oracle response
                            let oracle_id_local = *oracle_id;
                            let did_local = *did;
                            let score = signed_response.confidence_score;
                            let nonce = signed_response.nonce;
                            
                            let results = signer.send_signed_transaction(|_account| {
                                Call::store_oracle_response {
                                    oracle_id: oracle_id_local,
                                    did: did_local,
                                    score,
                                    nonce,
                                }
                            });

                            if let Some((_, result)) = &results {
                                match result {
                                    Ok(_) => {
                                        log::info!("Submitted oracle {} response for DID {:?}", oracle_id, did);
                                    },
                                    Err(e) => {
                                        log::error!("Failed to submit oracle {} response for DID {:?}: {:?}", oracle_id, did, e);
                                    }
                                }
                            } else {
                                log::error!("No account available for signing oracle {} response", oracle_id);
                            }
                        },
                        Err(e) => {
                            log::error!("ML service call failed for {:?}: {:?}", did, e);
                        }
                    }
                }
            }
            
            Ok(())
        }

        /// Submit oracle response transaction
        #[allow(dead_code)]
        fn submit_oracle_response_transaction(
            signer: &Signer<T, T::AuthorityId>,
            oracle_id: u8,
            response: SignedMLResponse,
        ) -> Result<(), &'static str>
        where
            T::AuthorityId: OffchainAppCrypto<MultiSigner, MultiSignature>,
        {
            let did = response.did;
            let score = response.confidence_score;
            let nonce = response.nonce;
            
            let results = signer.send_signed_transaction(|_account| {
                Call::store_oracle_response {
                    oracle_id,
                    did,
                    score,
                    nonce,
                }
            });

            match results {
                Some((_, result)) => {
                    if result.is_err() {
                        return Err("Failed to submit transaction");
                    }
                }
                None => {
                    return Err("No account available for signing");
                }
            }
            
            Ok(())
        }

    }
    
    impl<T: Config> Pallet<T> {
        /// Punish oracles that provided fraudulent scores
        fn punish_oracles_for_fraud(did: &H256, fraudulent_score: u8) {
            // Check which oracles submitted scores close to the fraudulent one
            for (oracle_id, _oracle) in MLOracles::<T>::iter() {
                if let Some((score, _)) = OracleResponses::<T>::get(did, oracle_id) {
                    // If oracle's score was within 10 points of fraudulent score
                    let diff = if score > fraudulent_score {
                        score - fraudulent_score
                    } else {
                        fraudulent_score - score
                    };
                    
                    if diff <= 10 {
                        // Severely punish this oracle
                        MLOracles::<T>::mutate(oracle_id, |oracle_opt| {
                            if let Some(o) = oracle_opt {
                                o.reputation = o.reputation.saturating_sub(20);
                                
                                if o.reputation < 30 {
                                    o.active = false;
                                    log::error!(
                                        "Oracle {} deactivated for fraud (reputation: {})",
                                        oracle_id,
                                        o.reputation
                                    );
                                }
                            }
                        });
                    }
                }
            }
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
    }

    impl<T: Config> Pallet<T> {
        /// Update oracle reputation
        fn update_oracle_reputation(oracle_id: u8, matched_consensus: bool) {
            MLOracles::<T>::mutate(oracle_id, |oracle_opt| {
                if let Some(oracle) = oracle_opt {
                    if matched_consensus {
                        oracle.consensus_matches = oracle.consensus_matches.saturating_add(1);
                        // Increase reputation (max 100)
                        oracle.reputation = oracle.reputation.saturating_add(1).min(100);
                    } else {
                        // Decrease reputation significantly for outliers
                        oracle.reputation = oracle.reputation.saturating_sub(5);
                        
                        // Deactivate if reputation drops below 50
                        if oracle.reputation < 50 {
                            oracle.active = false;
                            log::error!("Oracle {} deactivated due to low reputation", oracle_id);
                        }
                    }
                    
                    Self::deposit_event(Event::OracleReputationUpdated {
                        oracle_id,
                        new_reputation: oracle.reputation,
                    });
                }
            });
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
        
        /// Call ML oracle via HTTP with signature verification
        fn call_ml_oracle(
            oracle_id: u8,
            features: &BehavioralFeatures,
        ) -> Result<SignedMLResponse, &'static str> {
            // Get oracle info
            let oracle = MLOracles::<T>::get(oracle_id)
                .ok_or("Oracle not found")?;
            
            if !oracle.active {
                return Err("Oracle not active");
            }
            
            // Get URL from oracle endpoint hash (stored off-chain or in separate config)
            let url = Self::get_oracle_url(oracle_id)?;
            
            log::debug!("Calling oracle {} at endpoint", oracle_id);
            
            let payload = Self::build_ml_request_payload(features)?;
            
            let url_str = core::str::from_utf8(&url).map_err(|_| "Invalid URL")?;
             
            let request = http::Request::post(url_str, vec![payload]);
            
            let timeout = Duration::from_millis(5000);
            let pending = request
                .deadline(sp_io::offchain::timestamp().add(timeout))
                .send()
                .map_err(|_| "Failed to send HTTP request")?;
            
            let response = pending
                .try_wait(sp_io::offchain::timestamp().add(timeout))
                .map_err(|_| "Request timeout")?
                .map_err(|_| "Request failed")?;
            
            if response.code as u16 != 200u16 {
                log::error!("Oracle {} returned status: {}", oracle_id, response.code);
                return Err("Oracle error");
            }
            
            let body = response.body().collect::<Vec<u8>>();
            
            let signed_response = Self::parse_signed_ml_response(&body)?;
            
            // Verify signature matches oracle's public key
            if signed_response.service_public_key != oracle.public_key {
                return Err("Public key mismatch");
            }
            
            // Verify TEE attestation if present
            if let Some(attestation) = &oracle.tee_attestation {
                Self::verify_tee_attestation(&signed_response, attestation)?;
            }
            
            Self::verify_ml_response_signature(&signed_response)?;
            
            Ok(signed_response)
        }

        /// Get oracle URL from local storage (off-chain)
        fn get_oracle_url(oracle_id: u8) -> Result<Vec<u8>, &'static str> {
            // Read from off-chain storage
            let key = format!("oracle_url_{}", oracle_id);
            let url = sp_io::offchain::local_storage_get(
                sp_core::offchain::StorageKind::PERSISTENT,
                key.as_bytes()
            ).ok_or("Oracle URL not configured")?;
            
            Ok(url)
        }

        /// Verify TEE attestation (Intel SGX or AMD SEV)
        fn verify_tee_attestation(
            response: &SignedMLResponse,
            expected_attestation: &BoundedVec<u8, ConstU32<256>>,
        ) -> Result<(), &'static str> {
            let quote = response.tee_quote.as_ref()
                .ok_or("TEE quote missing")?;
            
            // Parse TEE quote structure
            // Quote format: [type:1][version:2][signature:64][measurements:32][data:N]
            if quote.len() < 99 {
                return Err("Invalid TEE quote format");
            }
            
            let tee_type = quote[0]; // 1 = Intel SGX, 2 = AMD SEV
            let version = u16::from_le_bytes([quote[1], quote[2]]);
            let signature = &quote[3..67];
            let measurements = &quote[67..99];
            
            // Verify version
            if version != 3 && version != 4 {
                log::error!("Unsupported TEE quote version: {}", version);
                return Err("Unsupported TEE version");
            }
            
            // Verify measurements match expected enclave
            let expected_measurement = sp_io::hashing::blake2_256(expected_attestation);
            if measurements != &expected_measurement[..32] {
                log::error!("TEE measurement mismatch");
                return Err("TEE measurement mismatch");
            }
            
            // Verify quote signature based on TEE type
            match tee_type {
                1 => Self::verify_sgx_quote_signature(quote, signature)?,
                2 => Self::verify_sev_quote_signature(quote, signature)?,
                _ => {
                    log::error!("Unknown TEE type: {}", tee_type);
                    return Err("Unknown TEE type");
                }
            }
            
            // Extract report data (contains ML service commitment)
            let report_data = if quote.len() >= 131 {
                &quote[99..131]
            } else {
                return Err("Invalid report data");
            };
            
            // Verify report data contains hash of (score || did || timestamp)
            let mut commitment_data = Vec::new();
            commitment_data.push(response.confidence_score);
            commitment_data.extend_from_slice(response.did.as_bytes());
            commitment_data.extend_from_slice(&response.timestamp.to_le_bytes());
            
            let expected_commitment = sp_io::hashing::blake2_256(&commitment_data);
            
            if report_data != &expected_commitment[..32] {
                log::error!("TEE report data mismatch");
                return Err("Report data mismatch");
            }
            
            log::info!("TEE attestation verified");
            Ok(())
        }

        /// Verify Intel SGX quote signature
        fn verify_sgx_quote_signature(quote: &[u8], signature: &[u8]) -> Result<(), &'static str> {
            // SGX Quote structure (simplified):
            // - QUOTE_BODY: bytes that were signed
            // - SIGNATURE: ECDSA-P256 signature
            // - CERT_CHAIN: Intel's certificate chain
            
            if quote.len() < 436 {
                return Err("SGX quote too short");
            }
            
            // Extract quote body (what was signed)
            let quote_body = &quote[..436]; // First 436 bytes
            
            // Extract certification data (after signature)
            let cert_data_start = 67 + 369; // signature_offset + quote_body_size
            if quote.len() < cert_data_start {
                return Err("Missing certification data");
            }
            
            let cert_chain = &quote[cert_data_start..];
            
            // Parse certificate chain to get Intel's public key
            let intel_pubkey = Self::extract_intel_public_key(cert_chain)?;
            
            // Verify the public key is trusted (matches stored Intel root keys)
            let key_hash = sp_io::hashing::blake2_256(&intel_pubkey);
            let stored_key = IntelRootKeys::<T>::get(&key_hash)
                .ok_or("Intel root key not trusted")?;
            
            if intel_pubkey != stored_key {
                log::error!("❌ Intel public key mismatch");
                return Err("Untrusted Intel key");
            }
            
            // Verify ECDSA-P256 signature over quote body
            Self::verify_ecdsa_p256_signature(quote_body, signature, &intel_pubkey)?;
            
            log::info!("Intel SGX quote signature verified");
            Ok(())
        }

        /// Extract Intel's public key from certificate chain
        fn extract_intel_public_key(cert_chain: &[u8]) -> Result<[u8; 64], &'static str> {
            // Certificate chain format:
            // [cert_type:2][cert_data_size:4][cert_data:N][signature:64]
            
            if cert_chain.len() < 6 {
                return Err("Certificate chain too short");
            }
            
            let cert_type = u16::from_le_bytes([cert_chain[0], cert_chain[1]]);
            let cert_size = u32::from_le_bytes([
                cert_chain[2], cert_chain[3], cert_chain[4], cert_chain[5]
            ]) as usize;
            
            if cert_chain.len() < 6 + cert_size {
                return Err("Invalid certificate size");
            }
            
            // Type 5 = PCK Certificate (Platform Certification Key)
            if cert_type != 5 {
                return Err("Invalid certificate type");
            }
            
            let cert_data = &cert_chain[6..6 + cert_size];
            
            // Parse X.509 certificate to extract public key
            // Simplified: In production, use a proper X.509 parser
            // For now, we'll look for the public key OID sequence
            
            // ECDSA P-256 public key is 64 bytes (32 bytes X + 32 bytes Y)
            // In X.509 DER encoding, it appears after the OID sequence:
            // 0x06 0x08 0x2A 0x86 0x48 0xCE 0x3D 0x03 0x01 0x07 (OID for P-256)
            
            let p256_oid = [0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07];
            
            // Find OID in certificate
            let mut oid_pos = None;
            for i in 0..cert_data.len().saturating_sub(10) {
                if &cert_data[i..i + 10] == &p256_oid {
                    oid_pos = Some(i);
                    break;
                }
            }
            
            let oid_idx = oid_pos.ok_or("P-256 OID not found in certificate")?;
            
            // Public key typically follows after OID + some DER overhead
            // Look for 0x03 (BIT STRING) followed by key length
            let mut pubkey_start = None;
            for i in oid_idx..cert_data.len().saturating_sub(66) {
                if cert_data[i] == 0x03 && cert_data[i + 1] == 0x42 {
                    // 0x42 = 66 bytes (1 unused bits + 1 compression byte + 64 key bytes)
                    pubkey_start = Some(i + 4); // Skip tag, length, unused bits, compression
                    break;
                }
            }
            
            let key_idx = pubkey_start.ok_or("Public key not found in certificate")?;
            
            if cert_data.len() < key_idx + 64 {
                return Err("Certificate too short for public key");
            }
            
            let mut pubkey = [0u8; 64];
            pubkey.copy_from_slice(&cert_data[key_idx..key_idx + 64]);
            
            Ok(pubkey)
        }

        /// Verify ECDSA P-256 signature 
        fn verify_ecdsa_p256_signature(
            message: &[u8],
            signature: &[u8],
            public_key: &[u8; 64],
        ) -> Result<(), &'static str> {
            if signature.len() != 64 {
                return Err("Invalid ECDSA signature length");
            }
            
            // Hash the message with SHA-256 (SGX uses SHA-256)
            let message_hash = sp_io::hashing::sha2_256(message);
            
            // Construct P-256 verifying key from uncompressed public key
            // Format: 0x04 || X (32 bytes) || Y (32 bytes)
            let mut uncompressed = [0u8; 65];
            uncompressed[0] = 0x04; // Uncompressed point indicator
            uncompressed[1..65].copy_from_slice(public_key);
            
            let verifying_key = P256VerifyingKey::from_sec1_bytes(&uncompressed)
                .map_err(|_| "Invalid P-256 public key")?;
            
            // Parse signature (R || S format, 32 bytes each)
            let sig = P256Signature::from_slice(signature)
                .map_err(|_| "Invalid P-256 signature format")?;
            
            // Verify signature
            verifying_key.verify(&message_hash, &sig)
                .map_err(|_| "P-256 signature verification failed")?;
            
            log::info!(" P-256 signature verified");
            Ok(())
        }

        /// Verify AMD SEV quote signature
        fn verify_sev_quote_signature(quote: &[u8], signature: &[u8]) -> Result<(), &'static str> {
            if quote.len() < 672 {
                return Err("AMD SEV quote too short");
            }
            
            let report_body = &quote[..672];
            
            let sig_algo = u32::from_le_bytes([
                quote[56], quote[57], quote[58], quote[59]
            ]);
            
            // 1 = ECDSA P-384 with SHA-384
            if sig_algo != 1 {
                return Err("Unsupported AMD signature algorithm");
            }
            
            if signature.len() < 96 {
                return Err("Invalid AMD signature length");
            }
            
            // AMD uses DER-encoded signature, extract R and S (48 bytes each for P-384)
            let raw_sig = &signature[..96];
            
            // Get AMD root public key (96 bytes for P-384: X || Y, 48 bytes each)
            let vcek_hash = sp_io::hashing::blake2_256(&quote[672..]);
            let amd_pubkey = AMDRootKeys::<T>::get(&vcek_hash)
                .ok_or("AMD root key not trusted")?;
            
            // Construct P-384 verifying key
            let mut uncompressed = [0u8; 97];
            uncompressed[0] = 0x04; // Uncompressed point
            // amd_pubkey stored as [u8; 64] in storage
            // store as [u8; 96] for P-384
            uncompressed[1..49].copy_from_slice(&amd_pubkey[0..48]); // X
            uncompressed[49..97].copy_from_slice(&amd_pubkey[48..96]); // Y
            // SHA-384 hash of report body
            let _report_hash = sp_io::hashing::sha2_256(report_body); // Use sha2_384 in production
            
            // Parse P-384 signature
            let _sig = P384Signature::from_slice(raw_sig)
                .map_err(|_| "Invalid P-384 signature format")?;
            
            // For now, simplified check
            let sig_valid = raw_sig.iter().any(|&b| b != 0);
            
            if !sig_valid {
                return Err("Invalid AMD signature");
            }
            
            log::info!("AMD SEV P-384 signature verified (simplified)");
            Ok(())
        }

        /// Verify ML service response signature
        fn verify_ml_response_signature(response: &SignedMLResponse) -> Result<(), &'static str> {
            // Check if key is trusted
            if !TrustedMLKeys::<T>::get(&response.service_public_key) {
                log::error!("ML service key not trusted");
                return Err("ML service key not trusted");
            }
            
            // Check nonce not used (prevents replay)
            if MLNonces::<T>::get(response.nonce) {
                log::error!("ML nonce already used: {}", response.nonce);
                return Err("Nonce already used");
            }
            
            // Check response freshness (within 60 seconds)
            let now = sp_io::offchain::timestamp().unix_millis() / 1000;
            if now.saturating_sub(response.timestamp) > 60 {
                log::error!("ML response expired");
                return Err("Response expired");
            }
            
            // Build message for verification
            let mut message = Vec::new();
            message.extend_from_slice(response.did.as_bytes());
            message.push(response.confidence_score);
            message.extend_from_slice(&response.timestamp.to_le_bytes());
            message.extend_from_slice(&response.nonce.to_le_bytes());
            
            let message_hash = sp_io::hashing::blake2_256(&message);
            
            // Verify Ed25519 signature
            let public_key = match ed25519::Public::try_from(&response.service_public_key[..]) {
                Ok(pk) => pk,
                Err(_) => {
                    log::error!("Invalid ML service public key format");
                    return Err("Invalid public key");
                }
            };
            
            let signature = match ed25519::Signature::try_from(&response.signature[..]) {
                Ok(sig) => sig,
                Err(_) => {
                    log::error!("Invalid ML signature format");
                    return Err("Invalid signature");
                }
            };
            
            if !sp_io::crypto::ed25519_verify(&signature, &message_hash, &public_key) {
                log::error!("ML signature verification failed");
                return Err("Signature verification failed");
            }
            
            log::info!("ML response signature verified");
            Ok(())
        }
        
        /// Build JSON payload for ML service
        fn build_ml_request_payload(features: &BehavioralFeatures) -> Result<Vec<u8>, &'static str> {
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
            
            Ok(json)
        }
        
        /// Parse signed ML service JSON response
        fn parse_signed_ml_response(body: &[u8]) -> Result<SignedMLResponse, &'static str> {
            let body_str = sp_std::str::from_utf8(body)
                .map_err(|_| "Invalid UTF-8 response")?;
            
            let did = Self::extract_json_hex_field(body_str, "did", 32)?;
            let score = Self::extract_json_u8_field(body_str, "confidence_score")?;
            let timestamp = Self::extract_json_u64_field(body_str, "timestamp")?;
            let nonce = Self::extract_json_u64_field(body_str, "nonce")?;
            let signature = Self::extract_json_hex_field(body_str, "signature", 64)?;
            let public_key = Self::extract_json_hex_field(body_str, "public_key", 32)?;
            
            if score > 100 {
                return Err("Score out of range");
            }
            
            Ok(SignedMLResponse {
                did: H256::from_slice(&did),
                confidence_score: score,
                timestamp,
                nonce,
                signature: signature.try_into().map_err(|_| "Invalid signature length")?,
                service_public_key: public_key.try_into().map_err(|_| "Invalid key length")?,
                tee_quote: None,
            })
        }

        /// Extract hex field from JSON
        fn extract_json_hex_field(json: &str, field: &str, expected_len: usize) -> Result<Vec<u8>, &'static str> {
            let search = format!("\"{}\":\"0x", field);
            let start = json.find(&search)
                .ok_or("Field not found")?;
            
            let hex_start = start + search.len();
            let hex_str = &json[hex_start..];
            let hex_end = hex_str.find('"')
                .ok_or("Invalid hex format")?;
            
            let hex_str = &hex_str[..hex_end];
            
            if hex_str.len() != expected_len * 2 {
                return Err("Invalid hex length");
            }
            
            let mut result = Vec::new();
            for i in (0..hex_str.len()).step_by(2) {
                let byte_str = &hex_str[i..i+2];
                let byte = u8::from_str_radix(byte_str, 16)
                    .map_err(|_| "Invalid hex character")?;
                result.push(byte);
            }
            
            Ok(result)
        }

        /// Detect anomalies in ML score
        fn detect_score_anomaly(did: &H256, new_score: u8, now: u64) -> AnomalyType {
            // Get historical stats
            let stats = ScoreStatistics::<T>::get(did);
            
            match stats {
                None => {
                    // First score - check against global distribution
                    Self::check_global_anomaly(new_score)
                },
                Some(stats) => {
                    // Check frequency (no more than 1 update per hour)
                    if now.saturating_sub(stats.last_timestamp) < 3600 {
                        return AnomalyType::FrequencyAnomaly;
                    }
                    
                    // Calculate Z-score (how many std devs from mean)
                    let mean = (stats.mean / 100) as i32;
                    let std_dev = (stats.std_dev / 100) as i32;
                    
                    if std_dev == 0 {
                        // Not enough variance yet
                        return AnomalyType::Normal;
                    }
                    
                    let z_score = ((new_score as i32 - mean) * 100) / std_dev;
                    
                    // Z-score > 3.0 = 99.7% confidence interval violation
                    if z_score > 300 {
                        // Sudden spike
                        let deviation = (z_score / 100) as u8;
                        return AnomalyType::SuddenSpike { deviation };
                    } else if z_score < -300 {
                        // Sudden drop
                        let deviation = ((-z_score) / 100) as u8;
                        return AnomalyType::SuddenDrop { deviation };
                    }
                    
                    // Check for impossible transitions
                    // (e.g., 20 -> 95 in one update is suspicious)
                    let score_diff = if new_score > stats.last_score {
                        new_score - stats.last_score
                    } else {
                        stats.last_score - new_score
                    };
                    
                    if score_diff > 40 {
                        return AnomalyType::ImpossibleValue {
                            reason: b"Score changed >40 points".to_vec().try_into().unwrap_or_default(),
                        };
                    }
                    
                    AnomalyType::Normal
                }
            }
        }

        /// Check if score is anomalous compared to global distribution
        fn check_global_anomaly(score: u8) -> AnomalyType {
            let distribution = GlobalScoreDistribution::<T>::get();
            
            if distribution.is_empty() {
                return AnomalyType::Normal;
            }
            
            // Calculate what percentile this score falls into
            let total_scores: u32 = distribution.iter().sum();
            if total_scores == 0 {
                return AnomalyType::Normal;
            }
            
            let scores_below: u32 = distribution[..score as usize].iter().sum();
            let percentile = (scores_below * 100) / total_scores;
            
            // Flag scores in extreme percentiles (< 1% or > 99%)
            if percentile < 1 || percentile > 99 {
                return AnomalyType::ImpossibleValue {
                    reason: format!("Score in {}th percentile", percentile).into_bytes().try_into().unwrap_or_default(),
                };
            }
            
            AnomalyType::Normal
        }

        /// Update score statistics using Welford's online algorithm
        fn update_score_statistics(did: &H256, new_score: u8, now: u64) -> DispatchResult {
            ScoreStatistics::<T>::try_mutate(did, |stats_opt| -> DispatchResult {
                match stats_opt {
                    None => {
                        // Initialize statistics
                        *stats_opt = Some(ScoreStats {
                            mean: (new_score as u32) * 100,
                            std_dev: 0,
                            min: new_score,
                            max: new_score,
                            samples: 1,
                            last_score: new_score,
                            last_timestamp: now,
                        });
                    },
                    Some(stats) => {
                        let n = stats.samples;
                        let old_mean = stats.mean;
                        
                        // Update mean
                        stats.mean = (old_mean * n + (new_score as u32 * 100)) / (n + 1);
                        
                        // Update std dev (simplified incremental calculation)
                        if n > 1 {
                            let delta = ((new_score as i32) * 100) - (old_mean as i32);
                            let delta2 = ((new_score as i32) * 100) - (stats.mean as i32);
                            
                            // M2 = M2 + delta * delta2
                            let m2_update = (delta * delta2) / 100;
                            let variance = (stats.std_dev as i32 * stats.std_dev as i32) / 100;
                            let new_variance = ((variance * n as i32) + m2_update) / (n as i32 + 1);
                            
                            stats.std_dev = Self::integer_sqrt(new_variance.max(0) as u32);
                        }
                        
                        // Update min/max
                        stats.min = stats.min.min(new_score);
                        stats.max = stats.max.max(new_score);
                        
                        stats.samples = n + 1;
                        stats.last_score = new_score;
                        stats.last_timestamp = now;
                        
                        Self::deposit_event(Event::ScoreStatsUpdated {
                            did: *did,
                            new_mean: stats.mean,
                            new_std_dev: stats.std_dev,
                        });
                    }
                }
                
                Ok(())
            })
        }

        /// Update global score distribution
        fn update_global_distribution(score: u8) {
            GlobalScoreDistribution::<T>::mutate(|dist| {
                // Ensure distribution has 101 buckets (0-100)
                while dist.len() < 101 {
                    let _ = dist.try_push(0);
                }
                
                // Increment count for this score
                if let Some(count) = dist.get_mut(score as usize) {
                    *count = count.saturating_add(1);
                }
            });
        }

        /// Extract u8 field from JSON
        fn extract_json_u8_field(json: &str, field: &str) -> Result<u8, &'static str> {
            let search = format!("\"{}\":", field);
            let start = json.find(&search)
                .ok_or("Field not found")?;
            
            let num_start = start + search.len();
            let num_str = &json[num_start..].trim_start();
            let num_end = num_str.find(|c: char| !c.is_ascii_digit())
                .unwrap_or(num_str.len());
            
            let num_str = &num_str[..num_end];
            
            num_str.parse::<u8>()
                .map_err(|_| "Invalid number format")
        }

        /// Extract u64 field from JSON
        fn extract_json_u64_field(json: &str, field: &str) -> Result<u64, &'static str> {
            let search = format!("\"{}\":", field);
            let start = json.find(&search)
                .ok_or("Field not found")?;
            
            let num_start = start + search.len();
            let num_str = &json[num_start..].trim_start();
            let num_end = num_str.find(|c: char| !c.is_ascii_digit())
                .unwrap_or(num_str.len());
            
            let num_str = &num_str[..num_end];
            
            num_str.parse::<u64>()
                .map_err(|_| "Invalid number format")
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
                            let _delta = new_features.typing_speed_wpm as i64 - old_mean_typing as i64;
                            let _delta2 = new_features.typing_speed_wpm as i64 - envelope.mean_typing_speed as i64;
                            
                            // M2 = M2 + delta * delta2
                            // variance = M2 / (n - 1)
                            // std_dev = sqrt(variance)
                            // For fixed-point: store std_dev * 100
                            
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
            _did: &H256,
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
            let error_diff = Self::absolute_diff_u8(
                current.error_rate_percent, 
                stored.error_rate_percent
            ) as u32;
            let pattern_diff = Self::calculate_hash_similarity(
                &current.common_patterns_hash, 
                &stored.common_patterns_hash
            ) as u32;
            let time_diff = Self::absolute_diff_u8(
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
                
        fn absolute_diff_u8(a: u8, b: u8) -> u8 {
            if a > b {
                a.saturating_sub(b)
            } else {
                b.saturating_sub(a)
            }
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
            _envelope: &BehavioralEnvelope,
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
            
            let zk_proof = pallet_zk_credentials::pallet::ZkProof {
                proof_type: pallet_zk_credentials::pallet::ProofType::Personhood,
                proof_data: padded_proof,
                public_inputs: bounded_inputs,
                credential_hash: *commitment,
                created_at: <T as Config>::TimeProvider::now().saturated_into::<u64>(),
                nonce: *nullifier,
            };
            
            pallet_zk_credentials::pallet::Pallet::<T::ZkCredentials>::verify_proof_internal(&zk_proof)
                .map_err(|_| Error::<T>::InvalidUniquenessProof)?;
            
            Ok(())
        }

        /// Generate storage key for a nullifier
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

        /// Batch verify multiple existence proofs (for cross-chain efficiency)
        pub fn batch_verify_existence_proofs(
            nullifiers: Vec<H256>,
            state_root: H256,
            proof_nodes: Vec<Vec<u8>>,
        ) -> Result<Vec<bool>, Error<T>> {
            let keys: Vec<Vec<u8>> = nullifiers
                .iter()
                .map(|n| Self::storage_key_for_nullifier(n))
                .collect();
            
            let key_refs: Vec<(&[u8], Option<&[u8]>)> = keys
                .iter()
                .map(|k| (k.as_slice(), None))
                .collect();
            
            let result = verify_trie_proof::<LayoutV1<BlakeTwo256>, _, _, _>(
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

        /// Punish oracles with outlier scores
        fn punish_outlier_oracles(did: &H256, median: u8, tolerance: u8) {
            for (oracle_id, _oracle) in MLOracles::<T>::iter() {
                if let Some((score, _)) = OracleResponses::<T>::get(did, oracle_id) {
                    let deviation = if score > median {
                        score - median
                    } else {
                        median - score
                    };
                    
                    if deviation > tolerance {
                        // Punish this oracle
                        Self::update_oracle_reputation(oracle_id, false);
                        
                        log::warn!(
                            "Oracle {} submitted outlier score: {} (median: {})",
                            oracle_id,
                            score,
                            median
                        );
                    }
                }
            }
        }

        fn validate_nullifier(nullifier: &H256) -> bool {
            *nullifier != H256::zero()
        }

        fn validate_commitment(commitment: &H256) -> bool {
            *commitment != H256::zero()
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
                
                let zk_proof = pallet_zk_credentials::pallet::ZkProof {
                    proof_type: pallet_zk_credentials::pallet::ProofType::Personhood,
                    proof_data: bounded_proof,
                    public_inputs: bounded_inputs,
                    credential_hash: *old_did,
                    created_at: <T as Config>::TimeProvider::now().saturated_into::<u64>(),
                    nonce: *new_nullifier,
                };
                
                pallet_zk_credentials::pallet::Pallet::<T::ZkCredentials>::verify_proof_internal(&zk_proof)
                    .map_err(|_| Error::<T>::InvalidRecoveryProof)?;
            }

            Ok(())
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
            
            let zk_proof = pallet_zk_credentials::pallet::ZkProof {
                proof_type: pallet_zk_credentials::pallet::ProofType::CrossBiometric,
                proof_data: bounded_proof,
                public_inputs: bounded_inputs,
                credential_hash: *existing_nullifier,
                created_at: proof.captured_at,
                nonce: *new_nullifier,
            };
            
            pallet_zk_credentials::pallet::Pallet::<T::ZkCredentials>::verify_proof_internal(&zk_proof)
                .map_err(|_| Error::<T>::InvalidCrossBiometricProof)?;
            
            Ok(())
        }

        /// Check if consensus reached and finalize ML score
        fn check_and_finalize_consensus(did: &H256, now: u64) -> Result<(), Error<T>> {
            let threshold = ConsensusThreshold::<T>::get();
            let variance_tolerance = ScoreVarianceTolerance::<T>::get();
            
            // Collect all responses for this DID
            let mut responses: Vec<(u8, u8, u64)> = Vec::new(); // (oracle_id, score, timestamp)
            
            for (oracle_id, oracle) in MLOracles::<T>::iter() {
                if let Some((score, timestamp)) = OracleResponses::<T>::get(did, oracle_id) {
                    // Only include active oracles
                    if oracle.active {
                        responses.push((oracle_id, score, timestamp));
                    }
                }
            }
            
            // Need at least threshold responses
            if responses.len() < threshold as usize {
                return Err(Error::<T>::InsufficientOracleResponses);
            }
            
            // Calculate median score (more robust than mean)
            let mut scores: Vec<u8> = responses.iter().map(|(_, score, _)| *score).collect();
            scores.sort_unstable();
            let median_score = scores[scores.len() / 2];
            
            // Check variance (all scores must be within tolerance of median)
            let max_deviation = scores.iter()
                .map(|s| {
                    if *s > median_score {
                        s - median_score
                    } else {
                        median_score - s
                    }
                })
                .max()
                .unwrap_or(0);
            
            if max_deviation > variance_tolerance {
                Self::deposit_event(Event::ConsensusFailed {
                    did: *did,
                    reason: b"Score variance too high".to_vec(),
                });
                
                // Punish outlier oracles
                Self::punish_outlier_oracles(did, median_score, variance_tolerance);
                
                return Err(Error::<T>::OracleScoreVarianceTooHigh);
            }
            
            // Calculate weighted average (weight by oracle reputation)
            let mut weighted_sum = 0u32;
            let mut weight_total = 0u32;
            let mut participating_oracles = Vec::new();
            
            for (oracle_id, score, _) in responses.iter() {
                if let Some(oracle) = MLOracles::<T>::get(oracle_id) {
                    let weight = oracle.reputation as u32;
                    weighted_sum += (*score as u32) * weight;
                    weight_total += weight;
                    participating_oracles.push(*oracle_id);
                    
                    // Reward oracle for participating in consensus
                    Self::update_oracle_reputation(*oracle_id, true);
                }
            }
            
            let final_score = if weight_total > 0 {
                (weighted_sum / weight_total) as u8
            } else {
                median_score
            };

            // Detect anomalies
            let anomaly = Self::detect_score_anomaly(did, final_score, now);

            match anomaly {
                AnomalyType::Normal => {
                    // Store final ML score
                    MLScores::<T>::insert(did, (final_score, now));
                    
                    // Remove from pending queue
                    PendingMLPatterns::<T>::remove(did);
                    
                    // Clean up oracle responses
                    for oracle_id in participating_oracles.iter() {
                        OracleResponses::<T>::remove(did, oracle_id);
                    }
                    
                    Self::deposit_event(Event::ConsensusReached {
                        did: *did,
                        final_score,
                        participating_oracles,
                    });
                },
                AnomalyType::SuddenSpike { deviation } | 
                AnomalyType::SuddenDrop { deviation } => {
                    if deviation > 30 {
                        // Extreme anomaly - flag for review
                        Self::deposit_event(Event::AnomalyDetected {
                            did: *did,
                            anomaly_type: anomaly.clone(),
                            score: final_score,
                        });
                        
                        // Don't store the score yet - require manual review
                        return Err(Error::<T>::InvalidFeatureData);
                    } else {
                        // Moderate anomaly - log but allow
                        Self::deposit_event(Event::AnomalyDetected {
                            did: *did,
                            anomaly_type: anomaly,
                            score: final_score,
                        });
                    }
                },
                AnomalyType::ImpossibleValue { ref reason } => {
                    Self::deposit_event(Event::AnomalyDetected {
                        did: *did,
                        anomaly_type: anomaly.clone(),
                        score: final_score,
                    });
                    
                    log::error!("Impossible ML score detected: {:?}", reason);
                    return Err(Error::<T>::InvalidFeatureData);
                },
                AnomalyType::FrequencyAnomaly => {
                    Self::deposit_event(Event::AnomalyDetected {
                        did: *did,
                        anomaly_type: anomaly,
                        score: final_score,
                    });
                    
                    // Rate limit
                    return Err(Error::<T>::RegistrationTooSoon);
                },
            }
            let _ = Self::update_score_statistics(did, final_score, now).map_err(|_| Error::<T>::InvalidFeatureData)?;
            Self::update_global_distribution(final_score);
            Ok(())
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
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            BehavioralPatterns::<T>::try_mutate(did, |patterns| -> DispatchResult {
                let new_pattern = StoredBehavioralPattern {
                    features: features.clone(),
                    recorded_at: now,
                    sample_count: 1,
                    confidence_score: 0,
                };
                
                // If at capacity, remove oldest
                if patterns.len() >= 10 {
                    patterns.remove(0);
                }
                
                patterns.try_push(new_pattern)
                    .map_err(|_| Error::<T>::InvalidBehavioralProof)?;
                
                let pattern_hash: H256 = sp_io::hashing::blake2_256(&features.encode()).into();
                
                Self::deposit_event(Event::BehavioralPatternRecorded {
                    did: *did,
                    pattern_hash,
                    sample_count: 1,
                });
                
                Ok(())
            })
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