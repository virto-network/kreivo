use super::{
	apis::{AssetsAPI, KreivoAPI, KreivoApisError, ListingsInventoriesAPI, ListingsItemsAPI},
	chain_extension::ChainExtension,
	config::{AccountIdOf, AssetBalanceOf, AssetIdOf, InventoryIdOf, ItemIdOf, ItemOf, ItemPriceOf},
	KreivoApiEnvironment,
};
use crate::apis::MembershipsAPI;
use crate::contract::config::{MembershipOf, RankOf};
use core::marker::PhantomData;
use frame_support::Parameter;
use ink::{
	prelude::vec::Vec,
	scale::{Decode, Encode},
	EnvAccess,
};
use ink_env::Environment;

pub struct KreivoApi<E = KreivoApiEnvironment>(PhantomData<E>);

impl<E: Environment<ChainExtension = ChainExtension>> KreivoAPI<EnvAccess<'_, E>> for KreivoApi<E> {
	type Assets = KreivoAssetsApi;
	type Listings = KreivoListingsApi;
	type Memberships = KreivoMembershipsApi;
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

	fn set_inventory_metadata(
		env: &EnvAccess<'_, E>,
		id: &Self::InventoryId,
		metadata: &[u8],
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__set_inventory_metadata(*id, metadata.to_vec())
			.map_err(|code| code.into())
	}

	fn clear_inventory_metadata(env: &EnvAccess<'_, E>, id: &Self::InventoryId) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__clear_inventory_metadata(*id)
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

	fn set_metadata(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		value: &[u8],
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__set_metadata(*inventory_id, *id, value.to_vec())
			.map_err(|code| code.into())
	}

	fn clear_metadata(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__clear_metadata(*inventory_id, *id)
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

	fn item_transfer(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		beneficiary: &Self::AccountId,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_transfer(*inventory_id, *id, *beneficiary)
			.map_err(|code| code.into())
	}

	fn item_creator_transfer(
		env: &EnvAccess<'_, E>,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		beneficiary: &Self::AccountId,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.listings__item_creator_transfer(*inventory_id, *id, *beneficiary)
			.map_err(|code| code.into())
	}
}

// Memberships
pub struct KreivoMembershipsApi;

impl<E> MembershipsAPI<EnvAccess<'_, E>> for KreivoMembershipsApi
where
	E: Environment<ChainExtension = ChainExtension>,
{
	type AccountId = AccountIdOf<KreivoApiEnvironment>;
	type MembershipId = MembershipOf<KreivoApiEnvironment>;
	type Rank = RankOf<KreivoApiEnvironment>;

	fn assign_membership(env: &EnvAccess<'_, E>, who: &Self::AccountId) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.memberships__assign_membership(*who)
			.map_err(|code| code.into())
	}

	fn membership_of(env: &EnvAccess<'_, E>, who: &Self::AccountId) -> Option<Self::MembershipId> {
		env.clone().extension().memberships__membership_of(*who)
	}

	fn rank_of(env: &EnvAccess<'_, E>, id: &Self::MembershipId) -> Option<Self::Rank> {
		env.clone().extension().memberships__rank_of(*id)
	}

	fn attribute<K: Encode, V: Parameter>(env: &EnvAccess<'_, E>, id: &Self::MembershipId, key: &K) -> Option<V> {
		env.clone()
			.extension()
			.memberships__attribute(*id, key.encode())
			.and_then(|v| Decode::decode(&mut v.as_ref()).ok())
	}

	fn set_attribute<K: Encode, V: Encode>(
		env: &EnvAccess<'_, E>,
		id: &Self::MembershipId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.memberships__set_attribute(*id, key.encode(), value.encode())
			.map_err(|code| code.into())
	}

	fn clear_attribute<K: Encode>(
		env: &EnvAccess<'_, E>,
		id: &Self::MembershipId,
		key: &K,
	) -> Result<(), KreivoApisError> {
		env.clone()
			.extension()
			.memberships__clear_attribute(*id, key.encode())
			.map_err(|code| code.into())
	}

	fn filter_membership<K: Encode, V: Parameter>(
		env: &EnvAccess<'_, E>,
		who: &Self::AccountId,
		key: &K,
		value: &V,
	) -> Option<Self::MembershipId> {
		env.clone()
			.extension()
			.memberships__filter_membership(*who, key.encode(), value.encode())
	}
}
