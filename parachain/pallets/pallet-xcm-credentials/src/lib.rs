#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::Time,
    };
    use frame_system::pallet_prelude::*;
    use codec::Encode;
    use sp_std::vec::Vec;
    use sp_core::H256;
    use xcm::latest::{
        Instruction,
        Location,
        Junction,
        Junctions,
        Xcm,
        OriginKind,
    };
    use pallet_xcm::Pallet as XcmPallet;
    use frame_support::BoundedVec;
    use crate::weights::WeightInfo;
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::marker::PhantomData;
    use xcm::prelude::SendXcm;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_xcm::Config {
        type TimeProvider: Time;
        type WeightInfo: WeightInfo;
        type ParachainId: Get<cumulus_primitives_core::ParaId>; 
        type XcmOriginToTransactDispatchOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Location>;
        type ParachainIdentity: frame_support::traits::EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin, Success = Location>;
        #[pallet::constant]
        type DefaultXcmFee: Get<Weight>;
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        // List of trusted parachains [ParaId, TrustedBool]
        pub registered_parachains: Vec<(u32, bool)>,
        #[serde(skip)]
        pub _marker: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (para_id, trusted) in &self.registered_parachains {
                let registry = ParachainRegistry {
                    para_id: *para_id,
                    trusted: *trusted,
                    endpoint: None,
                };
                RegisteredParachains::<T>::insert(para_id, registry);
            }
        }
    }
    
    /// Cross-chain credential verification request
    #[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct XcmCredentialRequest {
        /// Source parachain ID
        pub source_para_id: u32,
        /// Credential hash to verify
        pub credential_hash: H256,
        /// Requester on source chain
        pub requester: BoundedVec<u8, ConstU32<4096>>,
        /// Request timestamp
        pub timestamp: u64,
    }

    /// Cross-chain credential verification response
    #[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct XcmCredentialResponse {
        pub target_para_id: u32,
        pub credential_hash: H256,
        pub is_valid: bool,
        pub metadata: BoundedVec<u8, ConstU32<4096>>,
        pub created_at: u64,
    }

    /// Registered parachains for cross-chain credentials
    #[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ParachainRegistry {
        /// Parachain ID
        pub para_id: u32,
        /// Trusted for credential verification
        pub trusted: bool,
        /// Endpoint info (optional)
        pub endpoint: Option<BoundedVec<u8, ConstU32<4096>>>,
    }

    /// Storage: Registered parachains
    #[pallet::storage]
    #[pallet::getter(fn registered_parachains)]
    pub type RegisteredParachains<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // para_id
        ParachainRegistry,
        OptionQuery,
    >;

    /// Storage: Pending cross-chain verification requests
    #[pallet::storage]
    #[pallet::getter(fn pending_requests)]
    pub type PendingRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // request hash
        XcmCredentialRequest,
        OptionQuery,
    >;

    /// Storage: Cross-chain verification results
    #[pallet::storage]
    #[pallet::getter(fn verification_results)]
    pub type VerificationResults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // credential hash
        BoundedVec<XcmCredentialResponse, ConstU32<10>>,
        ValueQuery,
    >;

    /// Storage: Exported credentials (credentials shared across chains)
    #[pallet::storage]
    #[pallet::getter(fn exported_credentials)]
    pub type ExportedCredentials<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        H256, // credential hash
        Blake2_128Concat,
        u32, // destination para_id
        bool, // exported
        ValueQuery,
    >;

    /// Storage: Imported credentials from other chains
    #[pallet::storage]
    #[pallet::getter(fn imported_credentials)]
    pub type ImportedCredentials<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32, // source para_id
        Blake2_128Concat,
        H256, // credential hash
        BoundedVec<u8, ConstU32<4096>>, // credential data
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Parachain registered [para_id]
        ParachainRegistered { para_id: u32 },
        /// Parachain deregistered [para_id]
        ParachainDeregistered { para_id: u32 },
        /// Cross-chain verification requested [credential_hash, target_para_id]
        VerificationRequested {
            credential_hash: H256,
            target_para_id: u32,
        },
        /// Cross-chain verification response received [credential_hash, is_valid]
        VerificationResponseReceived {
            credential_hash: H256,
            is_valid: bool,
        },
        /// Credential exported [credential_hash, destination_para_id]
        CredentialExported {
            credential_hash: H256,
            destination_para_id: u32,
        },
        /// Credential imported [credential_hash, source_para_id]
        CredentialImported {
            credential_hash: H256,
            source_para_id: u32,
        },
        /// XCM message sent [destination, message_hash]
        XcmMessageSent { destination: u32, message_hash: H256 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Parachain not registered
        ParachainNotRegistered,
        /// Parachain already registered
        ParachainAlreadyRegistered,
        /// Parachain not trusted
        ParachainNotTrusted,
        /// Invalid XCM message
        InvalidXcmMessage,
        /// XCM send failed
        XcmSendFailed,
        /// Request not found
        RequestNotFound,
        /// Credential not found
        CredentialNotFound,
        /// Credential not exported
        CredentialNotExported,
        /// Already exported
        AlreadyExported,
        /// Too many verification responses
        TooManyResponses,
        EncodingError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a parachain for cross-chain credentials
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_parachain())]
        pub fn register_parachain(
            origin: OriginFor<T>,
            para_id: u32,
            trusted: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                !RegisteredParachains::<T>::contains_key(para_id),
                Error::<T>::ParachainAlreadyRegistered
            );

            let registry = ParachainRegistry {
                para_id,
                trusted,
                endpoint: None,
            };

            RegisteredParachains::<T>::insert(para_id, registry);

            Self::deposit_event(Event::ParachainRegistered { para_id });

            Ok(())
        }

        /// Request credential verification from another parachain
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::request_cross_chain_verification())]
        pub fn request_cross_chain_verification(
            origin: OriginFor<T>,
            credential_hash: H256,
            target_para_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if parachain is registered
            let registry = RegisteredParachains::<T>::get(target_para_id)
                .ok_or(Error::<T>::ParachainNotRegistered)?;

            ensure!(registry.trusted, Error::<T>::ParachainNotTrusted);

            let requester_bv: BoundedVec<u8, ConstU32<4096>> = who.encode().try_into().map_err(|_| Error::<T>::InvalidXcmMessage)?;

            // Create verification request
            let request = XcmCredentialRequest {
                source_para_id: Self::get_current_para_id(),
                credential_hash,
                requester: requester_bv,
                timestamp: <T as Config>::TimeProvider::now().saturated_into::<u64>().saturated_into::<u64>(),
            };

            // Calculate request hash
            let request_hash = sp_io::hashing::blake2_256(&request.encode()).into();

            // Store pending request
            PendingRequests::<T>::insert(request_hash, request.clone());

            // Send XCM message
            Self::send_verification_request(target_para_id, credential_hash, request_hash)?;

            Self::deposit_event(Event::VerificationRequested {
                credential_hash,
                target_para_id,
            });

            Ok(())
        }

        /// Export a credential to another parachain
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::export_credential())]
        pub fn export_credential(
            origin: OriginFor<T>,
            credential_hash: H256,
            destination_para_id: u32,
            credential_data: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Check if parachain is registered
            ensure!(
                RegisteredParachains::<T>::contains_key(destination_para_id),
                Error::<T>::ParachainNotRegistered
            );

            // Check if not already exported
            ensure!(
                !ExportedCredentials::<T>::get(credential_hash, destination_para_id),
                Error::<T>::AlreadyExported
            );

            // Send credential via XCM
            Self::send_credential_export(
                destination_para_id,
                credential_hash,
                credential_data,
            )?;

            // Mark as exported
            ExportedCredentials::<T>::insert(credential_hash, destination_para_id, true);

            Self::deposit_event(Event::CredentialExported {
                credential_hash,
                destination_para_id,
            });

            Ok(())
        }

        /// Handle incoming credential import (called by XCM)
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::import_credential())]
        pub fn import_credential(
            origin: OriginFor<T>,
            source_para_id: u32,
            credential_hash: H256,
            credential_data: Vec<u8>,
        ) -> DispatchResult {
            let actual_para_id = Self::ensure_sibling_para(origin)?;
    
            // Ensure the sender isn't spoofing the ID
            ensure!(actual_para_id == source_para_id, Error::<T>::InvalidXcmMessage);

            // Verify source parachain is trusted
            let registry = RegisteredParachains::<T>::get(source_para_id)
                .ok_or(Error::<T>::ParachainNotRegistered)?;

            let credential_bv: BoundedVec<u8, ConstU32<4096>> = credential_data.try_into().map_err(|_| Error::<T>::InvalidXcmMessage)?;

            ImportedCredentials::<T>::insert(source_para_id, credential_hash, credential_bv);

            Self::deposit_event(Event::CredentialImported {
                credential_hash,
                source_para_id,
            });

            Ok(())
        }

        /// Handle verification response (called by XCM)
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::handle_verification_response())]
        pub fn handle_verification_response(
            origin: OriginFor<T>,
            credential_hash: H256,
            is_valid: bool,
            metadata: Vec<u8>,
        ) -> DispatchResult {
            let _origin = ensure_root(origin)?;

            let metadata_bv: BoundedVec<u8, ConstU32<4096>> = metadata.try_into().map_err(|_| Error::<T>::InvalidXcmMessage)?;

            let response = XcmCredentialResponse {
                target_para_id: Self::get_current_para_id(),
                credential_hash,
                is_valid,
                metadata: metadata_bv,
                created_at: <T as Config>::TimeProvider::now().saturated_into::<u64>().saturated_into::<u64>(),
            };

            // Store response
            VerificationResults::<T>::try_mutate(credential_hash, |responses| {
                responses
                    .try_push(response)
                    .map_err(|_| Error::<T>::TooManyResponses)?;
                Ok::<(), Error<T>>(())
            })?;

            Self::deposit_event(Event::VerificationResponseReceived {
                credential_hash,
                is_valid,
            });

            Ok(())
        }

        /// Deregister a parachain
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::deregister_parachain())]
        pub fn deregister_parachain(
            origin: OriginFor<T>,
            para_id: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            RegisteredParachains::<T>::remove(para_id);

            Self::deposit_event(Event::ParachainDeregistered { para_id });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Send verification request via XCM
        fn send_verification_request(
            target_para_id: u32,
            credential_hash: H256,
            request_hash: H256,
        ) -> DispatchResult {
            let destination = Location::new(
                1, // parent = 1
                [Junction::Parachain(target_para_id)]
            );

            let encoded_call = Self::encode_verification_request_call(credential_hash, request_hash);

            let double = encoded_call.try_into().map_err(|_| Error::<T>::EncodingError)?;
            let message = Xcm(vec![
                Instruction::Transact {
                    origin_kind: OriginKind::Native,
                    fallback_max_weight: {
                        let fee = T::DefaultXcmFee::get();
                        Some(xcm::v3::Weight::from_parts(fee.ref_time(), fee.proof_size()))
                    },
                    call: double,
                }
            ]);

            <T as pallet_xcm::Config>::XcmRouter::send_xcm(
                destination, 
                message
            ).map_err(|_| Error::<T>::XcmSendFailed)?;

            Ok(())
        }

        /// Send credential export via XCM
        fn send_credential_export(
            destination_para_id: u32,
            credential_hash: H256,
            credential_data: Vec<u8>,
        ) -> DispatchResult {
            let destination = Location::new(
                1,
                [Junction::Parachain(destination_para_id)]
            );

            let source_para_id = Self::get_current_para_id();
            let encoded_call = Self::encode_import_credential_call(source_para_id, credential_hash, credential_data);

            let double = encoded_call.try_into().map_err(|_| Error::<T>::EncodingError)?;
            let message = Xcm(vec![
                Instruction::Transact {
                    origin_kind: OriginKind::Native,
                    fallback_max_weight: {
                        let fee = T::DefaultXcmFee::get();
                        Some(xcm::v3::Weight::from_parts(fee.ref_time(), fee.proof_size()))
                    },
                    call: double,
                }
            ]);

            <T as pallet_xcm::Config>::XcmRouter::send_xcm(
                destination, 
                message
            ).map_err(|_| Error::<T>::XcmSendFailed)?;

            Ok(())
        }

        /// Get current parachain ID 
        fn get_current_para_id() -> u32 {
            <T as Config>::ParachainId::get().into()
        }

        /// Encode verification request call
        fn encode_verification_request_call(
            credential_hash: H256,
            request_hash: H256,
        ) -> sp_std::vec::Vec<u8> {
            (
                1u8,  // Pallet index 
                2u8, // Call index for handle_verification_request
                credential_hash,
                request_hash,
            )
            .encode()
        }

        /// Encode import credential call
        fn encode_import_credential_call(
            source_para_id: u32,
            credential_hash: H256,
            credential_data: Vec<u8>,
        ) -> sp_std::vec::Vec<u8> {
            // Use SCALE encoding
            (
                1u8,  // Pallet index
                3u8,  // Call index for import_credential
                source_para_id,
                credential_hash,
                credential_data,
            )
            .encode()
        }

        fn ensure_sibling_para(origin: OriginFor<T>) -> Result<u32, Error<T>> {
            // 1. Convert origin to Location
            let location = T::ParachainIdentity::ensure_origin(origin)
                .map_err(|_| Error::<T>::InvalidXcmMessage)?;

            // 2. Match specific XCM V5 pattern
            // We match X1(junctions_array), then look at the first item [0]
            if let Location { parents: 1, interior: Junctions::X1(ref junctions) } = location {
                // Check if the first junction is a Parachain ID
                if let Junction::Parachain(id) = junctions[0] {
                    return Ok(id);
                }
            }
            
            Err(Error::<T>::InvalidXcmMessage)
        }

        /// Check if credential is valid across chains
        pub fn is_credential_valid_cross_chain(credential_hash: &H256) -> bool {
            let responses = VerificationResults::<T>::get(credential_hash);
            
            if responses.len() == 0 {
                return false;
            }

            // FIX: Add timestamp validation (responses not older than 1 hour)
            let current_time: u64 = <T as Config>::TimeProvider::now().saturated_into::<u64>().saturated_into();
            let one_hour_secs = 3600u64;

            let valid_responses: Vec<_> = responses
                .iter()
                .filter(|r| {
                    // Check response is fresh (within 1 hour)
                    let response_age = current_time.saturating_sub(r.created_at);
                    response_age < one_hour_secs && r.is_valid
                })
                .collect();

            // Require at least 2/3 valid responses
            let required_consensus = (responses.len() as u32)
                .saturating_mul(2)
                .saturating_div(3)
                .saturating_add(1);

            valid_responses.len() >= required_consensus as usize
        }

        /// Get all verification responses for a credential
        pub fn get_verification_responses(
            credential_hash: &H256,
        ) -> Vec<XcmCredentialResponse> {
            VerificationResults::<T>::get(credential_hash).into_inner()
        }
    }
}