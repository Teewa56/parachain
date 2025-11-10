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

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type TimeProvider: Time;
    }

    /// Identity information for a DID
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Identity<T: Config> {
        /// The account that controls this identity
        pub controller: T::AccountId,
        /// Public key for verification
        pub public_key: H256,
        /// Timestamp when identity was created
        pub created_at: u64,
        /// Timestamp when identity was last updated
        pub updated_at: u64,
        /// Whether the identity is active
        pub active: bool,
    }

    /// DID Document structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct DidDocument {
        /// DID identifier
        pub did: Vec<u8>,
        /// Public keys associated with the DID
        pub public_keys: Vec<H256>,
        /// Authentication methods
        pub authentication: Vec<H256>,
        /// Service endpoints
        pub services: Vec<Vec<u8>>,
    }

    /// Storage: Maps DID hash to Identity
    #[pallet::storage]
    #[pallet::getter(fn identities)]
    pub type Identities<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        H256, 
        Identity<T>, 
        OptionQuery
    >;

    /// Storage: Maps AccountId to their DID hash
    #[pallet::storage]
    #[pallet::getter(fn account_dids)]
    pub type AccountDids<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        T::AccountId, 
        H256, 
        OptionQuery
    >;

    /// Storage: DID Documents
    #[pallet::storage]
    #[pallet::getter(fn did_documents)]
    pub type DidDocuments<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        H256, 
        DidDocument, 
        OptionQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Identity created [did_hash, controller]
        IdentityCreated { did_hash: H256, controller: T::AccountId },
        /// Identity updated [did_hash, controller]
        IdentityUpdated { did_hash: H256, controller: T::AccountId },
        /// Identity deactivated [did_hash]
        IdentityDeactivated { did_hash: H256 },
        /// Identity reactivated [did_hash]
        IdentityReactivated { did_hash: H256 },
        /// DID document updated [did_hash]
        DidDocumentUpdated { did_hash: H256 },
    }

    #[pallet::error]
    pub enum Error<T> {
        IdentityAlreadyExists,
        IdentityNotFound,
        NotController,
        IdentityInactive,
        AccountAlreadyHasIdentity,
        DidDocumentNotFound,
        InvalidDidFormat,
        InvalidPublicKey,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new identity
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_identity(
            origin: OriginFor<T>,
            did: Vec<u8>,
            public_key: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Self::validate_did_format(&did),
                Error::<T>::InvalidDidFormat 
            );

            ensure!(
                Self::validate_public_key(&public_key),
                Error::<T>::InvalidPublicKey 
            );

            ensure!(
                !AccountDids::<T>::contains_key(&who),
                Error::<T>::AccountAlreadyHasIdentity
            );

            let did_hash = Self::hash_did(&did);

            ensure!(
                !Identities::<T>::contains_key(&did_hash),
                Error::<T>::IdentityAlreadyExists
            );

            let now = T::TimeProvider::now().as_secs();

            let identity = Identity {
                controller: who.clone(),
                public_key,
                created_at: now,
                updated_at: now,
                active: true,
            };

            let did_document = DidDocument {
                did: did.clone(),
                public_keys: sp_std::vec![public_key],
                authentication: sp_std::vec![public_key],
                services: sp_std::vec![],
            };

            Identities::<T>::insert(&did_hash, identity);
            AccountDids::<T>::insert(&who, did_hash);
            DidDocuments::<T>::insert(&did_hash, did_document);

            Self::deposit_event(Event::IdentityCreated { did_hash, controller: who });

            Ok(())
        }

        /// Update identity public key
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn update_identity(
            origin: OriginFor<T>,
            new_public_key: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let did_hash = AccountDids::<T>::get(&who)
                .ok_or(Error::<T>::IdentityNotFound)?;

            Identities::<T>::try_mutate(&did_hash, |identity_opt| -> DispatchResult {
                let identity = identity_opt.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                
                ensure!(identity.controller == who, Error::<T>::NotController);
                ensure!(identity.active, Error::<T>::IdentityInactive);

                identity.public_key = new_public_key;
                identity.updated_at = T::TimeProvider::now().as_secs();

                Self::deposit_event(Event::IdentityUpdated { did_hash, controller: who });

                Ok(())
            })
        }

        /// Deactivate identity
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn deactivate_identity(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let did_hash = AccountDids::<T>::get(&who)
                .ok_or(Error::<T>::IdentityNotFound)?;

            Identities::<T>::try_mutate(&did_hash, |identity_opt| -> DispatchResult {
                let identity = identity_opt.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                
                ensure!(identity.controller == who, Error::<T>::NotController);

                identity.active = false;
                identity.updated_at = T::TimeProvider::now().as_secs();

                Self::deposit_event(Event::IdentityDeactivated { did_hash });

                Ok(())
            })
        }

        /// Reactivate identity
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn reactivate_identity(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let did_hash = AccountDids::<T>::get(&who)
                .ok_or(Error::<T>::IdentityNotFound)?;

            Identities::<T>::try_mutate(&did_hash, |identity_opt| -> DispatchResult {
                let identity = identity_opt.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                
                ensure!(identity.controller == who, Error::<T>::NotController);

                identity.active = true;
                identity.updated_at = T::TimeProvider::now().as_secs();

                Self::deposit_event(Event::IdentityReactivated { did_hash });

                Ok(())
            })
        }

        /// Update DID document
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn update_did_document(
            origin: OriginFor<T>,
            public_keys: Vec<H256>,
            authentication: Vec<H256>,
            services: Vec<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let did_hash = AccountDids::<T>::get(&who)
                .ok_or(Error::<T>::IdentityNotFound)?;

            let identity = Identities::<T>::get(&did_hash)
                .ok_or(Error::<T>::IdentityNotFound)?;

            ensure!(identity.controller == who, Error::<T>::NotController);
            ensure!(identity.active, Error::<T>::IdentityInactive);

            DidDocuments::<T>::try_mutate(&did_hash, |doc_opt| -> DispatchResult {
                let doc = doc_opt.as_mut().ok_or(Error::<T>::DidDocumentNotFound)?;

                doc.public_keys = public_keys;
                doc.authentication = authentication;
                doc.services = services;

                Self::deposit_event(Event::DidDocumentUpdated { did_hash });

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Hash a DID to get its identifier
        pub fn hash_did(did: &[u8]) -> H256 {
            sp_io::hashing::blake2_256(did).into()
        }

        /// Verify if an identity exists and is active
        pub fn is_identity_active(did_hash: &H256) -> bool {
            if let Some(identity) = Identities::<T>::get(did_hash) {
                identity.active
            } else {
                false
            }
        }

        /// Get identity by account
        pub fn get_identity_by_account(account: &T::AccountId) -> Option<(H256, Identity<T>)> {
            if let Some(did_hash) = AccountDids::<T>::get(account) {
                if let Some(identity) = Identities::<T>::get(&did_hash) {
                    return Some((did_hash, identity));
                }
            }
            None
        }
    }

    impl<T: Config> Pallet<T> {
        /// Validate DID format according to W3C spec
        /// DID format: did:<method>:<method-specific-id>
        fn validate_did_format(did: &[u8]) -> bool {
            // Minimum length: "did:x:y" = 7 bytes
            if did.len() < 7 || did.len() > 255 {
                return false;
            }

            // Must start with "did:"
            if !did.starts_with(b"did:") {
                return false;
            }

            // Check valid characters (alphanumeric, dash, underscore only)
            for byte in did {
                match byte {
                    b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b':' => {},
                    _ => return false,
                }
            }

            // Must have at least one colon after "did:"
            if did.iter().skip(4).filter(|&&b| b == b':').count() == 0 {
                return false;
            }

            true
        }

        /// Validate public key is valid
        fn validate_public_key(public_key: &H256) -> bool {
            // Public key cannot be all zeros
            *public_key != H256::zero()
        }
    }
}