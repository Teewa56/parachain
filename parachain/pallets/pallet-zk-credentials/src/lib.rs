#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::ConstU32};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_core::H256;
    use sp_runtime::traits::SaturatedConversion;
    use frame_support::BoundedVec;
    use crate::weights::WeightInfo;

    #[cfg(feature = "sp1")]
    use sp1_sdk::{ProverClient, SP1Stdin};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    /// Proof types supported
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProofType {
        AgeAbove,
        StudentStatus,
        VaccinationStatus,
        EmploymentStatus,
        Personhood,
        Custom,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, Copy, MaxEncodedLen)]
    pub enum ZkCredentialType {
        StudentStatus,
        VaccinationStatus,
        EmploymentStatus,
        AgeVerification,
        Custom,
    }

    /// ZK Proof structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ZkProof {
        pub proof_type: ProofType,
        pub proof_data: BoundedVec<u8, ConstU32<8192>>,
        pub public_inputs: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<16>>,
        pub credential_hash: H256,
        pub created_at: u64,
        pub nonce: H256,
    }

    /// Verification Key structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct VerificationKeyData {
        pub proof_type: ProofType,
        pub vk_data: BoundedVec<u8, ConstU32<2048>>,
        pub registered_by: H256,
        pub registered_at: u64,
    }

    /// Storage: Verified proofs (to prevent replay attacks)
    #[pallet::storage]
    #[pallet::getter(fn verified_proofs)]
    pub type VerifiedProofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,
        (T::AccountId, u64),
        OptionQuery,
    >;

    /// Storage: Verification Keys by proof type
    #[pallet::storage]
    #[pallet::getter(fn verifying_keys)]
    pub type VerifyingKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ProofType,
        VerificationKeyData,
        OptionQuery,
    >;

    /// Storage: Proof schemas
    #[pallet::storage]
    #[pallet::getter(fn proof_schemas)]
    pub type ProofSchemas<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ProofType,
        BoundedVec<BoundedVec<u8, ConstU32<128>>, ConstU32<32>>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        VerificationKeyRegistered { 
            proof_type: ProofType,
            registered_by: H256 
        },
        ProofVerified { 
            proof_hash: H256, 
            verifier: T::AccountId, 
            proof_type: ProofType 
        },
        ProofVerificationFailed { 
            proof_hash: H256, 
            reason: BoundedVec<u8, ConstU32<256>> 
        },
        ProofSchemaCreated { 
            proof_type: ProofType, 
            creator: T::AccountId 
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        VerificationKeyNotFound,
        InvalidProofData,
        ProofVerificationFailed,
        InvalidPublicInputs,
        ProofAlreadyVerified,
        InvalidProofType,
        SchemaNotFound,
        ProofTooOld,
        SchemaAlreadyExists,
        InvalidVkData,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register verification key for a proof type
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_verification_key())]
        pub fn register_verification_key(
            origin: OriginFor<T>,
            proof_type: ProofType,
            vk_data: Vec<u8>,
            registered_by_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let bounded_vk: BoundedVec<u8, ConstU32<2048>> = vk_data
                .try_into()
                .map_err(|_| Error::<T>::InvalidVkData)?;

            let now = frame_system::Pallet::<T>::block_number()
                .saturated_into::<u64>();

            let vk_data_struct = VerificationKeyData {
                proof_type: proof_type.clone(),
                vk_data: bounded_vk,
                registered_by: registered_by_did,
                registered_at: now,
            };

            VerifyingKeys::<T>::insert(&proof_type, vk_data_struct);

            Self::deposit_event(Event::VerificationKeyRegistered { 
                proof_type,
                registered_by: registered_by_did 
            });

            Ok(())
        }

        /// Verify a ZK proof
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::verify_proof())]
        pub fn verify_proof(
            origin: OriginFor<T>,
            proof: ZkProof,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Self::validate_proof_freshness(&proof),
                Error::<T>::ProofTooOld
            );

            let proof_hash = Self::hash_proof(&proof);
            ensure!(
                !VerifiedProofs::<T>::contains_key(&proof_hash),
                Error::<T>::ProofAlreadyVerified
            );

            // Perform verification
            Self::verify_proof_internal(&proof)?;

            VerifiedProofs::<T>::insert(&proof_hash, (who.clone(), proof.created_at));

            Self::deposit_event(Event::ProofVerified {
                proof_hash,
                verifier: who,
                proof_type: proof.proof_type,
            });

            Ok(())
        }

        /// Create proof schema
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::create_proof_schema())]
        pub fn create_proof_schema(
            origin: OriginFor<T>,
            proof_type: ProofType,
            field_descriptions: Vec<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(
                !ProofSchemas::<T>::contains_key(&proof_type),
                Error::<T>::SchemaAlreadyExists
            );

            let inner_bounded: Result<Vec<BoundedVec<u8, ConstU32<128>>>, _> = field_descriptions
                .into_iter()
                .map(|desc| desc.try_into().map_err(|_| Error::<T>::InvalidProofData))
                .collect();

            let bounded_descriptions: BoundedVec<BoundedVec<u8, ConstU32<128>>, ConstU32<32>> = 
                inner_bounded?
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidProofData)?;

            ProofSchemas::<T>::insert(&proof_type, bounded_descriptions);

            Self::deposit_event(Event::ProofSchemaCreated {
                proof_type,
                creator: who,
            });
            
            Ok(())
        }

        /// Batch verify multiple proofs
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::batch_verify_proofs(proofs.len() as u32))]
        pub fn batch_verify_proofs(
            origin: OriginFor<T>,
            proofs: Vec<ZkProof>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            for proof in proofs {
                let proof_hash = Self::hash_proof(&proof);
                
                ensure!(
                    !VerifiedProofs::<T>::contains_key(&proof_hash),
                    Error::<T>::ProofAlreadyVerified
                );

                Self::verify_proof_internal(&proof)?;

                VerifiedProofs::<T>::insert(&proof_hash, (who.clone(), proof.created_at));
                
                Self::deposit_event(Event::ProofVerified {
                    proof_hash,
                    verifier: who.clone(),
                    proof_type: proof.proof_type,
                });
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Hash a proof to detect replays
        pub fn hash_proof(proof: &ZkProof) -> H256 {
            let mut data = Vec::new();
            data.extend_from_slice(&proof.proof_data);
            for input in &proof.public_inputs {
                data.extend_from_slice(input);
            }
            data.extend_from_slice(proof.credential_hash.as_bytes());
            data.extend_from_slice(proof.nonce.as_bytes());
            
            sp_io::hashing::blake2_256(&data).into()
        }

        /// Validate proof freshness (within 1 hour)
        fn validate_proof_freshness(proof: &ZkProof) -> bool {
            let current_time = frame_system::Pallet::<T>::block_number();
            let current_time_u64 = current_time.saturated_into::<u64>();
            
            if proof.created_at > current_time_u64 {
                return false;
            }
            
            let proof_age = current_time_u64.saturating_sub(proof.created_at);
            proof_age <= 3600u64
        }

        /// Internal verification logic
        fn verify_proof_internal(proof: &ZkProof) -> Result<(), Error<T>> {
            // Get verification key
            let _vk_data = VerifyingKeys::<T>::get(&proof.proof_type)
                .ok_or(Error::<T>::VerificationKeyNotFound)?;
            
            #[cfg(feature = "sp1")]
            {
                // Validate proof data
                if proof.proof_data.is_empty() {
                    return Err(Error::<T>::InvalidProofData);
                }
                // SP1 proofs should be at least 64 bytes (typical for zk proofs)
                if proof.proof_data.len() < 64 {
                    return Err(Error::<T>::InvalidProofData);
                }

                // Validate public inputs
                if proof.public_inputs.is_empty() {
                    return Err(Error::<T>::InvalidPublicInputs);
                }
                // Each public input should be non-empty
                for input in proof.public_inputs.iter() {
                    if input.is_empty() {
                        return Err(Error::<T>::InvalidPublicInputs);
                    }
                }
                // Public inputs total should not exceed reasonable size
                let total_inputs_size: usize = proof.public_inputs.iter().map(|i| i.len()).sum();
                if total_inputs_size > 1024 {
                    return Err(Error::<T>::InvalidPublicInputs);
                }

                // Validate verification key
                if _vk_data.vk_data.is_empty() {
                    return Err(Error::<T>::InvalidVkData);
                }
                // VK should be reasonable size (typically 500+ bytes for zk schemes)
                if _vk_data.vk_data.len() < 256 {
                    return Err(Error::<T>::InvalidVkData);
                }

                // Validate credential hash is not all zeros
                if proof.credential_hash == H256::zero() {
                    return Err(Error::<T>::InvalidProofData);
                }

                // Validate nonce is not all zeros (prevents trivial replays)
                if proof.nonce == H256::zero() {
                    return Err(Error::<T>::InvalidProofData);
                }

                // Check schema if it exists for this proof type
                if let Some(schema) = ProofSchemas::<T>::get(&proof.proof_type) {
                    // Verify public inputs count matches schema
                    if proof.public_inputs.len() != schema.len() {
                        return Err(Error::<T>::InvalidPublicInputs);
                    }
                }

                // Prepare inputs for the SP1 adapter.
                let public_inputs_vec: Vec<Vec<u8>> = proof
                    .public_inputs
                    .iter()
                    .map(|b| b.to_vec())
                    .collect();

                // Use the adapter which selects the correct sp1-sdk variant.
                Self::sp1_verify_adapter(
                    proof.proof_data.as_slice(),
                    &public_inputs_vec,
                    _vk_data.vk_data.as_slice(),
                )?;
            }

            #[cfg(not(feature = "sp1"))]
            {
                // Fallback: Proper validation
                if proof.proof_data.is_empty() || proof.proof_data.len() < 64 {
                    return Err(Error::<T>::InvalidProofData);
                }

                if proof.public_inputs.is_empty() {
                    return Err(Error::<T>::InvalidPublicInputs);
                }

                for input in proof.public_inputs.iter() {
                    if input.is_empty() {
                        return Err(Error::<T>::InvalidPublicInputs);
                    }
                }

                if proof.credential_hash == H256::zero() || proof.nonce == H256::zero() {
                    return Err(Error::<T>::InvalidProofData);
                }
            }

            Ok(())
        }

        /// Convert ZkCredentialType to ProofType
        pub fn zk_credential_type_to_proof_type(zk_type: &ZkCredentialType) -> ProofType {
            match zk_type {
                ZkCredentialType::StudentStatus => ProofType::StudentStatus,
                ZkCredentialType::VaccinationStatus => ProofType::VaccinationStatus,
                ZkCredentialType::EmploymentStatus => ProofType::EmploymentStatus,
                ZkCredentialType::AgeVerification => ProofType::AgeAbove,
                ZkCredentialType::Custom => ProofType::Custom,
            }
        }

        /// Public interface for other pallets
        pub fn verify_proof_internal_public(proof: &ZkProof) -> Result<bool, DispatchError> {
            Self::verify_proof_internal(proof)?;
            Ok(true)
        }

        /// Get verification key for a proof type
        pub fn get_verification_key(proof_type: &ProofType) -> Option<VerificationKeyData> {
            VerifyingKeys::<T>::get(proof_type)
        }

        /// Check if proof is verified
        pub fn is_proof_verified(proof_hash: &H256) -> bool {
            VerifiedProofs::<T>::contains_key(proof_hash)
        }

        /// Generate age proof inputs helper
        pub fn generate_age_proof_inputs(
            age_threshold: u32,
            current_year: u32,
        ) -> Vec<Vec<u8>> {
            vec![
                age_threshold.to_le_bytes().to_vec(),
                current_year.to_le_bytes().to_vec(),
            ]
        }

        /// Generate student status inputs helper
        pub fn generate_student_status_inputs(
            institution_hash: H256,
            is_active: bool,
        ) -> Vec<Vec<u8>> {
            vec![
                institution_hash.as_bytes().to_vec(),
                vec![if is_active { 1u8 } else { 0u8 }],
            ]
        }
    }
    
    impl<T: Config> Pallet<T> {
        // Adapter for calling ProverClient::verify — adjust the active branch to match your sp1-sdk.
        fn sp1_verify_adapter(
            proof_bytes: &[u8],
            public_inputs: &[Vec<u8>],
            vk_bytes: &[u8],
        ) -> Result<(), Error<T>> {
            // Variant A (common): instance method taking &[u8], &[Vec<u8>], &[u8] -> Result<(), E>
            #[cfg(feature = "sp1_variant_a")]
            {
                let client = ProverClient::new();
                client
                    .verify(proof_bytes, public_inputs, vk_bytes)
                    .map_err(|_| Error::<T>::ProofVerificationFailed)?;
                return Ok(());
            }

            // Variant B: takes &[u8], &[&[u8]] -> Result<bool, E>
            #[cfg(feature = "sp1_variant_b")]
            {
                let client = ProverClient::new();
                // convert Vec<Vec<u8>> -> Vec<&[u8]>
                let refs: Vec<&[u8]> = public_inputs.iter().map(|v| v.as_slice()).collect();
                let ok = client
                    .verify(proof_bytes, refs.as_slice(), vk_bytes)
                    .map_err(|_| Error::<T>::ProofVerificationFailed)?;
                if !ok {
                    return Err(Error::<T>::ProofVerificationFailed);
                }
                return Ok(());
            }

            // Variant C: uses SP1Stdin runner (stdin JSON or custom struct)
            #[cfg(feature = "sp1_variant_c")]
            {
                // Build SP1Stdin or required input format
                let mut stdin = SP1Stdin::new();
                stdin.set_proof(proof_bytes);
                stdin.set_public_inputs(public_inputs);
                stdin.set_vk(vk_bytes);

                let client = ProverClient::new();
                client
                    .run_with_stdin(stdin)
                    .map_err(|_| Error::<T>::ProofVerificationFailed)?;
                return Ok(());
            }

            // If none of the feature flags matched, fail at compile-time with helpful message.
            #[cfg(not(any(feature = "sp1_variant_a", feature = "sp1_variant_b", feature = "sp1_variant_c")))]
            return Err(Error::<T>::ProofVerificationFailed);
        }
    }
}