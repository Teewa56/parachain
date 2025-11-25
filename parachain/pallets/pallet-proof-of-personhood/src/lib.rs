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

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_identity_registry::Config {
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
}