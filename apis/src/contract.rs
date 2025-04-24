use super::*;

use crate::contract::config::ListingsConfig;
use contract::config::{AssetsConfig, Config};
use ink::env::{DefaultEnvironment, Environment};

mod api_impls;
mod chain_extension;
pub mod config;
pub use api_impls::KreivoApi;

#[derive(Clone)]
pub struct KreivoApiEnvironment<E: Clone = DefaultEnvironment>(E);

impl<E: Clone> KreivoApiEnvironment<E> {
	pub fn new(env: E) -> Self {
		Self(env)
	}
}

impl<E> Environment for KreivoApiEnvironment<E>
where
	E: Environment,
{
	const MAX_EVENT_TOPICS: usize = E::MAX_EVENT_TOPICS;
	type AccountId = E::AccountId;
	type Balance = E::Balance;
	type Hash = E::Hash;
	type Timestamp = E::Timestamp;
	type BlockNumber = E::BlockNumber;
	type ChainExtension = chain_extension::ChainExtension;
}

impl<E: Environment> Config for KreivoApiEnvironment<E> {
	type AccountId = ink::primitives::AccountId;
	type Balance = E::Balance;
}

impl<E: Environment> AssetsConfig for KreivoApiEnvironment<E> {
	type AssetId = virto_common::FungibleAssetLocation;
	type Balance = E::Balance;
}

impl<E: Environment> ListingsConfig for KreivoApiEnvironment<E> {
	type InventoryId = virto_common::listings::InventoryId;
	type ItemId = virto_common::listings::ItemId;
}
