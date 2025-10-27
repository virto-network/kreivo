use crate::*;
use runtime_constants::genesis_presets::*;
use sp_genesis_builder::PresetId;

mod dev {
	use super::*;

	pub fn genesis(
		id: ParaId,
		invulnerables: Vec<(AccountId, AuraId)>,
		endowed_accounts: Vec<AccountId>,
	) -> serde_json::Value {
		serde_json::json!({
			"balances": BalancesConfig {
				balances: endowed_accounts
					.iter()
					.cloned()
					.map(|k| (k, EXISTENTIAL_DEPOSIT * 4096 * 4096))
					.collect(),
				dev_accounts: None,
			},
			"parachainInfo": ParachainInfoConfig {
				parachain_id: id,
				..Default::default()
			},
			"collatorSelection": CollatorSelectionConfig {
				invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
				..Default::default()
			},
			"session": SessionConfig {
				keys: invulnerables
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),          // account id
							acc,                  // validator id
							SessionKeys { aura }, // session keys
						)
					})
					.collect(),
				..Default::default()
			},
			"polkadotXcm": {
				"safeXcmVersion": Some(SAFE_XCM_VERSION),
			},
		})
	}
}

mod local {
	use super::*;

	pub fn genesis(
		sudo: AccountId,
		id: ParaId,
		invulnerables: Vec<(AccountId, AuraId)>,
		endowed_accounts: Vec<AccountId>,
	) -> serde_json::Value {
		let mut genesis = dev::genesis(id, invulnerables, endowed_accounts)
			.as_object()
			.cloned()
			.expect("genesis is a json object");

		let mut community_manager = serde_json::json!({
			"communitiesManager": CommunitiesManagerConfig {
				// A community to cover a sudo-ish management.
				communities: vec![(1, String::from("root"), sudo, None, Some(1))],
				memberships: vec![(1, 10, UNITS, (None, None), None)],
			},
		});

		genesis.append(
			community_manager
				.as_object_mut()
				.expect("communities manager is a json object"),
		);

		let mut initial_assets = serde_json::json!(
			{
				"assets": AssetsConfig {
				assets: vec![
					(
						virto_common::FungibleAssetLocation::Here(1),
						alice(),
						true,
						1,
					),
				],
				metadata: vec![
					(
						virto_common::FungibleAssetLocation::Here(1),
						Vec::from("Asset 1"),
						Vec::from("ASSET1"),
						18,
					)
				],
				accounts: vec![(virto_common::FungibleAssetLocation::Here(1), alice(), 123456789)],
				next_asset_id: None
			}
			}
		);

		genesis.append(
			initial_assets
				.as_object_mut()
				.expect("assets is a json object"),
		);

		serde_json::Value::Object(genesis.clone())
	}
}

pub fn local_testnet_genesis(para_id: ParaId) -> serde_json::Value {
	local::genesis(alice(), para_id, invulnerables(), testnet_accounts())
}

pub fn dev_genesis(para_id: ParaId) -> serde_json::Value {
	dev::genesis(para_id, invulnerables(), testnet_accounts())
}

pub fn preset_names() -> Vec<PresetId> {
	vec![PresetId::from("development"), PresetId::from("local")]
}

pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_ref() {
		sp_genesis_builder::DEV_RUNTIME_PRESET => dev_genesis(2281.into()),
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_testnet_genesis(2281.into()),
		_ => return None,
	};

	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work; qed")
			.into_bytes(),
	)
}
