use super::*;

use crate::apis::KreivoApisErrorCode;
use crate::contract::config::{AssetBalanceOf, InventoryIdOf, ItemIdOf, ItemOf, ItemPriceOf, MembershipOf, RankOf};
use config::{AccountIdOf, AssetIdOf, BalanceOf};
use ink::{chain_extension, prelude::vec::Vec};

type Environment = KreivoApiEnvironment;
type CallResult = Result<(), KreivoApisErrorCode>;

#[chain_extension(extension = 0)]
pub trait ChainExtension {
	type ErrorCode = KreivoApisErrorCode;

	// Assets
	#[allow(non_snake_case)]
	#[ink(function = 0x0000, handle_status = false)]
	fn assets__balance(asset: AssetIdOf<Environment>, who: AccountIdOf<Environment>) -> AssetBalanceOf<Environment>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0001)]
	fn assets__deposit(
		asset: AssetIdOf<Environment>,
		amount: BalanceOf<Environment>,
	) -> Result<BalanceOf<Environment>, KreivoApisErrorCode>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0002)]
	fn assets__transfer(
		asset: AssetIdOf<Environment>,
		amount: BalanceOf<Environment>,
		beneficiary: AccountIdOf<Environment>,
	) -> Result<BalanceOf<Environment>, KreivoApisErrorCode>;

	// Listings: Inventories
	#[allow(non_snake_case)]
	#[ink(function = 0x0100, handle_status = false)]
	fn listings__inventory_exists(id: InventoryIdOf<Environment>) -> bool;

	#[allow(non_snake_case)]
	#[ink(function = 0x0101, handle_status = false)]
	fn listings__inventory_is_active(id: InventoryIdOf<Environment>) -> bool;

	#[allow(non_snake_case)]
	#[ink(function = 0x0102, handle_status = false)]
	fn listings__inventory_attribute(id: InventoryIdOf<Environment>, k: Vec<u8>) -> Option<Vec<u8>>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0103)]
	fn listings__inventory_create(id: InventoryIdOf<Environment>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0104)]
	fn listings__inventory_archive(id: InventoryIdOf<Environment>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0105)]
	fn listings__inventory_set_attribute(id: InventoryIdOf<Environment>, key: Vec<u8>, value: Vec<u8>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0106)]
	fn listings__inventory_clear_attribute(id: InventoryIdOf<Environment>, key: Vec<u8>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0107)]
	fn listings__set_inventory_metadata(id: InventoryIdOf<Environment>, metadata: Vec<u8>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0108)]
	fn listings__clear_inventory_metadata(id: InventoryIdOf<Environment>) -> CallResult;

	// Listings: Items
	#[allow(non_snake_case)]
	#[ink(function = 0x0110, handle_status = false)]
	fn listings__item(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
	) -> Option<ItemOf<Environment>>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0111, handle_status = false)]
	fn listings__item_attribute(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
		key: Vec<u8>,
	) -> Option<Vec<u8>>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0112, handle_status = false)]
	fn listings__item_transferable(inventory_id: InventoryIdOf<Environment>, id: ItemIdOf<Environment>) -> bool;

	#[allow(non_snake_case)]
	#[ink(function = 0x0113, handle_status = false)]
	fn listings__item_can_resell(inventory_id: InventoryIdOf<Environment>, id: ItemIdOf<Environment>) -> bool;

	#[allow(non_snake_case)]
	#[ink(function = 0x0114)]
	fn listings__item_publish(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
		name: Vec<u8>,
		maybe_price: Option<ItemPriceOf<Environment>>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0115)]
	fn listings__item_set_price(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
		price: ItemPriceOf<Environment>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0116)]
	fn listings__item_clear_price(inventory_id: InventoryIdOf<Environment>, id: ItemIdOf<Environment>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0117)]
	fn listings__item_enable_resell(inventory_id: InventoryIdOf<Environment>, id: ItemIdOf<Environment>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0118)]
	fn listings__item_disable_resell(inventory_id: InventoryIdOf<Environment>, id: ItemIdOf<Environment>)
		-> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0119)]
	fn listings__item_enable_transfer(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x011a)]
	fn listings__item_disable_transfer(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x011b)]
	fn listings__item_set_attribute(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
		key: Vec<u8>,
		value: Vec<u8>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x011c)]
	fn listings__item_clear_attribute(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
		key: Vec<u8>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x011d)]
	fn listings__item_transfer(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
		beneficiary: AccountIdOf<Environment>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x011e)]
	fn listings__item_creator_transfer(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
		beneficiary: AccountIdOf<Environment>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x011f)]
	fn listings__set_metadata(
		inventory_id: InventoryIdOf<Environment>,
		id: ItemIdOf<Environment>,
		metadata: Vec<u8>,
	) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0120)]
	fn listings__clear_metadata(inventory_id: InventoryIdOf<Environment>, id: ItemIdOf<Environment>) -> CallResult;

	// Memberships
	#[allow(non_snake_case)]
	#[ink(function = 0x0200)]
	fn memberships__assign_membership(who: AccountIdOf<Environment>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0201, handle_status = false)]
	fn memberships__membership_of(who: AccountIdOf<Environment>) -> Option<MembershipOf<Environment>>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0202, handle_status = false)]
	fn memberships__rank_of(id: MembershipOf<Environment>) -> Option<RankOf<Environment>>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0203, handle_status = false)]
	fn memberships__attribute(id: MembershipOf<Environment>, key: Vec<u8>) -> Option<Vec<u8>>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0204)]
	fn memberships__set_attribute(id: MembershipOf<Environment>, key: Vec<u8>, value: Vec<u8>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0205)]
	fn memberships__clear_attribute(id: MembershipOf<Environment>, key: Vec<u8>) -> CallResult;

	#[allow(non_snake_case)]
	#[ink(function = 0x0206, handle_status = false)]
	fn memberships__filter_membership(
		who: AccountIdOf<Environment>,
		key: Vec<u8>,
		value: Vec<u8>,
	) -> Option<MembershipOf<Environment>>;
}

impl ink::env::chain_extension::FromStatusCode for KreivoApisErrorCode {
	fn from_status_code(code: u32) -> Result<(), Self> {
		if code == 0 {
			Ok(())
		} else {
			Err(code.into())
		}
	}
}
