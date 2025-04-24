use frame_contrib_traits::listings;
use frame_contrib_traits::listings::ListingsIdentifier;
use frame_support::traits::{fungible, fungibles};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type MerchantIdOf<T> = <<T as Config>::MerchantIdInfo as MerchantIdInfo<AccountIdOf<T>>>::MerchantId;
type AssetIdOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::AssetId;
type AssetBalanceOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::Balance;

pub trait Config: pallet_contracts::Config {
	/// A type that implements the balances APIs.
	type Balances: fungible::Inspect<Self::AccountId>;
	/// A type that implements the assets' APIs.
	type Assets: fungibles::Inspect<Self::AccountId> + fungibles::Mutate<Self::AccountId>;
	/// A type that implements the `MerchantIdInfo` trait.
	type MerchantIdInfo: MerchantIdInfo<AccountIdOf<Self>>;
	/// A type that implements the listings' APIs.
	type Listings: listings::InspectInventory<
			InventoryId = <Self::Listings as listings::InspectItem<AccountIdOf<Self>>>::InventoryId,
		> + listings::InventoryLifecycle<Self::AccountId, MerchantId = MerchantIdOf<Self>>
		+ listings::MutateInventory<MerchantId = MerchantIdOf<Self>>
		+ listings::InspectItem<
			Self::AccountId,
			MerchantId = MerchantIdOf<Self>,
			Asset = AssetIdOf<Self>,
			Balance = AssetBalanceOf<Self>,
		> + listings::MutateItem<
			Self::AccountId,
			MerchantId = MerchantIdOf<Self>,
			Asset = AssetIdOf<Self>,
			Balance = AssetBalanceOf<Self>,
		>;
}

pub trait MerchantIdInfo<AccountId> {
	type MerchantId: ListingsIdentifier;

	fn maybe_merchant_id(who: &AccountId) -> Option<Self::MerchantId>;
}
