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
    use sp1_sdk::verify::verify_proof;
    use sp1_sdk::utils::load_verifying_key;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type WeightInfo: WeightInfo;
    }

    /// Proof types supported (kept for compatibility; SP1 handles generically)
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

    /// ZK Proof structure (updated for SP1: proof_data is SP1 proof bytes)
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ZkProof {
        pub proof_type: ProofType,
        pub proof_data: BoundedVec<u8, ConstU32<8192>>, // SP1 proofs ~4-8KB
        pub public_inputs: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<16>>,
        pub credential_hash: H256,
        pub created_at: u64,
        pub nonce: H256,
    }

    /// Storage: Verified proofs (to prevent replay attacks)
    #[pallet::storage]
    #[pallet::getter(fn verified_proofs)]
    pub type VerifiedProofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // proof hash
        (T::AccountId, u64), // (verifier, timestamp)
        OptionQuery,
    >;

    /// Storage: SP1 Verification Key (global for simplicity; expand per proof_type if needed)
    #[pallet::storage]
    #[pallet::getter(fn verification_key)]
    pub type VerificationKey<T: Config> = StorageValue<_, BoundedVec<u8, ConstU32<2048>>, ValueQuery>;  // SP1 VK ~1.8KB

    /// Storage: Proof schemas (kept; not SP1-specific)
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
        VerificationKeyRegistered { registered_by: H256 },
        ProofVerified { proof_hash: H256, verifier: T::AccountId, proof_type: ProofType },
        ProofVerificationFailed { proof_hash: H256, reason: BoundedVec<u8, ConstU32<256>> },
        ProofSchemaCreated { proof_type: ProofType, creator: T::AccountId },
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register SP1 verification key (admin/root)
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_verification_key())]
        pub fn register_verification_key(
            origin: OriginFor<T>,
            vk_data: Vec<u8>,
            registered_by_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let bounded_vk = vk_data.try_into().map_err(|_| Error::<T>::InvalidProofData)?;
            VerificationKey::<T>::put(bounded_vk);

            Self::deposit_event(Event::VerificationKeyRegistered { 
                registered_by: registered_by_did 
            });

            Ok(())
        }

        /// Verify a ZK proof using SP1 (on-chain)
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

            #[cfg(feature = "sp1")]
            {
                let vk = VerificationKey::<T>::get();
                if vk.is_empty() {
                    return Err(Error::<T>::VerificationKeyNotFound.into());
                }

                // On-chain SP1 verification
                SP1Verifier::verify(&proof.proof_data, &vk)
                    .map_err(|_| Error::<T>::ProofVerificationFailed)?;

                VerifiedProofs::<T>::insert(&proof_hash, (who.clone(), proof.created_at));

                Self::deposit_event(Event::ProofVerified {
                    proof_hash,
                    verifier: who,
                    proof_type: proof.proof_type,
                });

                Ok(())
            }

            #[cfg(not(feature = "sp1"))]
            {
                // Fallback if feature not enabled (add your old logic or error)
                Err(Error::<T>::ProofVerificationFailed.into())
            }
        }

        /// Create proof schema (kept; not SP1-specific)
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

            let bounded_descriptions: BoundedVec<BoundedVec<u8, ConstU32<128>>, ConstU32<32>> = field_descriptions.into_iter()
                .map(|desc| desc.try_into().map_err(|_| Error::<T>::InvalidProofData))
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| Error::<T>::InvalidProofData)?;

            ProofSchemas::<T>::insert(&proof_type, bounded_descriptions);

            Self::deposit_event(Event::ProofSchemaCreated {
                proof_type,
                creator: who,
            });
            Ok(())
        }

        /// Batch verify multiple proofs using SP1
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::batch_verify_proofs())]
        pub fn batch_verify_proofs(
            origin: OriginFor<T>,
            proofs: Vec<ZkProof>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            #[cfg(feature = "sp1")]
            {
                let vk = VerificationKey::<T>::get();
                if vk.is_empty() {
                    return Err(Error::<T>::VerificationKeyNotFound.into());
                }

                for proof in proofs {
                    let proof_hash = Self::hash_proof(&proof);
                    ensure!(
                        !VerifiedProofs::<T>::contains_key(&proof_hash),
                        Error::<T>::ProofAlreadyVerified
                    );

                    SP1Verifier::verify(&proof.proof_data, &vk)
                        .map_err(|_| Error::<T>::ProofVerificationFailed)?;

                    VerifiedProofs::<T>::insert(&proof_hash, (who.clone(), proof.created_at));
                    Self::deposit_event(Event::ProofVerified {
                        proof_hash,
                        verifier: who.clone(),
                        proof_type: proof.proof_type,
                    });
                }

                Ok(())
            }

            #[cfg(not(feature = "sp1"))]
            {
                Err(Error::<T>::ProofVerificationFailed.into())
            }
        }
    }

    impl<T: Config> Pallet<T> {
        fn bounded_reason(s: &'static [u8]) -> BoundedVec<u8, ConstU32<256>> {
            s.to_vec().try_into().expect("static reason fits in BoundedVec<256>")
        }

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

        pub fn generate_age_proof_inputs(
            age_threshold: u32,
            current_year: u32,
        ) -> Vec<Vec<u8>> {
            vec![
                age_threshold.to_le_bytes().to_vec(),
                current_year.to_le_bytes().to_vec(),
            ]
        }

        pub fn generate_student_status_inputs(
            institution_hash: H256,
            is_active: bool,
        ) -> Vec<Vec<u8>> {
            vec![
                institution_hash.as_bytes().to_vec(),
                vec![if is_active { 1u8 } else { 0u8 }],
            ]
        }

        pub fn is_proof_verified(proof_hash: &H256) -> bool {
            VerifiedProofs::<T>::contains_key(proof_hash)
        }

        fn validate_proof_freshness(proof: &ZkProof) -> bool {
            let current_time = frame_system::Pallet::<T>::block_number();
            let current_time_u64 = current_time.saturated_into::<u64>();
            
            if proof.created_at > current_time_u64 {
                return false;
            }
            let proof_age = current_time_u64.saturating_sub(proof.created_at);

            proof_age <= 3600u64
        }

        pub fn zk_credential_type_to_proof_type(zk_type: &ZkCredentialType) -> ProofType {
            match zk_type {
                ZkCredentialType::StudentStatus => ProofType::StudentStatus,
                ZkCredentialType::VaccinationStatus => ProofType::VaccinationStatus,
                ZkCredentialType::EmploymentStatus => ProofType::EmploymentStatus,
                ZkCredentialType::AgeVerification => ProofType::AgeAbove,
                ZkCredentialType::Custom => ProofType::Custom,
            }
        }

        pub fn verify_proof_internal(proof: &ZkProof) -> Result<bool, DispatchError> {
            let proof_hash = Self::hash_proof(proof);
            ensure!(
                !VerifiedProofs::<T>::contains_key(&proof_hash),
                Error::<T>::ProofAlreadyVerified
            );

            ensure!(
                Self::validate_proof_freshness(proof),
                Error::<T>::ProofTooOld
            );

            #[cfg(feature = "sp1")]
            {
                let vk = VerificationKey::<T>::get();
                if vk.is_empty() {
                    return Err(Error::<T>::VerificationKeyNotFound.into());
                }

                SP1Verifier::verify(&proof.proof_data, &vk)
                    .map_err(|_| Error::<T>::ProofVerificationFailed)?;

                Ok(true)
            }

            #[cfg(not(feature = "sp1"))]
            {
                Err(Error::<T>::ProofVerificationFailed.into())
            }
        }
    }
}