//! # Listings APIs
//!
//! Facilitate merchants (associated to a contract by having deployed it)
//! managing inventories and items for inventories created by them.

use crate::apis::error::KreivoApisError;
use alloc::vec::Vec;
use core::fmt;
use frame_contrib_traits::listings::item::{Item, ItemPrice};
use frame_support::Parameter;
use parity_scale_codec::{Codec, Decode, Encode, EncodeLike};

/// An API for managing the listings of a merchant. It is assumed that the `Env`
/// context must provide the info of whom the merchant is.
pub trait ListingsInventoriesAPI<E> {
	type InventoryId: Parameter;

	// InspectInventory

	/// Returns whether an inventory exists.
	fn inventory_exists(ext: &E, id: &Self::InventoryId) -> bool;

	/// Returns whether an inventory is active or not.
	fn inventory_is_active(ext: &E, id: &Self::InventoryId) -> bool;

	/// Returns the value of an inventory attribute, if it exists.
	fn inventory_attribute<K: Encode, V: Encode + Decode>(ext: &E, id: &Self::InventoryId, key: &K) -> Option<V>;

	// InventoryLifecycle

	/// Creates a new inventory, charging the merchant as the inventory owner.
	fn create(ext: &E, id: &Self::InventoryId) -> Result<(), KreivoApisError>;

	/// Archives an active inventory if owned by the merchant.
	fn archive(ext: &E, id: &Self::InventoryId) -> Result<(), KreivoApisError>;

	// MutateInventory

	/// Sets an attribute on an inventory
	fn inventory_set_attribute<K: Encode, V: Encode>(
		ext: &E,
		id: &Self::InventoryId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError>;

	/// Clears an attribute on an inventory
	fn inventory_clear_attribute<K: Encode, V: Encode>(
		ext: &E,
		id: &Self::InventoryId,
		key: &K,
	) -> Result<(), KreivoApisError>;
}

type AccountIdOf<T, E> = <T as ListingsItemsAPI<E>>::AccountId;
type AssetIdOf<T, E> = <T as ListingsItemsAPI<E>>::AssetId;
type BalanceOf<T, E> = <T as ListingsItemsAPI<E>>::Balance;

pub type ItemOf<T, E> = Item<AccountIdOf<T, E>, AssetIdOf<T, E>, BalanceOf<T, E>>;

pub trait ListingsItemsAPI<E> {
	type AccountId: Codec + EncodeLike + Clone + Eq + fmt::Debug;
	type InventoryId: Parameter;
	type ItemId: Parameter;
	type AssetId: Parameter;
	type Balance: Parameter + Copy;

	// InspectItems

	/// Retrieves an item by its `inventory_id` and item `id`.
	fn item(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> Option<ItemOf<Self, E>>;

	/// Retrieves an item attribute, if it exists.
	fn item_attribute<K: Encode, V: Decode>(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
	) -> Option<V>;

	/// Indicates whether an item is transferable. False if the item does not
	/// exist.
	fn item_transferable(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> bool;

	/// Indicates whether an item can be resold. False if the item does not
	/// exist.
	fn item_can_resell(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> bool;

	// MutateItems

	/// Given an existing active inventory, publishes a new item.
	fn publish(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		name: Vec<u8>,
		maybe_price: Option<ItemPrice<Self::AssetId, Self::Balance>>,
	) -> Result<(), KreivoApisError>;

	/// Sets the price of an item.
	fn set_price(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		price: ItemPrice<Self::AssetId, Self::Balance>,
	) -> Result<(), KreivoApisError>;

	/// Clears the price of an item.
	fn clear_price(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> Result<(), KreivoApisError>;

	/// Enables an item to be resold.
	fn item_enable_resell(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId) -> Result<(), KreivoApisError>;

	/// Disables an item to be resold.
	fn item_disable_resell(ext: &E, inventory_id: &Self::InventoryId, id: &Self::ItemId)
		-> Result<(), KreivoApisError>;

	/// Enables an item to be transferable.
	fn item_enable_transfer(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError>;

	/// Disables an item to be transferable.
	fn item_disable_transfer(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
	) -> Result<(), KreivoApisError>;

	/// Sets the attribute on an item.
	fn item_set_attribute<K: Encode, V: Encode>(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError>;

	/// Sets the attribute on an item.
	fn item_clear_attribute<K: Encode>(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		key: &K,
	) -> Result<(), KreivoApisError>;

	/// Transfers an item.
	fn item_transfer(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		beneficiary: &Self::AccountId,
	) -> Result<(), KreivoApisError>;

	/// Transfers an item, also marking the beneficiary as the item creator.
	fn item_creator_transfer(
		ext: &E,
		inventory_id: &Self::InventoryId,
		id: &Self::ItemId,
		beneficiary: &Self::AccountId,
	) -> Result<(), KreivoApisError>;
}
