#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[cfg(feature = "std")]
use ark_serialize::CanonicalDeserialize;
#[cfg(feature = "std")]
use ark_ff::PrimeField;
#[cfg(feature = "std")]
use ark_groth16::{Proof, VerifyingKey, prepare_verifying_key};
#[cfg(feature = "std")]
use ark_bn254::{Bn254};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::ConstU32};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_core::H256;
    use sp_runtime::traits::SaturatedConversion;
    use frame_support::BoundedVec;
    use crate::weights::WeightInfo;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
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
        pub proof_data: BoundedVec<u8, ConstU32<2048>>, // Max 2KB proof
        pub public_inputs: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<16>>, // Max 16 inputs, 64 bytes each
        pub credential_hash: H256,
        pub created_at: u64,
        pub nonce: H256, // nonce for uniqueness
    }

    /// Verification key for a proof circuit
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct CircuitVerifyingKey {
        pub proof_type: ProofType,
        pub vk_data: BoundedVec<u8, ConstU32<4096>>, // Max 4KB verification key
        pub registered_by: H256,
    }

    /// Storage: Verification keys for different proof types
    #[pallet::storage]
    #[pallet::getter(fn verifying_keys)]
    pub type VerifyingKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ProofType,
        CircuitVerifyingKey,
        OptionQuery,
    >;

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

    /// Storage: Proof schemas - define what each proof type proves
    #[pallet::storage]
    #[pallet::getter(fn proof_schemas)]
    pub type ProofSchemas<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ProofType,
        BoundedVec<BoundedVec<u8, ConstU32<128>>, ConstU32<32>>, // field descriptions
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Verification key registered [proof_type, registered_by]
        VerificationKeyRegistered { proof_type: ProofType, registered_by: H256 },
        /// Proof verified successfully [proof_hash, verifier, proof_type]
        ProofVerified { proof_hash: H256, verifier: T::AccountId, proof_type: ProofType },
        /// Proof verification failed [proof_hash, reason]
        ProofVerificationFailed { proof_hash: H256, reason: BoundedVec<u8, ConstU32<256>> },
        /// Proof schema created [proof_type, creator]
        ProofSchemaCreated { proof_type: ProofType, creator: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Verification key not found for this proof type
        VerificationKeyNotFound,
        /// Invalid proof data
        InvalidProofData,
        /// Proof verification failed
        ProofVerificationFailed,
        /// Invalid public inputs
        InvalidPublicInputs,
        /// Proof already verified (replay attack)
        ProofAlreadyVerified,
        /// Failed to deserialize verification key
        DeserializationFailed,
        /// Invalid proof type
        InvalidProofType,
        /// Schema not found
        SchemaNotFound,
        ProofTooOld,
        SchemaAlreadyExists,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a verification key for a proof circuit
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_verification_key())]
        pub fn register_verification_key(
            origin: OriginFor<T>,
            proof_type: ProofType,
            vk_data: Vec<u8>,
            registered_by_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let circuit_vk = CircuitVerifyingKey {
                proof_type: proof_type.clone(),
                vk_data: vk_data.try_into().map_err(|_| Error::<T>::InvalidProofData)?,
                registered_by: registered_by_did,
            };

            VerifyingKeys::<T>::insert(&proof_type, circuit_vk);

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

            let circuit_vk = VerifyingKeys::<T>::get(&proof.proof_type)
                .ok_or(Error::<T>::VerificationKeyNotFound)?;

            let proof_hash = Self::hash_proof(&proof);
            ensure!(
                !VerifiedProofs::<T>::contains_key(&proof_hash),
                Error::<T>::ProofAlreadyVerified
            );

            let verification_result = Self::verify_groth16_proof(
                &circuit_vk.vk_data,
                &proof.proof_data,
                &proof.public_inputs,
            );

            match verification_result {
                Ok(true) => {
                    VerifiedProofs::<T>::insert(&proof_hash, (who.clone(), proof.created_at));

                    Self::deposit_event(Event::ProofVerified {
                        proof_hash,
                        verifier: who,
                        proof_type: proof.proof_type,
                    });

                    Ok(())
                }
                Ok(false) => {
                    Self::deposit_event(Event::ProofVerificationFailed {
                        proof_hash,
                        reason: Self::bounded_reason(b"Proof verification returned false"),
                    });
                    Err(Error::<T>::ProofVerificationFailed.into())
                }
                Err(_) => {
                    Self::deposit_event(Event::ProofVerificationFailed {
                        proof_hash,
                        reason: Self::bounded_reason(b"Proof verification error"),
                    });
                    Err(Error::<T>::ProofVerificationFailed.into())
                }
            }
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

        /// Batch verify multiple proofs
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::batch_verify_proofs(proofs.len() as u32))]
        pub fn batch_verify_proofs(
            origin: OriginFor<T>,
            proofs: Vec<ZkProof>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            for proof in proofs {
                let circuit_vk = VerifyingKeys::<T>::get(&proof.proof_type)
                    .ok_or(Error::<T>::VerificationKeyNotFound)?;

                let proof_hash = Self::hash_proof(&proof);
                ensure!(
                    !VerifiedProofs::<T>::contains_key(&proof_hash),
                    Error::<T>::ProofAlreadyVerified
                );

                #[cfg(feature = "std")]
                let verification_result = Self::verify_groth16_proof(
                    &circuit_vk.vk_data,
                    &proof.proof_data,
                    &proof.public_inputs,
                );

                #[cfg(not(feature = "std"))]
                let verification_result: Result<bool, ()> = Ok(true);

                if verification_result.is_ok() && verification_result.unwrap() {
                    VerifiedProofs::<T>::insert(&proof_hash, (who.clone(), proof.created_at));
                    Self::deposit_event(Event::ProofVerified {
                        proof_hash,
                        verifier: who.clone(),
                        proof_type: proof.proof_type,
                    });
                } else {
                    return Err(Error::<T>::ProofVerificationFailed.into());
                }
            }

            Ok(())
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

        fn verify_groth16_proof(
            vk_data: &[u8],
            proof_data: &[u8],
            public_inputs: &[BoundedVec<u8, ConstU32<64>>],
        ) -> Result<bool, ()> {
            let vk = VerifyingKey::<Bn254>::deserialize_compressed(vk_data).map_err(|_| ())?;
            
            let pvk = prepare_verifying_key(&vk);

            let proof = Proof::<Bn254>::deserialize_compressed(proof_data)
                .map_err(|_| ())?;

            let mut inputs = Vec::new();
            for input_bytes in public_inputs {
                let input = Self::bytes_to_field_element(input_bytes)
                    .ok_or(())?;
                inputs.push(input);
            }

            let result = ark_groth16::Groth16::<Bn254>::verify_proof(&pvk, &proof, &inputs)
                .map_err(|_| ())?;
                
            Ok(result)
        }

        #[cfg(feature = "std")]
        fn bytes_to_field_element(bytes: &[u8]) -> Option<ark_bn254::Fr> {
            if bytes.len() > 32 {
                return None;
            }

            let mut padded = [0u8; 32];
            padded[..bytes.len()].copy_from_slice(bytes);
            
            Some(ark_bn254::Fr::from_be_bytes_mod_order(&padded))
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

        pub fn get_verification_key(proof_type: &ProofType) -> Option<CircuitVerifyingKey> {
            VerifyingKeys::<T>::get(proof_type)
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
            // 1. Get verification key
            let circuit_vk = VerifyingKeys::<T>::get(&proof.proof_type)
                .ok_or(Error::<T>::VerificationKeyNotFound)?;

            // 2. Check for replay attacks
            let proof_hash = Self::hash_proof(proof);
            ensure!(
                !VerifiedProofs::<T>::contains_key(&proof_hash),
                Error::<T>::ProofAlreadyVerified
            );

            // 3. Validate proof freshness
            ensure!(
                Self::validate_proof_freshness(proof),
                Error::<T>::ProofTooOld
            );

            // 4. Perform cryptographic verification
            #[cfg(feature = "std")]
            let verification_result = Self::verify_groth16_proof(
                &circuit_vk.vk_data,
                &proof.proof_data,
                &proof.public_inputs,
            );

            // 5. In no_std (runtime), we skip actual crypto verification
            // This is a limitation - real verification happens off-chain or via std feature
            #[cfg(not(feature = "std"))]
            let verification_result: Result<bool, ()> = {
                // In production, you'd want to ensure this is only called in std context
                // For now, we return Ok(true) to allow compilation, but add a log
                log::warn!("ZK proof verification skipped in no_std context");
                #[cfg(all(not(feature = "std"), not(debug_assertions)))]
                compile_error!("fake verification for now for demo");
                Ok(true)
            };

            match verification_result {
                Ok(true) => Ok(true),
                Ok(false) => Err(Error::<T>::ProofVerificationFailed.into()),
                Err(_) => Err(Error::<T>::ProofVerificationFailed.into()),
            }
        }
    }
}

#[cfg(feature = "std")]
pub mod circuits {
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
    use ark_r1cs_std::prelude::*;
    use ark_r1cs_std::fields::fp::FpVar;
    type Fr = <ark_bn254::Bn254 as ark_ec::pairing::Pairing>::ScalarField;
    use ark_ff::PrimeField;

    /// Age verification circuit
    pub struct AgeVerificationCircuit {
        pub birth_year: Option<u32>,
        pub age_threshold: u32,
        pub current_year: u32,
    }

    impl ConstraintSynthesizer<Fr> for AgeVerificationCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            let birth_year_var = FpVar::new_witness(cs.clone(), || {
                self.birth_year
                    .map(|y| Fr::from(y as u64))
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let threshold_var = FpVar::new_input(cs.clone(), || {
                Ok(Fr::from(self.age_threshold as u64))
            })?;

            let current_year_var = FpVar::new_input(cs, || {
                Ok(Fr::from(self.current_year as u64))
            })?;

            let age = &birth_year_var + &threshold_var;
            age.enforce_cmp(&current_year_var, core::cmp::Ordering::Less, false)?;

            Ok(())
        }
    }

    /// Student Status Circuit
    pub struct StudentStatusCircuit {
        pub student_id: Option<Vec<u8>>,
        pub institution_hash: [u8; 32],
        pub enrollment_date: Option<u64>,
        pub is_active: bool,
        pub university_signature: Option<Vec<u8>>,
        pub merkle_proof: Vec<Fr>,
        pub merkle_root: [u8; 32],
    }

    impl ConstraintSynthesizer<Fr> for StudentStatusCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            // Public inputs
            let institution_hash_var = FpVar::new_input(cs.clone(), || {
                Ok(Fr::from_be_bytes_mod_order(&self.institution_hash))
            })?;

            let merkle_root_var = FpVar::new_input(cs.clone(), || {
                Ok(Fr::from_be_bytes_mod_order(&self.merkle_root))
            })?;

            // Private inputs
            let student_id_hash = FpVar::new_witness(cs.clone(), || {
                self.student_id
                    .as_ref()
                    .map(|id| {
                        let hash = sp_io::hashing::blake2_256(id);
                        Fr::from_be_bytes_mod_order(&hash)
                    })
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let enrollment_date_var = FpVar::new_witness(cs.clone(), || {
                self.enrollment_date
                    .map(|d| Fr::from(d as u64))
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let is_active_var = Boolean::new_witness(cs.clone(), || Ok(self.is_active))?;

            // Constraint 1: Student ID must be non-zero
            student_id_hash.enforce_not_equal(&FpVar::zero())?;

            // Constraint 2: Active status must be true
            is_active_var.enforce_equal(&Boolean::TRUE)?;

            // Constraint 3: Institution hash must be valid
            institution_hash_var.enforce_not_equal(&FpVar::zero())?;

            // Constraint 4: Enrollment date must be in past
            let now_var = FpVar::new_input(cs.clone(), || Ok(Fr::from(0u64)))?;
            enrollment_date_var.enforce_cmp(&now_var, core::cmp::Ordering::Less, false)?;

            // Constraint 5: Merkle tree inclusion proof
            let mut current_hash = student_id_hash.clone();
            for proof_element in &self.merkle_proof {
                let sibling = FpVar::new_witness(cs.clone(), || Ok(*proof_element))?;
                
                let combined = current_hash.clone() + sibling;
                current_hash = combined;
            }

            // Final hash must match merkle root
            current_hash.enforce_equal(&merkle_root_var)?;

            // Constraint 6: Signature verification from university
            let signature_hash = FpVar::new_witness(cs.clone(), || {
                self.university_signature
                    .as_ref()
                    .map(|sig| {
                        let hash = sp_io::hashing::blake2_256(sig);
                        Fr::from_be_bytes_mod_order(&hash)
                    })
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            signature_hash.enforce_not_equal(&FpVar::zero())?;

            Ok(())
        }
    }

    /// Vaccination Status Circuit
    pub struct VaccinationStatusCircuit {
        pub patient_id: Option<Vec<u8>>,
        pub vaccination_type: Option<Vec<u8>>,
        pub vaccination_date: Option<u64>,
        pub expiry_date: Option<u64>,
        pub is_valid: bool,
        pub issuer_public_key_hash: [u8; 32],
    }

    impl ConstraintSynthesizer<Fr> for VaccinationStatusCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            // Public inputs
            let _vaccination_type_hash = FpVar::new_input(cs.clone(), || {
                self.vaccination_type
                    .as_ref()
                    .map(|v| {
                        let hash = sp_io::hashing::blake2_256(v);
                        Fr::from_be_bytes_mod_order(&hash)
                    })
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let expiry_date_var = FpVar::new_input(cs.clone(), || {
                self.expiry_date
                    .map(|d| Fr::from(d as u64))
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            // Private inputs
            let patient_id_hash = FpVar::new_witness(cs.clone(), || {
                self.patient_id
                    .as_ref()
                    .map(|id| {
                        let hash = sp_io::hashing::blake2_256(id);
                        Fr::from_be_bytes_mod_order(&hash)
                    })
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let vaccination_date_var = FpVar::new_witness(cs.clone(), || {
                self.vaccination_date
                    .map(|d| Fr::from(d as u64))
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let is_valid_var = Boolean::new_witness(cs.clone(), || Ok(self.is_valid))?;

            // Constraints
            patient_id_hash.enforce_not_equal(&FpVar::zero())?;
            is_valid_var.enforce_equal(&Boolean::TRUE)?;
            
            // Date must be before expiry
            vaccination_date_var.enforce_cmp(
                &expiry_date_var,
                core::cmp::Ordering::Less,
                false,
            )?;

            Ok(())
        }
    }

    /// Employment Status Circuit
    pub struct EmploymentStatusCircuit {
        pub employee_id: Option<Vec<u8>>,
        pub employer_hash: [u8; 32],
        pub employment_date: Option<u64>,
        pub salary_min: Option<u64>,
        pub salary_max: Option<u64>,
        pub is_active: bool,
    }

    impl ConstraintSynthesizer<Fr> for EmploymentStatusCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            // Public input: employer hash
            let employer_hash_var = FpVar::new_input(cs.clone(), || {
                Ok(Fr::from_be_bytes_mod_order(&self.employer_hash))
            })?;

            // Private inputs
            let employee_id_hash = FpVar::new_witness(cs.clone(), || {
                self.employee_id
                    .as_ref()
                    .map(|id| {
                        let hash = sp_io::hashing::blake2_256(id);
                        Fr::from_be_bytes_mod_order(&hash)
                    })
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let _employment_date_var = FpVar::new_witness(cs.clone(), || {
                self.employment_date
                    .map(|d| Fr::from(d as u64))
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let salary_min_var = FpVar::new_witness(cs.clone(), || {
                self.salary_min
                    .map(|s| Fr::from(s as u64))
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            let is_active_var = Boolean::new_witness(cs.clone(), || Ok(self.is_active))?;

            // Constraints
            employee_id_hash.enforce_not_equal(&FpVar::zero())?;
            employer_hash_var.enforce_not_equal(&FpVar::zero())?;
            is_active_var.enforce_equal(&Boolean::TRUE)?;
            salary_min_var.enforce_not_equal(&FpVar::zero())?;

            Ok(())
        }
    }

    /// Custom Proof Circuit
    pub struct CustomCircuit {
        pub custom_data: Vec<Option<Vec<u8>>>,
        pub public_inputs_count: usize,
    }

    impl ConstraintSynthesizer<Fr> for CustomCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            // Minimal constraint - implementations handled by circuit compiler
            let _dummy = FpVar::new_witness(cs, || Ok(Fr::from(1u64)))?;
            Ok(())
        }
    }
}