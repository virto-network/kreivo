use frame_contrib_traits::listings::item::{Item, ItemPrice};
use ink::env::FromLittleEndian;
use ink::scale;

// Global Traits
pub trait Parameter: 'static + scale::Codec + scale::MaxEncodedLen + Clone + PartialEq + Eq {}

impl<T> Parameter for T where T: 'static + scale::Codec + scale::MaxEncodedLen + Clone + PartialEq + Eq {}

pub trait Number:
	'static + scale::Codec + Clone + PartialEq + Eq + Copy + From<u16> + From<u32> + FromLittleEndian
{
}

impl<T> Number for T where
	T: 'static + scale::Codec + Clone + PartialEq + Eq + Copy + From<u16> + From<u32> + FromLittleEndian
{
}

// System
pub trait Config {
	type AccountId: Parameter;
	type Balance: Number;
}

pub type AccountIdOf<T> = <T as Config>::AccountId;
pub type BalanceOf<T> = <T as Config>::Balance;

// Assets
pub trait AssetsConfig: Config {
	type AssetId: Parameter;
	type Balance: Number;
}

pub type AssetIdOf<T> = <T as AssetsConfig>::AssetId;
pub type AssetBalanceOf<T> = <T as AssetsConfig>::Balance;

// Listings
pub trait ListingsConfig: Config {
	type InventoryId: Parameter + Copy;
	type ItemId: Parameter + Copy;
}

pub type InventoryIdOf<T> = <T as ListingsConfig>::InventoryId;
pub type ItemIdOf<T> = <T as ListingsConfig>::ItemId;
pub type ItemOf<T> = Item<AccountIdOf<T>, AssetIdOf<T>, AssetBalanceOf<T>>;
pub type ItemPriceOf<T> = ItemPrice<AssetIdOf<T>, AssetBalanceOf<T>>;

// Memberships
pub trait MembershipsConfig: Config {
	type Membership: Parameter + Copy;
	type Rank: Parameter + Copy;
}

pub type MembershipOf<T> = <T as MembershipsConfig>::Membership;
pub type RankOf<T> = <T as MembershipsConfig>::Rank;
