use super::*;

#[cfg(not(feature = "paseo"))]
const RELAY_CHAIN_STRING: &str = "kusama-local";

#[cfg(not(feature = "paseo"))]
const CHAIN_ID_STRING: &str = "kreivo_kusama_local";
#[cfg(not(feature = "paseo"))]
const CHAIN_NAME_STRING: &str = "Kreivo-Kusama Local";

#[cfg(not(feature = "paseo"))]
pub fn properties() -> sc_chain_spec::Properties {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();

	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 2.into());

	properties
}

#[cfg(feature = "paseo")]
const RELAY_CHAIN_STRING: &str = "paseo-local";

#[cfg(feature = "paseo")]
const CHAIN_ID_STRING: &str = "kreivo_paseo_local";
#[cfg(feature = "paseo")]
const CHAIN_NAME_STRING: &str = "Kreivo de Paseo-Local";

#[cfg(feature = "paseo")]
pub fn properties() -> sc_chain_spec::Properties {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();

	properties.insert("tokenSymbol".into(), "PAS".into());
	properties.insert("tokenDecimals".into(), 10.into());
	properties.insert("ss58Format".into(), 1.into());

	properties
}

pub fn chain_spec() -> Result<Box<dyn ChainSpec>, String> {
	Ok(Box::new(
		KreivoChainSpec::builder(
			kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
			Extensions {
				relay_chain: RELAY_CHAIN_STRING.into(),
				// You MUST set this to the correct network!
				para_id: KREIVO_PARA_ID,
			},
		)
		.with_id(CHAIN_ID_STRING)
		.with_name(CHAIN_NAME_STRING)
		.with_chain_type(ChainType::Local)
		.with_properties(properties())
		.with_genesis_config_preset_name("local_testnet")
		.build(),
	))
}
