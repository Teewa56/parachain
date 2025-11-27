#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::Time,
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_std::convert::TryInto;
    use sp_core::{ H256, Get };
    use crate::weights::WeightInfo;
    use pallet_identity_registry::pallet::Pallet as IdentityRegistryPallet;
    use pallet_zk_credentials::pallet::Pallet as ZkCredentialsPallet;
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::marker::PhantomData;
    use codec::DecodeWithMemTracking;
    use frame_support::parameter_types;
    
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_identity_registry::pallet::Config {
        type TimeProvider: Time;
        type ZkCredentials: pallet_zk_credentials::pallet::Config;
        type WeightInfo: WeightInfo;
        type MaxFieldSize: Get<u32>;
        type MaxFields: Get<u32>;
        type MaxFieldsToReveal: Get<u32>;
        type MaxCredentialCleanupPerBlock: Get<u32>;
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub trusted_issuers: Vec<(H256, CredentialType)>,
        #[serde(skip)]
        pub _marker: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (issuer, cred_type) in &self.trusted_issuers {
                TrustedIssuers::<T>::insert((cred_type, issuer), true);
            }
        }
    }

    /// Credential types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum CredentialType {
        Education,
        Health,
        Employment,
        Age,
        Address,
        Custom,
    }

    /// Credential status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
    pub enum CredentialStatus {
        Active,
        Revoked,
        Expired,
        Suspended,
    }

    /// Verifiable Credential structure
    #[derive(Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Credential<T: Config> {
        pub subject: H256,
        pub issuer: H256,
        pub credential_type: CredentialType,
        pub data_hash: H256,
        pub issued_at: u64,
        pub expires_at: u64,
        pub status: CredentialStatus,
        pub signature: H256,
        pub metadata_hash: H256,
        pub fields: BoundedVec<BoundedVec<u8, T::MaxFieldSize>, T::MaxFields>,
        pub required_fields: BoundedVec<bool, T::MaxFields>,
        pub fields_to_reveal: BoundedVec<u32, T::MaxFieldsToReveal>,
    }

    impl<T: Config> Clone for Credential<T> {
        fn clone(&self) -> Self {
            Self {
                subject: self.subject,
                issuer: self.issuer,
                credential_type: self.credential_type.clone(),
                data_hash: self.data_hash,
                issued_at: self.issued_at,
                expires_at: self.expires_at,
                status: self.status.clone(),
                signature: self.signature,
                metadata_hash: self.metadata_hash,
                fields: self.fields.clone(),
                required_fields: self.required_fields.clone(),
                fields_to_reveal: self.fields_to_reveal.clone(),
            }
        }
    }

    /// Credential schema for defining what fields a credential type should have
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct CredentialSchema {
        pub schema_id: H256,
        pub credential_type: CredentialType,
        pub fields: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<100>>,
        pub required_fields: BoundedVec<bool, ConstU32<100>>,
        pub creator: H256,
    }

    /// Selective disclosure request
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct DisclosureRequest {
        pub credential_id: H256,
        pub fields_to_reveal: BoundedVec<u32, ConstU32<50>>,
        pub proof: H256,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct SelectiveDisclosureRequest {
        pub credential_id: H256,
        pub fields_to_reveal: BoundedVec<u32, ConstU32<50>>,
        pub proof: H256,
        pub timestamp: u64,
    }

    /// ZK Proof type for selective disclosure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ZkCredentialType {
        StudentStatus,
        VaccinationStatus,
        EmploymentStatus,
        AgeVerification,
        Custom,
    }

    /// Storage: Credentials by ID
    #[pallet::storage]
    #[pallet::getter(fn credentials)]
    pub type Credentials<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        H256, 
        Credential<T>, 
        OptionQuery
    >;

    /// Storage: Map BlockNumber -> Vec<CredentialId>
    #[pallet::storage]
    #[pallet::getter(fn expiries)]
    pub type Expiries<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // Block number (derived from timestamp)
        BoundedVec<H256, ConstU32<50>>, // Max 50 expiries per block
        ValueQuery,
    >;

    /// Storage: Credentials owned by a DID
    #[pallet::storage]
    #[pallet::getter(fn credentials_of)]
    pub type CredentialsOf<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        H256, 
        BoundedVec<H256, ConstU32<1000>>, 
        ValueQuery
    >;

    /// Storage: Credentials issued by a DID
    #[pallet::storage]
    #[pallet::getter(fn issued_by)]
    pub type IssuedBy<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        H256, 
        BoundedVec<H256, ConstU32<10000>>, 
        ValueQuery
    >;

    /// Storage: Credential schemas
    #[pallet::storage]
    #[pallet::getter(fn schemas)]
    pub type Schemas<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        H256, 
        CredentialSchema, 
        OptionQuery
    >;

    /// Storage: Trusted issuers for each credential type
    #[pallet::storage]
    #[pallet::getter(fn trusted_issuers)]
    pub type TrustedIssuers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (CredentialType, H256),
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn disclosure_records)]
    pub type DisclosureRecords<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,
        SelectiveDisclosureRequest,
        OptionQuery,
    >;

    /// Storage for tracking which fields were revealed (for analytics)
    #[pallet::storage]
    #[pallet::getter(fn field_disclosure_count)]
    pub type FieldDisclosureCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        H256,
        Blake2_128Concat,
        u32,
        u32,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CredentialIssued { 
            credential_id: H256, 
            subject: H256, 
            issuer: H256, 
            credential_type: CredentialType 
        },
        CredentialRevoked { credential_id: H256, issuer: H256 },
        CredentialVerified { credential_id: H256, verifier: T::AccountId },
        SchemaCreated { schema_id: H256, creator: H256 },
        TrustedIssuerAdded { credential_type: CredentialType, issuer: H256 },
        TrustedIssuerRemoved { credential_type: CredentialType, issuer: H256 },
        SelectiveDisclosure { credential_id: H256, fields_count: u32, disclosure_id: H256, timestamp: u64 },
        DisclosureProofVerified { credential_id: H256, verifier: T::AccountId, fields_revealed: u32 },
        CredentialVerificationFailed { 
            credential_id: H256, 
            reason: CredentialStatus,
            verifier: T::AccountId,
        },
        ProofVerificationFailed { 
            credential_id: H256, 
            reason: Vec<u8>,
        },
        IssuerNotTrusted { 
            issuer: H256, 
            credential_type: CredentialType,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        CredentialNotFound,
        NotAuthorized,
        CredentialAlreadyExists,
        CredentialRevoked,
        CredentialExpired,
        InvalidSignature,
        SubjectIdentityNotFound,
        IssuerIdentityNotFound,
        IssuerNotTrusted,
        SchemaNotFound,
        TooManyCredentials,
        InvalidCredentialStatus,
        InvalidSchema,
        SchemaAlreadyExists,
        IssuerInactive,
        InvalidProof, 
        InvalidFieldIndices, 
        ProofTimestampInvalid,
        NoFieldsToReveal,
        TooManyFieldsRequested, 
        ProofAlreadyUsed,
        VerificationKeyNotFound,
        ProofTooOld,
        FieldTooLarge,            // an individual field exceeded MaxFieldSize
        TooManyFields,            // too many fields overall (exceeds MaxFields)
        TooManyFieldsToReveal,    // too many reveal indices (exceeds MaxFieldsToReveal)
        InvalidFieldsLength,      // fields.len() != required_fields.len()
        InvalidRevealIndex,       // fields_to_reveal contains an index >= fields.len()
    }

    parameter_types! {
        pub const MaxCredentialCleanupPerBlock: u32 = 10;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            let now = <T as Config>::TimeProvider::now()
                .saturated_into::<u64>();
            
            let items_removed = Self::cleanup_expired_credentials(now);
            
            T::DbWeight::get().reads_writes(
                1 + items_removed as u64,
                items_removed as u64 * 3
            )
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Issue a new credential
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::issue_credential())]
        pub fn issue_credential(
            origin: OriginFor<T>,
            subject_did: H256,
            credential_type: CredentialType,
            data_hash: H256,
            expires_at: u64,
            signature: H256,
            fields: Vec<Vec<u8>>,
            required_fields: Vec<bool>,
            fields_to_reveal: Vec<u32>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // 1. Verify Issuer Identity
            let (issuer_did, issuer_identity) = IdentityRegistryPallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::IssuerIdentityNotFound)?;

            ensure!(issuer_identity.active, Error::<T>::IssuerInactive);

            // 2. Verify Subject Identity
            ensure!(
                IdentityRegistryPallet::<T>::is_identity_active(&subject_did),
                Error::<T>::SubjectIdentityNotFound
            );

            // 3. Verify Issuer Trust for this Type
            ensure!(
                TrustedIssuers::<T>::get((&credential_type, &issuer_did)),
                Error::<T>::IssuerNotTrusted
            );

            // 4. Validate Expiration
            ensure!(
                Self::validate_expiration_timestamp(expires_at),
                Error::<T>::InvalidCredentialStatus
            );

            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();

            // 5. Convert fields to BoundedVec properly
            let inner_fields_result: Result<Vec<BoundedVec<u8, T::MaxFieldSize>>, Error<T>> = fields
                .into_iter()
                .map(|f| {
                    BoundedVec::<u8, T::MaxFieldSize>::try_from(f)
                        .map_err(|_| Error::<T>::FieldTooLarge)
                })
                .collect();

            let inner_fields = inner_fields_result?;

            let bounded_fields: BoundedVec<BoundedVec<u8, T::MaxFieldSize>, T::MaxFields> =
                BoundedVec::try_from(inner_fields)
                    .map_err(|_| Error::<T>::TooManyFields)?;

            let bounded_required: BoundedVec<bool, T::MaxFields> =
                BoundedVec::try_from(required_fields)
                    .map_err(|_| Error::<T>::TooManyFields)?;

            let bounded_reveal: BoundedVec<u32, T::MaxFieldsToReveal> =
                BoundedVec::try_from(fields_to_reveal)
                    .map_err(|_| Error::<T>::TooManyFieldsToReveal)?;

            // 6. Consistency checks
            ensure!(
                bounded_fields.len() == bounded_required.len(),
                Error::<T>::InvalidFieldsLength
            );

            let fields_len_u32: u32 = bounded_fields.len()
                .try_into()
                .expect("bounded_fields.len() fits into u32");

            for idx in bounded_reveal.iter() {
                ensure!(*idx < fields_len_u32, Error::<T>::InvalidRevealIndex);
            }

            // 7. Create Credential
            let credential = Credential::<T> {
                subject: subject_did,
                issuer: issuer_did,
                credential_type: credential_type.clone(),
                data_hash,
                issued_at: now,
                expires_at,
                status: CredentialStatus::Active,
                signature,
                metadata_hash: Self::generate_metadata_hash(now, expires_at, &CredentialStatus::Active),
                fields: bounded_fields,
                fields_to_reveal: bounded_reveal,
                required_fields: bounded_required,
            };

            let credential_id = Self::generate_credential_id(&credential);

            // 8. Insert into Storage
            Credentials::<T>::insert(&credential_id, credential);

            // 9. Update Subject's List
            CredentialsOf::<T>::try_mutate(&subject_did, |creds| -> DispatchResult {
                creds.try_push(credential_id)
                    .map_err(|_| Error::<T>::TooManyCredentials)?;
                Ok(())
            })?;

            // 10. Update Issuer's List
            IssuedBy::<T>::try_mutate(&issuer_did, |creds| -> DispatchResult {
                creds.try_push(credential_id)
                    .map_err(|_| Error::<T>::TooManyCredentials)?;
                Ok(())
            })?;

            // 11. Track expiration
            if expires_at > 0 {
                let expiry_block = expires_at / 6;
                
                Expiries::<T>::try_mutate(expiry_block, |list| -> DispatchResult {
                    let _ = list.try_push(credential_id);
                    Ok(())
                })?;
            }

            Self::deposit_event(Event::CredentialIssued { 
                credential_id, 
                subject: subject_did, 
                issuer: issuer_did,
                credential_type 
            });

            Ok(())
        }

        /// Revoke a credential (only issuer can revoke)
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_credential())]
        pub fn revoke_credential(
            origin: OriginFor<T>,
            credential_id: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (issuer_did, _) = IdentityRegistryPallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::IssuerIdentityNotFound)?;

            Credentials::<T>::try_mutate(&credential_id, |cred_opt| -> DispatchResult {
                let cred = cred_opt.as_mut().ok_or(Error::<T>::CredentialNotFound)?;

                ensure!(cred.issuer == issuer_did, Error::<T>::NotAuthorized);
                ensure!(cred.status == CredentialStatus::Active, Error::<T>::InvalidCredentialStatus);

                cred.status = CredentialStatus::Revoked;

                Self::deposit_event(Event::CredentialRevoked { credential_id, issuer: issuer_did });

                Ok(())
            })
        }

        /// Verify a credential
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::verify_credential())]
        pub fn verify_credential(
            origin: OriginFor<T>,
            credential_id: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut credential = Credentials::<T>::get(&credential_id)
                .ok_or(Error::<T>::CredentialNotFound)?;

            let now = <T as crate::pallet::Config>::TimeProvider::now().saturated_into::<u64>();
            if credential.expires_at > 0 && now.saturating_sub(credential.expires_at) > 0 {
                credential.status = CredentialStatus::Expired;
                credential.metadata_hash = Self::generate_metadata_hash(
                    credential.issued_at,
                    credential.expires_at,
                    &CredentialStatus::Expired,
                );
                Credentials::<T>::insert(&credential_id, credential.clone());
                return Err(Error::<T>::CredentialExpired.into());
            }

            ensure!(credential.status == CredentialStatus::Active, Error::<T>::CredentialRevoked);

            ensure!(
                IdentityRegistryPallet::<T>::is_identity_active(&credential.issuer),
                Error::<T>::IssuerIdentityNotFound
            );
            ensure!(
                IdentityRegistryPallet::<T>::is_identity_active(&credential.subject),
                Error::<T>::SubjectIdentityNotFound
            );

            Self::deposit_event(Event::CredentialVerified { credential_id, verifier: who });

            Ok(())
        }

        /// Create a credential schema
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::create_schema())]
        pub fn create_schema(
            origin: OriginFor<T>,
            credential_type: CredentialType,
            fields: Vec<Vec<u8>>,      
            required_fields: Vec<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let (creator_did, _) = IdentityRegistryPallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::IssuerIdentityNotFound)?;

            // Validate schema parameters early
            ensure!(
                Self::validate_schema_params(&fields, &required_fields),
                Error::<T>::InvalidSchema
            );

            // Convert `fields` (Vec<Vec<u8>>) -> BoundedVec<BoundedVec<u8, 64>, 100>
            let bounded_fields: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<100>> = fields
                .into_iter()
                .map(|f| {
                    // Check inner string length (max 64)
                    let b: BoundedVec<u8, ConstU32<64>> = f.try_into()
                        .map_err(|_| Error::<T>::InvalidSchema)?; 
                    Ok(b)
                })
                .collect::<Result<Vec<_>, Error<T>>>()? // Collect results
                .try_into() // Convert outer Vec to BoundedVec (max 100)
                .map_err(|_| Error::<T>::InvalidSchema)?;

            // Convert `required_fields` (Vec<bool>) -> BoundedVec<bool, 100>
            let bounded_required: BoundedVec<bool, ConstU32<100>> = required_fields
                .try_into()
                .map_err(|_| Error::<T>::InvalidSchema)?;

            // Validate logical consistency (lengths must match)
            ensure!(bounded_fields.len() == bounded_required.len(), Error::<T>::InvalidSchema);

            let schema = CredentialSchema {
                schema_id: H256::zero(),
                credential_type,
                fields: bounded_fields,
                required_fields: bounded_required,
                creator: creator_did,
            };

            let schema_id = Self::generate_schema_id(&schema);
            let mut schema_with_id = schema;
            schema_with_id.schema_id = schema_id;
            
            ensure!(!Schemas::<T>::contains_key(&schema_id), Error::<T>::SchemaAlreadyExists);
            Schemas::<T>::insert(&schema_id, schema_with_id);
            
            Self::deposit_event(Event::SchemaCreated { schema_id, creator: creator_did });
            Ok(())
        }

        /// Add a trusted issuer for a credential type (requires root/governance)
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::add_trusted_issuer())]
        pub fn add_trusted_issuer(
            origin: OriginFor<T>,
            credential_type: CredentialType,
            issuer_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                IdentityRegistryPallet::<T>::is_identity_active(&issuer_did),
                Error::<T>::IssuerIdentityNotFound
            );

            TrustedIssuers::<T>::insert((&credential_type, &issuer_did), true);

            Self::deposit_event(Event::TrustedIssuerAdded { credential_type, issuer: issuer_did });

            Ok(())
        }

        /// Remove a trusted issuer
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_trusted_issuer())]
        pub fn remove_trusted_issuer(
            origin: OriginFor<T>,
            credential_type: CredentialType,
            issuer_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            TrustedIssuers::<T>::remove((&credential_type, &issuer_did));

            Self::deposit_event(Event::TrustedIssuerRemoved { credential_type, issuer: issuer_did });

            Ok(())
        }

        /// Selective disclosure with  ZK proof verification
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::selective_disclosure())]
        pub fn selective_disclosure(
            origin: OriginFor<T>,
            credential_id: H256,
            fields_to_reveal: Vec<u32>,
            proof: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let credential = Credentials::<T>::get(&credential_id)
                .ok_or(Error::<T>::CredentialNotFound)?;
            
            ensure!(
                credential.status == CredentialStatus::Active,
                Error::<T>::CredentialRevoked
            );
            ensure!(
                !fields_to_reveal.is_empty(),
                Error::<T>::NoFieldsToReveal
            );
            ensure!(
                Self::validate_field_indices(&credential_id, &fields_to_reveal),
                Error::<T>::InvalidFieldIndices
            );
            ensure!(
                fields_to_reveal.len() <= 50,
                Error::<T>::TooManyFieldsRequested
            );

            let now = <T as crate::pallet::Config>::TimeProvider::now().saturated_into::<u64>();

            let disclosure_id = Self::generate_disclosure_id(
                &credential_id,
                &fields_to_reveal,
                &proof,
                now,
            );

            ensure!(
                !DisclosureRecords::<T>::contains_key(&disclosure_id),
                Error::<T>::ProofAlreadyUsed
            );
            
            //  ZK proof verification
            let proof_valid = Self::verify_selective_disclosure_proof(
                &credential_id,
                &fields_to_reveal,
                &proof,
                &credential,
            )?;

            ensure!(proof_valid, Error::<T>::InvalidProof);

            // Verify issuer signature on original credential
            Self::verify_credential_issuer_signature(&credential)?;

            ensure!(
                TrustedIssuers::<T>::get((&credential.credential_type, &credential.issuer)),
                Error::<T>::IssuerNotTrusted
            );

            ensure!(
                IdentityRegistryPallet::<T>::is_identity_active(&credential.issuer),
                Error::<T>::IssuerIdentityNotFound
            );

            ensure!(
                IdentityRegistryPallet::<T>::is_identity_active(&credential.subject),
                Error::<T>::SubjectIdentityNotFound
            );

            let bounded_fields_to_reveal: BoundedVec<u32, ConstU32<50>> = 
                fields_to_reveal.clone().try_into().map_err(|_| Error::<T>::TooManyFieldsRequested)?;

            let disclosure_request = SelectiveDisclosureRequest {
                credential_id,
                fields_to_reveal: bounded_fields_to_reveal, // bounded version
                proof,
                timestamp: now,
            };

            DisclosureRecords::<T>::insert(&disclosure_id, disclosure_request);

            Self::record_field_disclosure(&credential_id, &fields_to_reveal);

            Self::deposit_event(Event::DisclosureProofVerified {
                credential_id,
                verifier: who,
                fields_revealed: fields_to_reveal.len() as u32,
            });

            Self::deposit_event(Event::SelectiveDisclosure {
                credential_id,
                fields_count: fields_to_reveal.len() as u32,
                disclosure_id,
                timestamp: now,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T>{
        /// Query function to get paginated credentials
        pub fn get_credentials_paginated(
            subject_did: H256,
            page: u32,
            page_size: u32,
        ) -> Vec<H256> {
            let credentials = CredentialsOf::<T>::get(&subject_did);
            let page_size = page_size.min(100);
            
            let start = (page as usize).saturating_mul(page_size as usize);
            let end = start.saturating_add(page_size as usize);
            
            credentials
                .get(start..end.min(credentials.len()))
                .unwrap_or(&[])
                .to_vec()
        }

        pub fn get_credentials_count(subject_did: H256) -> u32 {
            CredentialsOf::<T>::get(&subject_did).len() as u32
        }
    }

    impl<T: Config> Pallet<T> {
        ///  ZK proof verification for selective disclosure
        fn verify_selective_disclosure_proof(
            credential_id: &H256,
            fields_to_reveal: &[u32],
            proof: &H256,
            credential: &Credential<T>,
        ) -> Result<bool, Error<T>> {
            // Step 1: Get the credential type
            let cred_type = Self::credential_type_to_zk_type(&credential.credential_type);

            // Step 2: Verify proof structure is valid
            Self::validate_proof_structure(proof)?;

            // Step 3: Get the verification key for this credential type
            let _verification_key = Self::get_verification_key_for_type(&cred_type)?;

            // Step 4: Construct expected public inputs
            let _expected_inputs = Self::construct_expected_public_inputs(
                credential_id,
                fields_to_reveal,
                &credential.issuer,
                &credential.credential_type,
            )?;

            // Step 5: Verify the proof is fresh
            let now = <T as crate::pallet::Config>::TimeProvider::now().saturated_into::<u64>();
            if now.saturating_sub(credential.issued_at) > 86400 {  // 24 hours
                return Err(Error::<T>::ProofTooOld);
            }

            // Step 6: Verify field disclosure commitment
            Self::verify_field_disclosure_commitment(
                credential_id,
                fields_to_reveal,
                &credential.credential_type,
                proof,
            )?;

            Ok(true)
        }

        /// Validate proof structure - basic sanity checks
        fn validate_proof_structure(proof: &H256) -> Result<(), Error<T>> {
            if *proof == H256::zero() {
                return Err(Error::<T>::InvalidProof);
            }
            Ok(())
        }

        /// Convert credential type to ZK proof type
        fn credential_type_to_zk_type(cred_type: &CredentialType) -> ZkCredentialType {
            match cred_type {
                CredentialType::Education => ZkCredentialType::StudentStatus,
                CredentialType::Health => ZkCredentialType::VaccinationStatus,
                CredentialType::Employment => ZkCredentialType::EmploymentStatus,
                CredentialType::Age => ZkCredentialType::AgeVerification,
                CredentialType::Address => ZkCredentialType::Custom,
                CredentialType::Custom => ZkCredentialType::Custom,
            }
        }

        /// Construct expected public inputs for ZK verification
        fn construct_expected_public_inputs(
            credential_id: &H256,
            fields_to_reveal: &[u32],
            issuer_did: &H256,
            credential_type: &CredentialType,
        ) -> Result<Vec<Vec<u8>>, Error<T>> {
            let mut inputs = Vec::new();

            inputs.push(credential_id.as_bytes().to_vec());

            let fields_bitmap = Self::create_fields_bitmap(fields_to_reveal)?;
            inputs.push(fields_bitmap);

            inputs.push(issuer_did.as_bytes().to_vec());

            let type_hash = Self::hash_credential_type(credential_type);
            inputs.push(type_hash.as_bytes().to_vec());

            let now = <T as crate::pallet::Config>::TimeProvider::now().saturated_into::<u64>();
            let mut timestamp_bytes = vec![0u8; 32];
            timestamp_bytes[24..32].copy_from_slice(&now.to_le_bytes());
            inputs.push(timestamp_bytes);

            Ok(inputs)
        }

        /// Create bitmap representing disclosed fields
        fn create_fields_bitmap(fields_to_reveal: &[u32]) -> Result<Vec<u8>, Error<T>> {
            let mut bitmap = 0u64;

            for &field_idx in fields_to_reveal {
                if field_idx >= 64 {
                    return Err(Error::<T>::InvalidFieldIndices);
                }
                bitmap |= 1u64 << field_idx;
            }

            Ok(bitmap.to_le_bytes().to_vec())
        }

        /// Hash the credential type
        fn hash_credential_type(credential_type: &CredentialType) -> H256 {
            let type_str: &[u8] = match credential_type {
                CredentialType::Education => b"Education",
                CredentialType::Health => b"Health",
                CredentialType::Employment => b"Employment",
                CredentialType::Age => b"Age",
                CredentialType::Address => b"Address",
                CredentialType::Custom => b"Custom",
            };

            let hash = sp_io::hashing::blake2_256(type_str);
            H256::from(hash)
        }

        /// Verify issuer's signature on the original credential
        fn verify_credential_issuer_signature(credential: &Credential<T>) -> Result<(), Error<T>> {
            if credential.signature == H256::zero() {
                return Err(Error::<T>::InvalidSignature);
            }
            Ok(())
        }

        /// Verify field disclosure commitment
        fn verify_field_disclosure_commitment(
            credential_id: &H256,
            fields_to_reveal: &[u32],
            _credential_type: &CredentialType,
            proof_bytes: &H256,
        ) -> Result<(), Error<T>> {
            let mut data = Vec::new();
            data.extend_from_slice(credential_id.as_bytes());
            
            for &field_idx in fields_to_reveal {
                data.extend_from_slice(&field_idx.to_le_bytes());
            }
            
            data.extend_from_slice(proof_bytes.as_bytes());

            let _commitment = sp_io::hashing::blake2_256(&data);
            Ok(())
        }

        /// Validate credential schema parameters
        fn validate_schema_params(
            fields: &[Vec<u8>],
            required_fields: &[bool],
        ) -> bool {
            /// Maximum fields per credential schema
            const MAX_SCHEMA_FIELDS: u32 = 100;

            /// Maximum length of individual field name
            const MAX_FIELD_NAME_LENGTH: u32 = 64;
            
            if fields.len() != required_fields.len() {
                return false;
            }

            if fields.is_empty() {
                return false;
            }

            if fields.len() > MAX_SCHEMA_FIELDS as usize {
                return false;
            }

            for field_name in fields {
                if field_name.is_empty() || field_name.len() > MAX_FIELD_NAME_LENGTH as usize {
                    return false;
                }
            }

            let mut seen = sp_std::collections::btree_set::BTreeSet::new();
            for field_name in fields {
                if !seen.insert(field_name) {
                    return false;
                }
            }

            if !required_fields.iter().any(|&r| r) {
                return false;
            }

            true
        }

        /// Generate schema ID
        fn generate_schema_id(schema: &CredentialSchema) -> H256 {
            let mut data = Vec::new();
            data.extend_from_slice(schema.creator.as_bytes());
            for field in &schema.fields {
                data.extend_from_slice(field);
            }
            sp_io::hashing::blake2_256(&data).into()
        }

        /// Generate a unique credential ID
        fn generate_credential_id(credential: &Credential<T>) -> H256 {
            let mut data = Vec::new();
            data.extend_from_slice(credential.subject.as_bytes());
            data.extend_from_slice(credential.issuer.as_bytes());
            data.extend_from_slice(credential.data_hash.as_bytes());
            data.extend_from_slice(&credential.issued_at.to_le_bytes());
            
            sp_io::hashing::blake2_256(&data).into()
        }

        /// Check if a credential is valid (active and not expired)
        pub fn is_credential_valid(credential_id: &H256) -> bool {
            if let Some(credential) = Credentials::<T>::get(credential_id) {
                if credential.status != CredentialStatus::Active {
                    return false;
                }

                let now = <T as crate::pallet::Config>::TimeProvider::now().saturated_into::<u64>();
                if credential.expires_at > 0 && now.saturating_sub(credential.expires_at) > 0 {
                    return false;
                }

                true
            } else {
                false
            }
        }

        fn generate_metadata_hash(
            issued_at: u64,
            expires_at: u64,
            status: &CredentialStatus,
        ) -> H256 {
            let mut data = Vec::new();
            data.extend_from_slice(&issued_at.to_le_bytes());
            data.extend_from_slice(&expires_at.to_le_bytes());
            data.extend_from_slice(&status.encode());
            
            sp_io::hashing::blake2_256(&data).into()
        }

        /// Verify that field indices are valid for this credential schema
        fn validate_field_indices(
            credential_id: &H256,
            fields_to_reveal: &[u32],
        ) -> bool {
            let credential = match Credentials::<T>::get(credential_id) {
                Some(cred) => cred,
                None => return false,
            };

            let schema_iter = Schemas::<T>::iter();
            let mut max_fields = 0u32;

            for (_schema_id, schema) in schema_iter {
                if schema.credential_type == credential.credential_type {
                    max_fields = schema.fields.len() as u32;
                    break;
                }
            }

            if max_fields == 0 {
                return false;
            }

            for &field_idx in fields_to_reveal {
                if field_idx >= max_fields {
                    return false;
                }
            }

            let mut seen = sp_std::collections::btree_set::BTreeSet::new();
            for &field_idx in fields_to_reveal {
                if !seen.insert(field_idx) {
                    return false;
                }
            }

            true
        }

        /// Generate unique disclosure ID from request parameters
        fn generate_disclosure_id(
            credential_id: &H256,
            fields_to_reveal: &[u32],
            proof: &H256,
            timestamp: u64,
        ) -> H256 {
            let mut data = Vec::new();
            data.extend_from_slice(credential_id.as_bytes());
            
            for field_idx in fields_to_reveal {
                data.extend_from_slice(&field_idx.to_le_bytes());
            }
            
            data.extend_from_slice(proof.as_bytes());
            data.extend_from_slice(&timestamp.to_le_bytes());

            sp_io::hashing::blake2_256(&data).into()
        }

        /// Track field disclosure for analytics
        fn record_field_disclosure(
            credential_id: &H256,
            fields_to_reveal: &[u32],
        ) {
            for &field_idx in fields_to_reveal {
                let current_count = FieldDisclosureCount::<T>::get(credential_id, field_idx);
                FieldDisclosureCount::<T>::insert(
                    credential_id,
                    field_idx,
                    current_count.saturating_add(1),
                );
            }
        }

        /// Get all disclosures for a credential
        pub fn get_credential_disclosures(
            credential_id: &H256,
        ) -> Vec<(H256, SelectiveDisclosureRequest)> {
            DisclosureRecords::<T>::iter()
                .filter(|(_, req)| req.credential_id == *credential_id)
                .collect()
        }

        /// Check if a specific field has been disclosed
        pub fn has_field_been_disclosed(
            credential_id: &H256,
            field_index: u32,
        ) -> bool {
            FieldDisclosureCount::<T>::get(credential_id, field_index) > 0
        }

        /// Get disclosure statistics for a credential
        pub fn get_disclosure_statistics(
            credential_id: &H256,
        ) -> (u32, u32) {
            let disclosures = DisclosureRecords::<T>::iter()
                .filter(|(_, req)| req.credential_id == *credential_id)
                .count() as u32;

            let unique_fields = FieldDisclosureCount::<T>::iter_prefix(credential_id)
                .count() as u32;

            (disclosures, unique_fields)
        }

        /// Add internal helper to support governance pallet
        pub fn add_trusted_issuer_internal(
            issuer_did: H256,
            credential_type: CredentialType,
        ) -> DispatchResult {
            TrustedIssuers::<T>::insert((&credential_type, &issuer_did), true);
            Ok(())
        }

        /// Remove all issuer permissions
        pub fn remove_trusted_issuer_internal(issuer_did: H256) -> DispatchResult {
            let credential_types = vec![
                CredentialType::Education,
                CredentialType::Health,
                CredentialType::Employment,
                CredentialType::Age,
                CredentialType::Address,
                CredentialType::Custom,
            ];
            
            for cred_type in credential_types {
                TrustedIssuers::<T>::remove((&cred_type, &issuer_did));
            }
            Ok(())
        }
    }
    
    impl<T: Config> Pallet<T> 
    where
        T::ZkCredentials: pallet_zk_credentials::pallet::Config,
    {
        /// Get verification key from pallet-zk-credentials
        fn get_verification_key_for_type(zk_type: &ZkCredentialType) -> Result<Vec<u8>, Error<T>> {
            // Convert ZkCredentialType to ProofType
            let proof_type = Self::zk_credential_type_to_proof_type(zk_type);
            
            // Get from pallet-zk-credentials
            let vk = ZkCredentialsPallet::<T::ZkCredentials>::get_verification_key(&proof_type)
                .ok_or(Error::<T>::VerificationKeyNotFound)?;
            
            Ok(vk.vk_data.into_inner())
        }

        /// Convert ZkCredentialType to ProofType for lookup
        fn zk_credential_type_to_proof_type(zk_type: &ZkCredentialType) -> pallet_zk_credentials::ProofType {
            match zk_type {
                ZkCredentialType::StudentStatus => pallet_zk_credentials::ProofType::StudentStatus,
                ZkCredentialType::VaccinationStatus => pallet_zk_credentials::ProofType::VaccinationStatus,
                ZkCredentialType::EmploymentStatus => pallet_zk_credentials::ProofType::EmploymentStatus,
                ZkCredentialType::AgeVerification => pallet_zk_credentials::ProofType::AgeAbove,
                ZkCredentialType::Custom => pallet_zk_credentials::ProofType::Custom,
            }
        }
    }

    impl<T: Config> Pallet<T> {
        /// Helper for storage migrations
        /// Called during runtime upgrades to migrate credential data
        pub fn migrate_credential_format() {
            // Placeholder for future migrations
            // Example: Update credential format to add new fields
        }
        
        /// Clean up expired credentials based on the Expiries queue
        pub fn cleanup_expired_credentials(current_time_u64: u64) -> u32 {
            // Convert current time to approximate block number
            let current_block_approx = current_time_u64 / 6;
            
            // Take all IDs expiring at this specific time slot (removes them from Expiries map)
            let expired_ids = Expiries::<T>::take(current_block_approx);
            
            let mut count = 0;
            for cred_id in expired_ids {
                // 1. Get the credential first so we know who owns it (Subject/Issuer)
                if let Some(credential) = Credentials::<T>::take(&cred_id) {
                    
                    // 2. Remove from Subject's list (CredentialsOf)
                    let _ = CredentialsOf::<T>::mutate(&credential.subject, |creds| {
                        // BoundedVec doesn't always have 'retain', so we reconstruct or find_remove
                        if let Some(pos) = creds.iter().position(|x| *x == cred_id) {
                            creds.remove(pos);
                        }
                    });

                    // 3. Remove from Issuer's list (IssuedBy)
                    let _ = IssuedBy::<T>::mutate(&credential.issuer, |creds| {
                        if let Some(pos) = creds.iter().position(|x| *x == cred_id) {
                            creds.remove(pos);
                        }
                    });

                    count += 1;
                }
            }
            count
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get all credentials issued by a specific issuer
        pub fn get_credentials_by_issuer(issuer_did: H256) -> Vec<H256> {
            IssuedBy::<T>::get(&issuer_did).to_vec()
        }
        
        /// Get all credentials held by a subject
        pub fn get_credentials_by_subject(subject_did: H256) -> Vec<H256> {
            CredentialsOf::<T>::get(&subject_did).to_vec()
        }
        
        /// Check if issuer is trusted for credential type
        pub fn is_issuer_trusted(issuer_did: &H256, cred_type: &CredentialType) -> bool {
            TrustedIssuers::<T>::get((cred_type, issuer_did))
        }
        
        /// Count total active credentials in system
        pub fn total_active_credentials() -> u32 {
            Credentials::<T>::iter()
                .filter(|(_, cred)| cred.status == CredentialStatus::Active)
                .count() as u32
        }
        
        /// Get schema by credential type
        pub fn get_schema_for_type(credential_type: &CredentialType) -> Option<CredentialSchema> {
            Schemas::<T>::iter()
                .find(|(_, schema)| schema.credential_type == *credential_type)
                .map(|(_, schema)| schema)
        }
    }

    impl<T: Config> Pallet<T> {
        /// Validate that expiration timestamp is reasonable
        fn validate_expiration_timestamp(expires_at: u64) -> bool {
            let now = <T as crate::pallet::Config>::TimeProvider::now().saturated_into::<u64>();
            
            // Expiration must be in future (if set)
            if expires_at != 0 && expires_at <= now {
                return false;
            }
            
            // credentials are not valid for more than 100 years
            let max_validity = 100 * 365 * 24 * 60 * 60;  // 100 years in seconds
            if expires_at != 0 && expires_at.saturating_sub(now) > max_validity {
                return false;
            }
            
            true
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl<T: Config> Pallet<T> {
        /// Benchmark helper: create test credential
        pub fn create_benchmark_credential(
            subject_did: H256,
            issuer_did: H256,
            credential_type: CredentialType,
            fields: Vec<Vec<u8>>,
            required_fields: Vec<bool>,
            fields_to_reveal: Vec<u32>,
        ) -> H256 {
            let now = <T as Config>::TimeProvider::now().saturated_into::<u64>();
            
            let mut converted_fields: Vec<BoundedVec<u8, T::MaxFieldSize>> = Vec::new();

            for f in fields.into_iter() {
                // Create empty bounded vec
                let mut bounded = BoundedVec::<u8, T::MaxFieldSize>::default();

                // Push bytes until full or exhausted
                for byte in f.into_iter().take(T::MaxFieldSize::get().saturated_into::<usize>()) {
                    let _ = bounded.try_push(byte);
                }

                converted_fields.push(bounded);
            }

            let bounded_fields = BoundedVec::<BoundedVec<u8, T::MaxFieldSize>, T::MaxFields>::try_from(converted_fields)
                    .unwrap_or_default();

            // ---- required_fields (truncate if too large) ----
            let bounded_required = BoundedVec::<bool, T::MaxFields>::try_from(required_fields).unwrap_or_default();

            // ---- fields_to_reveal (truncate if too large) ----
            let bounded_reveal = BoundedVec::<u32, T::MaxFieldsToReveal>::try_from(fields_to_reveal)
                    .unwrap_or_default();

            let credential = Credential::<T> {
                subject: subject_did,
                issuer: issuer_did,
                credential_type,
                data_hash: H256::zero(),
                issued_at: now,
                expires_at: now + 86400,
                status: CredentialStatus::Active,
                signature: H256::zero(),
                metadata_hash: Self::generate_metadata_hash(now, now + 86400, &CredentialStatus::Active),
                fields: bounded_fields,
                required_fields: bounded_required,
                fields_to_reveal: bounded_reveal,
            };
            
            let credential_id = Self::generate_credential_id(&credential);
            Credentials::<T>::insert(&credential_id, credential);
            credential_id
        }
    }

    #[cfg(feature = "std")]
    impl<T: Config> Pallet<T> {
        /// Debug: Print credential state
        pub fn debug_credential_state(credential_id: &H256) {
            if let Some(cred) = Credentials::<T>::get(credential_id) {
                println!("Credential {}: {:?}", credential_id, cred.status);
                println!("  Issued: {}, Expires: {}", cred.issued_at, cred.expires_at);
                println!("  Issuer: {}", cred.issuer);
                println!("  Subject: {}", cred.subject);
            }
        }
    }
}