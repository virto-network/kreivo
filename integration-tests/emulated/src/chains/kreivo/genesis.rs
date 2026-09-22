use emulated_integration_tests_common::{
	accounts, build_genesis_storage, collators, ASSETS_PALLET_ID, ASSET_HUB_WESTEND_ID, SAFE_XCM_VERSION, USDT_ID,
};
use kreivo_runtime::{
	AssetsConfig, BalancesConfig, CollatorSelectionConfig, FungibleAssetLocation, ParachainInfoConfig,
	PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys, EXISTENTIAL_DEPOSIT,
};
use parachains_common::{AccountId, Balance};
use sp_core::storage::Storage;
use sp_keyring::Sr25519Keyring as Keyring;
use virto_common::Para;

pub const PARA_ID: u32 = 2281;
pub const ED: Balance = EXISTENTIAL_DEPOSIT;
pub const USDT_MIN_BALANCE: Balance = 1_000;

/// Asset Hub's USDT, as registered on Kreivo.
pub fn usdt() -> FungibleAssetLocation {
	FungibleAssetLocation::Sibling(Para {
		id: ASSET_HUB_WESTEND_ID as u16,
		pallet: ASSETS_PALLET_ID,
		index: USDT_ID,
	})
}

pub fn asset_owner() -> AccountId {
	Keyring::Alice.to_account_id()
}

pub fn genesis() -> Storage {
	let genesis_config = RuntimeGenesisConfig {
		balances: BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096 * 4096))
				.collect(),
			..Default::default()
		},
		parachain_info: ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		collator_selection: CollatorSelectionConfig {
			invulnerables: collators::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| (acc.clone(), acc, SessionKeys { aura }))
				.collect(),
			..Default::default()
		},
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		assets: AssetsConfig {
			assets: vec![(usdt(), asset_owner(), true, USDT_MIN_BALANCE)],
			metadata: vec![(usdt(), b"Tether USD".to_vec(), b"USDT".to_vec(), 6)],
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}
