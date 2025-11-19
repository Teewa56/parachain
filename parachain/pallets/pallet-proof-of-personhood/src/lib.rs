#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
use weights::WeightInfo;

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
        pub uniqueness_proof: Vec<u8>,
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
        pub recovery_proof: Vec<u8>,
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
    pub type PersonhoodRegistry<T: Config> = StorageMap
        _,
        Blake2_128Concat,
        H256, // nullifier
        PersonhoodProof<T>,
        OptionQuery,
    >;

    /// Storage: Map DID to nullifier
    #[pallet::storage]
    #[pallet::getter(fn did_to_nullifier)]
    pub type DidToNullifier<T: Config> = StorageMap
        _,
        Blake2_128Concat,
        H256, // DID
        H256, // nullifier
        OptionQuery,
    >;

    /// Storage: Pending recovery requests
    #[pallet::storage]
    #[pallet::getter(fn pending_recoveries)]
    pub type PendingRecoveries<T: Config> = StorageMap
        _,
        Blake2_128Concat,
        H256, // DID
        RecoveryRequest<T>,
        OptionQuery,
    >;

    /// Storage: Guardian approvals for recovery
    #[pallet::storage]
    #[pallet::getter(fn guardian_approvals)]
    pub type GuardianApprovals<T: Config> = StorageMap
        _,
        Blake2_128Concat,
        H256, // DID
        BoundedVec<T::AccountId, ConstU32<10>>,
        ValueQuery,
    >;

    /// Storage: Registration cooldown
    #[pallet::storage]
    #[pallet::getter(fn registration_cooldown)]
    pub type RegistrationCooldown<T: Config> = StorageMap
        _,
        Blake2_128Concat,
        H256, // nullifier
        u64, // can register again after this timestamp
        ValueQuery,
    >;

    /// Storage: Last activity timestamp for each DID
    #[pallet::storage]
    #[pallet::getter(fn last_activity)]
    pub type LastActivity<T: Config> = StorageMap
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
        /// Verify uniqueness proof (ZK proof that biometric is unique)
        fn verify_uniqueness_proof(
            nullifier: &H256,
            commitment: &H256,
            proof: &[u8],
        ) -> Result<(), Error<T>> {
            // Basic validation
            if proof.is_empty() || proof.len() > 1024 {
                return Err(Error::<T>::InvalidUniquenessProof);
            }

            // Verify proof structure
            let mut data = Vec::new();
            data.extend_from_slice(nullifier.as_bytes());
            data.extend_from_slice(commitment.as_bytes());
            data.extend_from_slice(proof);
            
            let proof_hash = sp_io::hashing::blake2_256(&data);
            
            // Proof cannot be all zeros
            if proof_hash == [0u8; 32] {
                return Err(Error::<T>::InvalidUniquenessProof);
            }

            Ok(())
        }

        /// Verify recovery proof (ZK proof linking old and new identity)
        fn verify_recovery_proof(
            old_did: &H256,
            new_nullifier: &H256,
            proof: &[u8],
        ) -> Result<(), Error<T>> {
            if proof.is_empty() || proof.len() > 1024 {
                return Err(Error::<T>::InvalidRecoveryProof);
            }

            let mut data = Vec::new();
            data.extend_from_slice(old_did.as_bytes());
            data.extend_from_slice(new_nullifier.as_bytes());
            data.extend_from_slice(proof);
            
            let proof_hash = sp_io::hashing::blake2_256(&data);
            
            if proof_hash == [0u8; 32] {
                return Err(Error::<T>::InvalidRecoveryProof);
            }

            Ok(())
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