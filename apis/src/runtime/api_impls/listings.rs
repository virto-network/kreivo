use super::*;

use config::Config;
use frame_contrib_traits::listings::{item::ItemPrice, *};

type MerchantIdOf<T> = <<T as Config>::MerchantIdInfo as MerchantIdInfo<AccountIdOf<T>>>::MerchantId;

/// A helper structure that implements [`ListingsInventoriesAPI`] in the context
/// of the Runtime.
pub struct RuntimeListingsAPI<T>(PhantomData<T>);

impl<T: Config> RuntimeListingsAPI<T> {
	fn merchant_id<E: Ext<T = T>>(ext: &E) -> Option<MerchantIdOf<T>> {
		T::MerchantIdInfo::maybe_merchant_id(ext.address())
	}
}

impl<T: Config, E> ListingsInventoriesAPI<E> for RuntimeListingsAPI<T>
where
	E: Ext<T = T>,
{
	type InventoryId = <<T as Config>::Listings as InspectItem<AccountIdOf<T>>>::InventoryId;

	fn inventory_exists(ext: &E, id: &Self::InventoryId) -> bool {
		Self::merchant_id(ext).is_some_and(|merchant_id| T::Listings::exists(&(merchant_id, *id)))
	}

	fn inventory_is_active(ext: &E, id: &Self::InventoryId) -> bool {
		Self::merchant_id(ext).is_some_and(|merchant_id| T::Listings::is_active(&(merchant_id, *id)))
	}

	fn inventory_attribute<K: Encode, V: Encode + Decode>(ext: &E, id: &Self::InventoryId, key: &K) -> Option<V> {
		let merchant_id = Self::merchant_id(ext)?;

		T::Listings::inventory_attribute(&(merchant_id, *id), key)
	}

	fn create(ext: &E, id: &Self::InventoryId) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::create((merchant_id, *id), ext.address())
			.map_err(|_| ListingsApiError::FailedToCreateInventory.into())
	}

	fn archive(ext: &E, id: &Self::InventoryId) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		if !T::Listings::exists(&(merchant_id, *id)) {
			Err(ListingsApiError::UnknownInventory)?
		}
		if !T::Listings::is_active(&(merchant_id, *id)) {
			Err(ListingsApiError::ArchivedInventory)?
		}

		T::Listings::archive(&(merchant_id, *id)).map_err(|_| ListingsApiError::FailedToArchiveInventory.into())
	}

	fn inventory_set_attribute<K: Encode, V: Encode>(
		ext: &E,
		id: &Self::InventoryId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		if !T::Listings::exists(&(merchant_id, *id)) {
			Err(ListingsApiError::UnknownInventory)?
		}

		T::Listings::set_inventory_attribute(&(merchant_id, *id), key, value)
			.map_err(|_| ListingsApiError::FailedToSetAttribute.into())
	}

	fn inventory_clear_attribute<K: Encode, V: Encode>(
		ext: &E,
		id: &Self::InventoryId,
		key: &K,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		if !T::Listings::exists(&(merchant_id, *id)) {
			Err(ListingsApiError::UnknownInventory)?
		}

		T::Listings::clear_inventory_attribute(&(merchant_id, *id), key)
			.map_err(|_| ListingsApiError::FailedToSetAttribute.into())
	}
}

type AssetIdOf<T> = <<T as Config>::Assets as frame_support::traits::fungibles::Inspect<AccountIdOf<T>>>::AssetId;
type BalanceOf<T> = <<T as Config>::Assets as frame_support::traits::fungibles::Inspect<AccountIdOf<T>>>::Balance;

type InventoryIdOf<T> = <<T as Config>::Listings as InspectItem<AccountIdOf<T>>>::InventoryId;
type ItemIdOf<T> = <<T as Config>::Listings as InspectItem<AccountIdOf<T>>>::ItemId;

impl<T: Config, E> ListingsItemsAPI<E> for RuntimeListingsAPI<T>
where
	E: Ext<T = T>,
{
	type AccountId = AccountIdOf<T>;
	type InventoryId = InventoryIdOf<T>;
	type ItemId = ItemIdOf<T>;
	type AssetId = AssetIdOf<T>;
	type Balance = BalanceOf<T>;

	fn item(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> Option<ItemOf<Self, E>> {
		let merchant_id = Self::merchant_id(ext)?;
		T::Listings::item(&(merchant_id, *inventory_id), id)
	}

	fn item_attribute<K: Encode, V: Decode>(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
	) -> Option<V> {
		let merchant_id = Self::merchant_id(ext)?;
		T::Listings::attribute(&(merchant_id, *inventory_id), id, key)
	}

	fn item_transferable(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> bool {
		Self::merchant_id(ext).is_some_and(|merchant_id| T::Listings::transferable(&(merchant_id, *inventory_id), id))
	}

	fn item_can_resell(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> bool {
		Self::merchant_id(ext).is_some_and(|merchant_id| T::Listings::can_resell(&(merchant_id, *inventory_id), id))
	}

	fn publish(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		name: Vec<u8>,
		maybe_price: Option<ItemPrice<Self::AssetId, Self::Balance>>,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::publish(&(merchant_id, *inventory_id), id, name, maybe_price)
			.map_err(|_| ListingsApiError::FailedToCreateInventory.into())
	}

	fn set_price(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		price: ItemPrice<Self::AssetId, Self::Balance>,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::set_price(&(merchant_id, *inventory_id), id, price)
			.map_err(|_| ListingsApiError::FailedToCreateInventory.into())
	}

	fn clear_price(_ext: &E, _inventory_id: &Self::InventoryId, _id: &Self::ItemId) -> Result<(), KreivoApisError> {
		// TODO: Wait until this is defined in the Listings APIs.
		Err(KreivoApisError::UnknownError)
	}

	fn item_enable_resell(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::enable_resell(&(merchant_id, *inventory_id), id)
			.map_err(|_| ListingsApiError::FailedToSetNotForResale.into())
	}

	fn item_disable_resell(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::disable_resell(&(merchant_id, *inventory_id), id)
			.map_err(|_| ListingsApiError::FailedToSetNotForResale.into())
	}

	fn item_enable_transfer(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::enable_transfer(&(merchant_id, *inventory_id), id)
			.map_err(|_| ListingsApiError::FailedToSetTransferable.into())
	}

	fn item_disable_transfer(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::disable_transfer(&(merchant_id, *inventory_id), id)
			.map_err(|_| ListingsApiError::FailedToSetTransferable.into())
	}

	fn item_set_attribute<K: Encode, V: Encode>(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::set_attribute(&(merchant_id, *inventory_id), id, key, value)
			.map_err(|_| ListingsApiError::FailedToSetAttribute.into())
	}

	fn item_clear_attribute<K: Encode>(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
	) -> Result<(), KreivoApisError> {
		let merchant_id = Self::merchant_id(ext).ok_or(ListingsApiError::NoMerchantId)?;

		T::Listings::clear_attribute(&(merchant_id, *inventory_id), id, key)
			.map_err(|_| ListingsApiError::FailedToSetAttribute.into())
	}
}
