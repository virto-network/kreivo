// Based on pallet-asset-precompile

use pallet_assets_precompiles::{AssetIdExtractor, AssetPrecompileConfig};
use pallet_payments::Decode;
use pallet_revive::precompiles::{alloy::sol_types::Revert, AddressMatcher, Error};
use virto_common::FungibleAssetLocation;

pub struct KreivoAssetIdExtractor;

impl AssetIdExtractor for KreivoAssetIdExtractor {
	type AssetId = FungibleAssetLocation;
	fn asset_id_from_address(addr: &[u8; 20]) -> Result<Self::AssetId, Error> {
		let bytes: [u8; 8] = addr[0..8].try_into().expect("slice is 8 bytes; qed");
		FungibleAssetLocation::decode(&mut &bytes[..])
			.map_err(|_| Error::Revert(Revert::from("Invalid encoded AssetID")))
	}
}

/// A precompile configuration that uses a prefix [`AddressMatcher`].
pub struct KreivoAssetsConfig<const PREFIX: u16>;

impl<const PREFIX: u16> AssetPrecompileConfig for KreivoAssetsConfig<PREFIX> {
	const MATCHER: AddressMatcher = AddressMatcher::Prefix(core::num::NonZero::new(PREFIX).unwrap());
	type AssetIdExtractor = KreivoAssetIdExtractor;
}
