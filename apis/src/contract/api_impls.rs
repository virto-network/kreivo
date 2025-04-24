use core::marker::PhantomData;
use ink::{
	prelude::vec::Vec,
	scale::{Decode, Encode},
	EnvAccess,
};
use ink_env::Environment;

use super::{
	apis::{AssetsAPI, KreivoAPI, KreivoApisError, ListingsInventoriesAPI, ListingsItemsAPI},
	chain_extension::ChainExtension,
	config::{AccountIdOf, AssetBalanceOf, AssetIdOf, InventoryIdOf, ItemIdOf, ItemOf, ItemPriceOf},
	KreivoApiEnvironment,
};

pub struct KreivoApi<E = KreivoApiEnvironment>(PhantomData<E>);

impl<E: Environment<ChainExtension = ChainExtension>> KreivoAPI<EnvAccess<'_, E>> for KreivoApi<E> {
	type Assets = KreivoAssetsApi;
	type Listings = KreivoListingsApi;
}

// Assets
pub struct KreivoAssetsApi;

impl<E> AssetsAPI<EnvAccess<'_, E>> for KreivoAssetsApi
where
	E: Environment<ChainExtension = ChainExtension>,
{
	type AccountId = AccountIdOf<KreivoApiEnvironment>;
	type AssetId = AssetIdOf<KreivoApiEnvironment>;
	type Balance = AssetBalanceOf<KreivoApiEnvironment>;

	fn balance(env: &EnvAccess<'_, E>, asset: Self::AssetId, who: &Self::AccountId) -> Self::Balance {
		env.clone().extension().assets__balance(asset, *who)
	}

	fn deposit(
		env: &EnvAccess<'_, E>,
		asset: Self::AssetId,
		amount: Self::Balance,
	) -> Result<Self::Balance, KreivoApisError> {
		env.clone()
			.extension()
			.assets__deposit(asset, amount)
			.map_err(|code| code.into())
	}

	fn transfer(
		env: &EnvAccess<'_, E>,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError> {
		env.clone()
			.extension()
			.assets__transfer(asset, amount, *beneficiary)
			.map_err(|code| code.into())
	}
}

// Listings
pub struct KreivoListingsApi;

impl<E> ListingsInventoriesAPI<EnvAccess<'_, E>> for KreivoListingsApi
where
	E: Environment<ChainExtension = ChainExtension>,
{
	type InventoryId = InventoryIdOf<KreivoApiEnvironment>;

	fn inventory_exists(env: &EnvAccess<'_, E>, id: &Self::InventoryId) -> bool {
		env.clone().extension().listings__inventory_exists(*id)
	}

	fn inventory_is_active(env: &EnvAccess<'_, E>, id: &Self::InventoryId) -> bool {
		env.clone().extension().listings__inventory_is_active(*id)
	}

	fn inventory_attribute<K: Encode, V: Encode + Decode>(
		env: &EnvAccess<'_, E>,
		id: &Self::InventoryId,
		key: &K,
	) -> Option<V> {
		env.clone()
			.extension()
			// Infallible: Truncates the key up to the BoundedVec limit.
			.listings__inventory_attribute(*id, key.encode())
			// Infallible: Falls back to `None` if unable to decode.
			.and_then(|v| Decode::decode(&mut v.as_ref()).ok())
	}

	fn create(env: &EnvAccess<'_, E>, id: &Self::InventoryId) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__inventory_create(*id)
			.map_err(|code| code.into())
	}

	fn archive(env: &EnvAccess<'_, E>, id: &Self::InventoryId) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__inventory_archive(*id)
			.map_err(|code| code.into())
	}

	fn inventory_set_attribute<K: Encode, V: Encode>(
		env: &EnvAccess<'_, E>,
		id: &Self::InventoryId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__inventory_set_attribute(*id, key.encode(), value.encode())
			.map_err(|code| code.into())
	}

	fn inventory_clear_attribute<K: Encode, V: Encode>(
		env: &EnvAccess<'_, E>,
		id: &Self::InventoryId,
		key: &K,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__inventory_clear_attribute(*id, key.encode())
			.map_err(|code| code.into())
	}
}

impl<E> ListingsItemsAPI<EnvAccess<'_, E>> for KreivoListingsApi
where
	E: Environment<ChainExtension = ChainExtension>,
{
	type AccountId = AccountIdOf<KreivoApiEnvironment>;
	type InventoryId = InventoryIdOf<KreivoApiEnvironment>;
	type ItemId = ItemIdOf<KreivoApiEnvironment>;
	type AssetId = AssetIdOf<KreivoApiEnvironment>;
	type Balance = AssetBalanceOf<KreivoApiEnvironment>;

	fn item(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Option<ItemOf<KreivoApiEnvironment>> {
		env.clone().extension().listings__item(*inventory_id, *id)
	}

	fn item_attribute<K: Encode, V: Decode>(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
	) -> Option<V> {
		env.clone()
			.extension()
			.listings__item_attribute(*inventory_id, *id, key.encode())
			.and_then(|v| Decode::decode(&mut v.as_ref()).ok())
	}

	fn item_transferable(env: &EnvAccess<'_, E>, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> bool {
		env.clone().extension().listings__item_transferable(*inventory_id, *id)
	}

	fn item_can_resell(env: &EnvAccess<'_, E>, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> bool {
		env.clone().extension().listings__item_can_resell(*inventory_id, *id)
	}

	fn publish(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		name: Vec<u8>,
		maybe_price: Option<ItemPriceOf<KreivoApiEnvironment>>,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_publish(*inventory_id, *id, name, maybe_price)
			.map_err(|code| code.into())
	}

	fn set_price(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		price: ItemPriceOf<KreivoApiEnvironment>,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_set_price(*inventory_id, *id, price)
			.map_err(|code| code.into())
	}

	fn clear_price(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_clear_price(*inventory_id, *id)
			.map_err(|code| code.into())
	}

	fn item_enable_resell(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_enable_resell(*inventory_id, *id)
			.map_err(|code| code.into())
	}

	fn item_disable_resell(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_disable_resell(*inventory_id, *id)
			.map_err(|code| code.into())
	}

	fn item_enable_transfer(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_enable_transfer(*inventory_id, *id)
			.map_err(|code| code.into())
	}

	fn item_disable_transfer(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_disable_transfer(*inventory_id, *id)
			.map_err(|code| code.into())
	}

	fn item_set_attribute<K: Encode, V: Encode>(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_set_attribute(*inventory_id, *id, key.encode(), value.encode())
			.map_err(|code| code.into())
	}

	fn item_clear_attribute<K: Encode>(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_clear_attribute(*inventory_id, *id, key.encode())
			.map_err(|code| code.into())
	}
}
