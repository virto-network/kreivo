use super::*;

use frame_contrib_traits::listings::item::ItemPrice;
use frame_support::traits::fungible::Inspect;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};

pub type CodeHash<T> = <T as frame_system::Config>::Hash;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type AppInfoFor<T> = AppInfo<CodeHash<T>, AccountIdOf<T>, ItemPriceOf<T>>;
pub type BalanceOf<T> = <<T as pallet_contracts::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance;
pub type Contracts<T> = pallet_contracts::Pallet<T>;
pub type ListingsMerchantIdOf<T> = <<T as Config>::Listings as InspectInventory>::MerchantId;
pub type ListingsAssetOf<T> = <<T as Config>::Listings as InspectItem<AccountIdOf<T>>>::Asset;
pub type ListingsBalanceOf<T> = <<T as Config>::Listings as InspectItem<AccountIdOf<T>>>::Balance;
pub type ItemPriceOf<T> = ItemPrice<ListingsAssetOf<T>, ListingsBalanceOf<T>>;

type ListingsOf<T> = <T as Config>::Listings;
pub type LicenseIdFor<T> = <ListingsOf<T> as InspectItem<AccountIdOf<T>>>::ItemId;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct AppInfo<Hash, AccountId, ItemPrice> {
	pub(crate) code_hash: Hash,
	pub(crate) publisher: AccountId,
	pub(crate) max_instances: Option<u64>,
	pub(crate) instances: u64,
	pub(crate) price: Option<ItemPrice>,
	pub(crate) version: u32,
}

impl<Hash, AccountId, ItemPrice> AppInfo<Hash, AccountId, ItemPrice> {
	pub(crate) fn bump_version(&mut self, code_hash: Hash) -> Option<()> {
		self.code_hash = code_hash;
		self.version = self.version.checked_add(1)?;
		Some(())
	}
}
