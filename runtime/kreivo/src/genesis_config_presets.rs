use crate::*;
use runtime_constants::genesis_presets::*;
use sp_genesis_builder::PresetId;

fn local_genesis(
	sudo: AccountId,
	id: ParaId,
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
) -> serde_json::Value {
	serde_json::json!({
		"communitiesManager": CommunitiesManagerConfig {
			// A community to cover a sudo-ish management.
			communities: vec![
				(1, String::from("root"), sudo, None, Some(1)),
			],
			memberships: vec![
				(1, 10, UNITS, (None, None), None)
			]
		},
		"balances": BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, EXISTENTIAL_DEPOSIT * 4096 * 4096))
				.collect(),
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

pub fn local_testnet_genesis(para_id: ParaId) -> serde_json::Value {
	local_genesis(alice(), para_id, invulnerables(), testnet_accounts())
}

pub fn preset_names() -> Vec<PresetId> {
	vec![PresetId::from("development"), PresetId::from("local")]
}

pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.try_into() {
		Ok("development") => local_testnet_genesis(2281.into()),
		Ok("local") => local_testnet_genesis(2281.into()),
		_ => return None,
	};

	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work; qed")
			.into_bytes(),
	)
}
