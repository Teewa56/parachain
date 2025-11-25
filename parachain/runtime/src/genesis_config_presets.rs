use crate::{
	AccountId, BalancesConfig, CollatorSelectionConfig, ParachainInfoConfig, PolkadotXcmConfig,
	RuntimeGenesisConfig, SessionConfig, SessionKeys, SudoConfig, EXISTENTIAL_DEPOSIT,
};

use alloc::{vec, vec::Vec};

use polkadot_sdk::{staging_xcm as xcm, *};

use cumulus_primitives_core::ParaId;
use sp_core::{H256, Pair, Public};
use serde_json::Value;
use sp_genesis_builder::{PresetId, DEV_RUNTIME_PRESET, LOCAL_TESTNET_RUNTIME_PRESET};
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::{IdentifyAccount, Verify};
use serde_json::Value;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
/// Parachain id used for genesis config presets of parachain template.
#[docify::export_content]
pub const PARACHAIN_ID: u32 = 1000;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}

fn get_did_hash(did: &str) -> H256 {
    sp_io::hashing::blake2_256(did.as_bytes()).into()
}

fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountSignature: Verify<Signer = TPublic>,
    TPublic::Pair: Pair,
    AccountId: From<TPublic::From>,
    AccountId: IdentifyAccount<AccountId = AccountId>,
{
    AccountSignature::from(TPublic::Pair::from_string(&format!("//{}", seed), None).expect("static values are valid; qed").public()).into_account()
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	root: AccountId,
	id: ParaId,
) -> Value {
	serde_json::json!({
        "balances": {
            "balances": endowed_accounts
                .iter()
                .map(|k| (k.clone(), 1u128 << 60))
                .collect::<Vec<_>>()
        },
        "parachainInfo": {
            "parachainId": id
        },
        "collatorSelection": {
            "invulnerables": invulnerables.iter().map(|(acc, _)| acc.clone()).collect::<Vec<_>>(),
            "candidacyBond": EXISTENTIAL_DEPOSIT * 16
        },
        "session": {
            "keys": invulnerables
                .into_iter()
                .map(|(acc, aura)| (acc.clone(), acc, template_session_keys(aura)))
                .collect::<Vec<_>>()
        },
        "polkadotXcm": {
            "safeXcmVersion": SAFE_XCM_VERSION
        },
        "sudo": {
            "key": root
        }
    })
}

fn local_testnet_genesis() -> Value {
	testnet_genesis(
		// initial collators.
		vec![
			(Sr25519Keyring::Alice.to_account_id(), Sr25519Keyring::Alice.public().into()),
			(Sr25519Keyring::Bob.to_account_id(), Sr25519Keyring::Bob.public().into()),
		],
		Sr25519Keyring::well_known().map(|k| k.to_account_id()).collect(),
		Sr25519Keyring::Alice.to_account_id(),
		PARACHAIN_ID.into(),
	)
}

fn development_config_genesis() -> Value {
    let alice_did = get_did_hash("did:src:alice");
    let bob_did = get_did_hash("did:src:bob");
    
    let alice = Sr25519Keyring::Alice.to_account_id();
    let root_key = alice.clone();
    let mut invulnerables = vec![
        (Sr25519Keyring::Alice.to_account_id(), Sr25519Keyring::Alice.public().into()),
        (Sr25519Keyring::Bob.to_account_id(), Sr25519Keyring::Bob.public().into()),
    ];
    
    let endowed_accounts = vec![
        Sr25519Keyring::Alice.to_account_id(),
        Sr25519Keyring::Bob.to_account_id(),
        Sr25519Keyring::Charlie.to_account_id(),
        Sr25519Keyring::Dave.to_account_id(),
        Sr25519Keyring::Eve.to_account_id(),
        Sr25519Keyring::Ferdie.to_account_id(),
    ];

    serde_json::json!({
        "system": {},
        "balances": {
            "balances": endowed_accounts.into_iter().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
        },
        "parachainInfo": {
            "parachainId": 1000
        },
        "collatorSelection": {
            "invulnerables": invulnerables.iter().map(|(acc, _)| acc).collect::<Vec<_>>(),
            "candidacyBond": 1u64 << 57,
        },
        "session": {
            "keys": invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),
                        acc,
                        SessionKeys { aura },
                    )
                })
                .collect::<Vec<_>>(),
        },
        "sudo": {
            "key": root_key,
        },
        "polkadotXcm": {
            "safeXcmVersion": Some(3)
        },
        "verifiableCredentials": {
            "trustedIssuers": [
                [alice_did, "Education"],
                [bob_did, "Health"]
            ]
        },
        "xcmCredentials": {
            "registeredParachains": [
                [2000, true],
                [3000, true]
            ]
        },
        "proofOfPersonhood": {}
    })
}

/// Provides the JSON preset based on the requested name
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    let patch = match id.as_ref() {
        DEV_RUNTIME_PRESET => development_config_genesis(),
        LOCAL_TESTNET_RUNTIME_PRESET => local_testnet_genesis(),
        _ => return None,
    };

    Some(serde_json::to_string(&patch).unwrap().into_bytes())
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
	vec![
		PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
		PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
	]
}
