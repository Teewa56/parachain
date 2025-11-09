#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::Time,
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_core::H256;
    use pallet_identity_registry;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_identity_registry::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type TimeProvider: Time;
    }

    /// Credential types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum CredentialType {
        Education,      // Student ID, Degree, etc.
        Health,         // Vaccination, Medical records
        Employment,     // Work history, certifications
        Age,            // Age verification
        Address,        // Proof of residence
        Custom,         // Custom credential types
    }

    /// Credential status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum CredentialStatus {
        Active,
        Revoked,
        Expired,
        Suspended,
    }

    /// Verifiable Credential structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Credential<T: Config> {
        /// Subject's DID hash
        pub subject: H256,
        /// Issuer's DID hash
        pub issuer: H256,
        /// Credential type
        pub credential_type: CredentialType,
        /// Hash of the credential data (for privacy)
        pub data_hash: H256,
        /// Issuance timestamp
        pub issued_at: u64,
        /// Expiration timestamp (0 means no expiration)
        pub expires_at: u64,
        /// Current status
        pub status: CredentialStatus,
        /// Issuer's signature
        pub signature: H256,
    }

    /// Credential schema for defining what fields a credential type should have
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct CredentialSchema {
        /// Schema ID
        pub schema_id: H256,
        /// Credential type this schema defines
        pub credential_type: CredentialType,
        /// Field names (as bytes)
        pub fields: Vec<Vec<u8>>,
        /// Whether fields are required
        pub required_fields: Vec<bool>,
        /// Creator of this schema
        pub creator: H256,
    }

    /// Selective disclosure request
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct DisclosureRequest {
        /// Which credential to disclose
        pub credential_id: H256,
        /// Which fields to reveal (field indices)
        pub fields_to_reveal: Vec<u32>,
        /// Proof of ownership
        pub proof: H256,
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

    /// Storage: Credentials owned by a DID
    #[pallet::storage]
    #[pallet::getter(fn credentials_of)]
    pub type CredentialsOf<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        H256, 
        BoundedVec<H256, ConstU32<100>>, 
        ValueQuery
    >;

    /// Storage: Credentials issued by a DID
    #[pallet::storage]
    #[pallet::getter(fn issued_by)]
    pub type IssuedBy<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        H256, 
        BoundedVec<H256, ConstU32<1000>>, 
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
    pub type TrustedIssuers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CredentialType,
        Blake2_128Concat,
        H256,
        bool,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Credential issued [credential_id, subject, issuer]
        CredentialIssued { 
            credential_id: H256, 
            subject: H256, 
            issuer: H256, 
            credential_type: CredentialType 
        },
        /// Credential revoked [credential_id, issuer]
        CredentialRevoked { credential_id: H256, issuer: H256 },
        /// Credential verified [credential_id, verifier]
        CredentialVerified { credential_id: H256, verifier: T::AccountId },
        /// Schema created [schema_id, creator]
        SchemaCreated { schema_id: H256, creator: H256 },
        /// Trusted issuer added [credential_type, issuer_did]
        TrustedIssuerAdded { credential_type: CredentialType, issuer: H256 },
        /// Trusted issuer removed [credential_type, issuer_did]
        TrustedIssuerRemoved { credential_type: CredentialType, issuer: H256 },
        /// Selective disclosure made [credential_id, fields_revealed]
        SelectiveDisclosure { credential_id: H256, fields_count: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Credential not found
        CredentialNotFound,
        /// Not authorized to perform this action
        NotAuthorized,
        /// Credential already exists
        CredentialAlreadyExists,
        /// Credential is revoked
        CredentialRevoked,
        /// Credential is expired
        CredentialExpired,
        /// Invalid signature
        InvalidSignature,
        /// Subject identity not found
        SubjectIdentityNotFound,
        /// Issuer identity not found
        IssuerIdentityNotFound,
        /// Issuer not trusted for this credential type
        IssuerNotTrusted,
        /// Schema not found
        SchemaNotFound,
        /// Too many credentials
        TooManyCredentials,
        /// Invalid credential status
        InvalidCredentialStatus,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Issue a new credential
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn issue_credential(
            origin: OriginFor<T>,
            subject_did: H256,
            credential_type: CredentialType,
            data_hash: H256,
            expires_at: u64,
            signature: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get issuer's DID
            let (issuer_did, _) = pallet_identity_registry::Pallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::IssuerIdentityNotFound)?;

            // Verify subject identity exists
            ensure!(
                pallet_identity_registry::Pallet::<T>::is_identity_active(&subject_did),
                Error::<T>::SubjectIdentityNotFound
            );

            // Check if issuer is trusted for this credential type
            ensure!(
                TrustedIssuers::<T>::get(&credential_type, &issuer_did),
                Error::<T>::IssuerNotTrusted
            );

            let now = T::TimeProvider::now().as_secs();

            // Create credential
            let credential = Credential {
                subject: subject_did,
                issuer: issuer_did,
                credential_type: credential_type.clone(),
                data_hash,
                issued_at: now,
                expires_at,
                status: CredentialStatus::Active,
                signature,
            };

            // Generate credential ID
            let credential_id = Self::generate_credential_id(&credential);

            // Store credential
            Credentials::<T>::insert(&credential_id, credential);

            // Add to subject's credentials
            CredentialsOf::<T>::try_mutate(&subject_did, |creds| -> DispatchResult {
                creds.try_push(credential_id)
                    .map_err(|_| Error::<T>::TooManyCredentials)?;
                Ok(())
            })?;

            // Add to issuer's issued credentials
            IssuedBy::<T>::try_mutate(&issuer_did, |creds| -> DispatchResult {
                creds.try_push(credential_id)
                    .map_err(|_| Error::<T>::TooManyCredentials)?;
                Ok(())
            })?;

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
        #[pallet::weight(10_000)]
        pub fn revoke_credential(
            origin: OriginFor<T>,
            credential_id: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (issuer_did, _) = pallet_identity_registry::Pallet::<T>::get_identity_by_account(&who)
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
        #[pallet::weight(10_000)]
        pub fn verify_credential(
            origin: OriginFor<T>,
            credential_id: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let credential = Credentials::<T>::get(&credential_id)
                .ok_or(Error::<T>::CredentialNotFound)?;

            // Check if revoked
            ensure!(credential.status == CredentialStatus::Active, Error::<T>::CredentialRevoked);

            // Check if expired
            let now = T::TimeProvider::now().as_secs();
            if credential.expires_at > 0 && now > credential.expires_at {
                return Err(Error::<T>::CredentialExpired.into());
            }

            // Verify issuer and subject identities are still active
            ensure!(
                pallet_identity_registry::Pallet::<T>::is_identity_active(&credential.issuer),
                Error::<T>::IssuerIdentityNotFound
            );
            ensure!(
                pallet_identity_registry::Pallet::<T>::is_identity_active(&credential.subject),
                Error::<T>::SubjectIdentityNotFound
            );

            Self::deposit_event(Event::CredentialVerified { credential_id, verifier: who });

            Ok(())
        }

        /// Create a credential schema
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn create_schema(
            origin: OriginFor<T>,
            credential_type: CredentialType,
            fields: Vec<Vec<u8>>,
            required_fields: Vec<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (creator_did, _) = pallet_identity_registry::Pallet::<T>::get_identity_by_account(&who)
                .ok_or(Error::<T>::IssuerIdentityNotFound)?;

            let schema = CredentialSchema {
                schema_id: H256::zero(), // Will be set below
                credential_type,
                fields,
                required_fields,
                creator: creator_did,
            };

            let schema_id = Self::generate_schema_id(&schema);
            let mut schema_with_id = schema;
            schema_with_id.schema_id = schema_id;

            Schemas::<T>::insert(&schema_id, schema_with_id);

            Self::deposit_event(Event::SchemaCreated { schema_id, creator: creator_did });

            Ok(())
        }

        /// Add a trusted issuer for a credential type (requires root/governance)
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn add_trusted_issuer(
            origin: OriginFor<T>,
            credential_type: CredentialType,
            issuer_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                pallet_identity_registry::Pallet::<T>::is_identity_active(&issuer_did),
                Error::<T>::IssuerIdentityNotFound
            );

            TrustedIssuers::<T>::insert(&credential_type, &issuer_did, true);

            Self::deposit_event(Event::TrustedIssuerAdded { credential_type, issuer: issuer_did });

            Ok(())
        }

        /// Remove a trusted issuer (requires root/governance)
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn remove_trusted_issuer(
            origin: OriginFor<T>,
            credential_type: CredentialType,
            issuer_did: H256,
        ) -> DispatchResult {
            ensure_root(origin)?;

            TrustedIssuers::<T>::remove(&credential_type, &issuer_did);

            Self::deposit_event(Event::TrustedIssuerRemoved { credential_type, issuer: issuer_did });

            Ok(())
        }

        /// Request selective disclosure (ZK proof placeholder)
        #[pallet::call_index(6)]
        #[pallet::weight(10_000)]
        pub fn selective_disclosure(
            origin: OriginFor<T>,
            credential_id: H256,
            fields_to_reveal: Vec<u32>,
            proof: H256,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let credential = Credentials::<T>::get(&credential_id)
                .ok_or(Error::<T>::CredentialNotFound)?;

            ensure!(credential.status == CredentialStatus::Active, Error::<T>::CredentialRevoked);

            // TODO: Verify ZK proof here
            // For now, we just emit an event

            Self::deposit_event(Event::SelectiveDisclosure { 
                credential_id, 
                fields_count: fields_to_reveal.len() as u32 
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Generate a unique credential ID
        fn generate_credential_id(credential: &Credential<T>) -> H256 {
            let mut data = Vec::new();
            data.extend_from_slice(credential.subject.as_bytes());
            data.extend_from_slice(credential.issuer.as_bytes());
            data.extend_from_slice(credential.data_hash.as_bytes());
            data.extend_from_slice(&credential.issued_at.to_le_bytes());
            
            sp_io::hashing::blake2_256(&data).into()
        }

        /// Generate a schema ID
        fn generate_schema_id(schema: &CredentialSchema) -> H256 {
            let mut data = Vec::new();
            data.extend_from_slice(schema.creator.as_bytes());
            for field in &schema.fields {
                data.extend_from_slice(field);
            }
            
            sp_io::hashing::blake2_256(&data).into()
        }

        /// Check if a credential is valid (active and not expired)
        pub fn is_credential_valid(credential_id: &H256) -> bool {
            if let Some(credential) = Credentials::<T>::get(credential_id) {
                if credential.status != CredentialStatus::Active {
                    return false;
                }

                let now = T::TimeProvider::now().as_secs();
                if credential.expires_at > 0 && now > credential.expires_at {
                    return false;
                }

                true
            } else {
                false
            }
        }
    }
}