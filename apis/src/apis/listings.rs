//! # Listings APIs
//!
//! Facilitate merchants (associated to a contract by having deployed it)
//! managing inventories and items for inventories created by them.

use crate::apis::error::KreivoApisError;
use alloc::vec::Vec;
use frame_contrib_traits::listings::item::{Item, ItemPrice};
use frame_support::Parameter;
use parity_scale_codec::{Decode, Encode};

type AccountIdOf<T, Env> = <T as ListingsAPI<Env>>::AccountId;
type AssetIdOf<T, Env> = <T as ListingsAPI<Env>>::AssetId;
type BalanceOf<T, Env> = <T as ListingsAPI<Env>>::Balance;

pub type IdItemOf<T, Env> = (
	(<T as ListingsAPI<Env>>::InventoryId, <T as ListingsAPI<Env>>::ItemId),
	Item<AccountIdOf<T, Env>, AssetIdOf<T, Env>, BalanceOf<T, Env>>,
);

/// An API for managing the listings of a merchant. It is assumed that the `Env`
/// context must provide the info of whom the merchant is.
pub trait ListingsAPI<Env> {
	type AccountId: Parameter;
	type InventoryId: Parameter;
	type ItemId: Parameter;
	type Balance: Parameter + Copy;
	type AssetId: Parameter;

	// Inventories

	/// Creates a new inventory, charging the merchant as the inventory owner.
	fn create(&self, id: Self::InventoryId) -> Result<(), KreivoApisError>;

	/// Archives an active inventory if owned by the merchant.
	fn archive(&self, id: Self::InventoryId) -> Result<(), KreivoApisError>;

	/// Returns whether an inventory exists.
	fn inventory_exists(&self, id: Self::InventoryId) -> bool;

	/// Returns whether an inventory is active or not.
	fn is_inventory_active(&self, id: Self::InventoryId) -> bool;

	// Items

	/// Given an existing active inventory, publishes a new item.
	fn publish(
		&self,
		inventory_id: Self::InventoryId,
		id: Self::ItemId,
		name: Vec<u8>,
		maybe_price: Option<ItemPrice<Self::AssetId, Self::Balance>>,
	) -> Result<(), KreivoApisError>;

	/// Retrieves an item by its `inventory_id` and item `id`.
	fn item(&self, inventory_id: Self::InventoryId, id: Self::ItemId) -> IdItemOf<Self, Env>;

	/// Retrieves an item attribute, if it exists.
	fn item_attribute<K: Encode, V: Decode>(
		&self,
		inventory_id: Self::InventoryId,
		id: Self::ItemId,
		key: K,
	) -> Option<V>;

	/// Indicates whether an item is transferable. False if the item does not
	/// exist.
	fn item_transferable(&self, inventory_id: Self::InventoryId, id: Self::ItemId) -> bool;

	/// Indicates whether an item can be resold. False if the item does not
	/// exist.
	fn item_can_resell(&self, inventory_id: Self::InventoryId, id: Self::ItemId) -> bool;
}
