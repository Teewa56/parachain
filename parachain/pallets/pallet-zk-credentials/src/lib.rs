#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "std")]
use ark_serialize::CanonicalSerialize;

use ark_serialize::CanonicalDeserialize;
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Proof, VerifyingKey, prepare_verifying_key};
use ark_ff::PrimeField;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_core::H256;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    /// Proof types supported
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProofType {
        /// Prove age is above threshold without revealing exact age
        AgeAbove,
        /// Prove student status without revealing details
        StudentStatus,
        /// Prove vaccination without revealing health records
        VaccinationStatus,
        /// Prove employment without revealing salary
        EmploymentStatus,
        /// Custom proof type
        Custom,
    }

    /// ZK Proof structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct ZkProof {
        /// Proof type
        pub proof_type: ProofType,
        /// Serialized Groth16 proof
        pub proof_data: Vec<u8>,
        /// Public inputs (what's revealed)
        pub public_inputs: Vec<Vec<u8>>,
        /// Hash of the credential being proven
        pub credential_hash: H256,
        /// Proof creation timestamp
        pub created_at: u64,
    }

    /// Verification key for a proof circuit
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct CircuitVerifyingKey {
        /// Proof type this key is for
        pub proof_type: ProofType,
        /// Serialized verification key
        pub vk_data: Vec<u8>,
        /// Who registered this key
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
        Vec<Vec<u8>>, // field descriptions
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
        ProofVerificationFailed { proof_hash: H256, reason: Vec<u8> },
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a verification key for a proof circuit
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_verification_key(
            origin: OriginFor<T>,
            proof_type: ProofType,
            vk_data: Vec<u8>,
            registered_by_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let circuit_vk = CircuitVerifyingKey {
                proof_type: proof_type.clone(),
                vk_data,
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
        #[pallet::weight(50_000)]
        pub fn verify_proof(
            origin: OriginFor<T>,
            proof: ZkProof,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get verification key for this proof type
            let circuit_vk = VerifyingKeys::<T>::get(&proof.proof_type)
                .ok_or(Error::<T>::VerificationKeyNotFound)?;

            // Calculate proof hash to check for replay
            let proof_hash = Self::hash_proof(&proof);
            ensure!(
                !VerifiedProofs::<T>::contains_key(&proof_hash),
                Error::<T>::ProofAlreadyVerified
            );

            // Verify the proof
            let verification_result = Self::verify_groth16_proof(
                &circuit_vk.vk_data,
                &proof.proof_data,
                &proof.public_inputs,
            );

            match verification_result {
                Ok(true) => {
                    // Store verified proof
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
                        reason: b"Proof verification returned false".to_vec(),
                    });
                    Err(Error::<T>::ProofVerificationFailed.into())
                }
                Err(_) => {
                    Self::deposit_event(Event::ProofVerificationFailed {
                        proof_hash,
                        reason: b"Proof verification error".to_vec(),
                    });
                    Err(Error::<T>::ProofVerificationFailed.into())
                }
            }
        }

        /// Create a proof schema
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn create_proof_schema(
            origin: OriginFor<T>,
            proof_type: ProofType,
            field_descriptions: Vec<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ProofSchemas::<T>::insert(&proof_type, field_descriptions);

            Self::deposit_event(Event::ProofSchemaCreated {
                proof_type,
                creator: who,
            });

            Ok(())
        }

        /// Batch verify multiple proofs (more efficient)
        #[pallet::call_index(3)]
        #[pallet::weight(100_000)]
        pub fn batch_verify_proofs(
            origin: OriginFor<T>,
            proofs: Vec<ZkProof>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            for proof in proofs {
                // Get verification key
                let circuit_vk = VerifyingKeys::<T>::get(&proof.proof_type)
                    .ok_or(Error::<T>::VerificationKeyNotFound)?;

                // Check for replay
                let proof_hash = Self::hash_proof(&proof);
                ensure!(
                    !VerifiedProofs::<T>::contains_key(&proof_hash),
                    Error::<T>::ProofAlreadyVerified
                );

                // Verify
                let verification_result = Self::verify_groth16_proof(
                    &circuit_vk.vk_data,
                    &proof.proof_data,
                    &proof.public_inputs,
                );

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
        /// Hash a proof to detect replays
        fn hash_proof(proof: &ZkProof) -> H256 {
            let mut data = Vec::new();
            data.extend_from_slice(&proof.proof_data);
            for input in &proof.public_inputs {
                data.extend_from_slice(input);
            }
            data.extend_from_slice(proof.credential_hash.as_bytes());
            
            sp_io::hashing::blake2_256(&data).into()
        }

        /// Verify a Groth16 proof using arkworks
        fn verify_groth16_proof(
            vk_data: &[u8],
            proof_data: &[u8],
            public_inputs: &[Vec<u8>],
        ) -> Result<bool, ()> {
            // Deserialize verification key
            let vk = VerifyingKey::<Bn254>::deserialize_compressed(vk_data)
                .map_err(|_| ())?;
            
            // Prepare verification key
            let pvk = prepare_verifying_key(&vk);

            // Deserialize proof
            let proof = Proof::<Bn254>::deserialize_compressed(proof_data)
                .map_err(|_| ())?;

            // Convert public inputs to field elements
            let mut inputs = Vec::new();
            for input_bytes in public_inputs {
                // Convert bytes to Fr (field element)
                let input = Self::bytes_to_field_element(input_bytes)
                    .ok_or(())?;
                inputs.push(input);
            }

            // Verify the proof
            let result = ark_groth16::verify_proof(&pvk, &proof, &inputs)
                .map_err(|_| ())?;

            Ok(result)
        }

        /// Convert bytes to field element
        fn bytes_to_field_element(bytes: &[u8]) -> Option<Fr> {
            if bytes.len() > 32 {
                return None;
            }

            let mut padded = [0u8; 32];
            padded[..bytes.len()].copy_from_slice(bytes);
            
            Fr::from_be_bytes_mod_order(&padded).into()
        }

        /// Generate public inputs for age proof
        pub fn generate_age_proof_inputs(
            age_threshold: u32,
            current_year: u32,
        ) -> Vec<Vec<u8>> {
            vec![
                age_threshold.to_le_bytes().to_vec(),
                current_year.to_le_bytes().to_vec(),
            ]
        }

        /// Generate public inputs for student status proof
        pub fn generate_student_status_inputs(
            institution_hash: H256,
            is_active: bool,
        ) -> Vec<Vec<u8>> {
            vec![
                institution_hash.as_bytes().to_vec(),
                vec![if is_active { 1u8 } else { 0u8 }],
            ]
        }

        /// Check if a proof has been verified
        pub fn is_proof_verified(proof_hash: &H256) -> bool {
            VerifiedProofs::<T>::contains_key(proof_hash)
        }

        /// Get verification key for proof type
        pub fn get_verification_key(proof_type: &ProofType) -> Option<CircuitVerifyingKey> {
            VerifyingKeys::<T>::get(proof_type)
        }
    }
}

/// Circuit example for age verification
/// This would be compiled separately using arkworks
#[cfg(feature = "std")]
pub mod circuits {
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
    use ark_r1cs_std::prelude::*;
    use ark_bn254::Fr;

    /// Age verification circuit
    /// Proves: birthYear + threshold <= currentYear
    /// Public inputs: threshold, currentYear
    /// Private input: birthYear
    pub struct AgeVerificationCircuit {
        pub birth_year: Option<u32>,
        pub age_threshold: u32,
        pub current_year: u32,
    }

    impl ConstraintSynthesizer<Fr> for AgeVerificationCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            // Allocate private input (birth year)
            let birth_year_var = FpVar::new_witness(cs.clone(), || {
                self.birth_year
                    .map(|y| Fr::from(y as u64))
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;

            // Allocate public inputs
            let threshold_var = FpVar::new_input(cs.clone(), || {
                Ok(Fr::from(self.age_threshold as u64))
            })?;

            let current_year_var = FpVar::new_input(cs, || {
                Ok(Fr::from(self.current_year as u64))
            })?;

            // Constraint: birth_year + threshold <= current_year
            let age = &birth_year_var + &threshold_var;
            age.enforce_cmp(&current_year_var, core::cmp::Ordering::Less, false)?;

            Ok(())
        }
    }

    /// Student status circuit
    /// Proves: user has valid student credential
    /// Public inputs: institution_hash, is_active
    /// Private inputs: student_id, enrollment_date
    pub struct StudentStatusCircuit {
        pub student_id: Option<Vec<u8>>,
        pub institution_hash: [u8; 32],
        pub enrollment_date: Option<u64>,
        pub is_active: bool,
    }

    impl ConstraintSynthesizer<Fr> for StudentStatusCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            // This is a simplified version
            // Real implementation would verify signatures and more complex logic
            
            // Public input: is_active
            let is_active_var = Boolean::new_input(cs.clone(), || Ok(self.is_active))?;

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

            // Enforce that student_id_hash is non-zero (valid)
            student_id_hash.enforce_not_equal(&FpVar::zero())?;

            // Enforce active status
            is_active_var.enforce_equal(&Boolean::TRUE)?;

            Ok(())
        }
    }
}