use super::*;
use frame_support::traits::MapSuccess;

use pallet_listings::{InventoryId, InventoryIdFor, ItemIdOf};
use sp_runtime::traits::{AccountIdConversion, Verify};

#[cfg(not(feature = "runtime-benchmarks"))]
use frame_system::EnsureNever;
use frame_system::EnsureSigned;
use sp_runtime::morph_types;
use virto_common::listings;

parameter_types! {
	pub KeyLimit: u32 = 64;
	pub ValueLimit: u32 = 256;
}

pub type ListingsInstance = pallet_listings::Instance1;

// #[runtime::pallet_index(61)]
// pub type Listings
pub struct EnsureCommunity;

impl<Id> EnsureOriginWithArg<RuntimeOrigin, InventoryId<CommunityId, Id>> for EnsureCommunity {
	type Success = AccountId;

	fn try_origin(
		o: RuntimeOrigin,
		InventoryId(community_id, _): &InventoryId<CommunityId, Id>,
	) -> Result<Self::Success, RuntimeOrigin> {
		match o.clone().caller {
			OriginCaller::Communities(origin) => (origin.id() == *community_id)
				.then_some(Communities::community_account(community_id))
				.ok_or(o),
			OriginCaller::system(frame_system::RawOrigin::Signed(ref who)) => {
				let Some((_, id)) = PalletId::try_from_sub_account::<CommunityId>(who) else {
					return Err(o);
				};
				ensure!(community_id == &id, o);
				Ok(who.clone())
			}
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(InventoryId(community_id, _): &InventoryId<CommunityId, Id>) -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::signed(Communities::community_account(community_id)))
	}
}

impl pallet_listings::Config<ListingsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Balances = Balances;
	type Assets = Assets;
	type Nonfungibles = ListingsCatalog;
	type NonfungiblesKeyLimit = KeyLimit;
	type NonfungiblesValueLimit = ValueLimit;
	type CreateInventoryOrigin = EnsureCommunity;
	type InventoryAdminOrigin = EnsureCommunity;
	type MerchantId = CommunityId;
	type InventoryId = listings::InventoryId;
	type ItemSKU = listings::ItemId;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmarks::ListingsBenchmarkHelper;
}

// #[runtime::pallet_index(62)]
// pub type ListingsCatalog
impl pallet_nfts::Config<ListingsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = InventoryIdFor<Self, ListingsInstance>;
	type ItemId = ItemIdOf<Self, ListingsInstance>;
	type Currency = Balances;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ForceOrigin = EnsureNever<AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type ForceOrigin = EnsureRoot<AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type CreateOrigin = EnsureNever<AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type CreateOrigin = frame_system::EnsureSigned<AccountId>;
	type Locker = ();
	type CollectionDeposit = ();
	type ItemDeposit = ();
	type MetadataDepositBase = ();
	type AttributeDepositBase = ();
	type DepositPerByte = ();
	type StringLimit = ValueLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type ApprovalsLimit = ();
	type ItemAttributesApprovalsLimit = ();
	type MaxTips = ();
	type MaxDeadlineDuration = ();
	type MaxAttributesPerCall = ();
	type Features = ();
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = benchmarks::ListingsCatalogBenchmarkHelper;
	type WeightInfo = ();
	type BlockNumberProvider = System;
}

morph_types! {
	pub type MaxCartsForRegularUsers = |id: AccountId| -> (AccountId, u32) { (id, 5) };
	pub type MaxItemsForRegularUsers = |id: AccountId| -> (AccountId, u32) { (id, 64) };
}

parameter_types! {
	pub MaxLifetimeForCheckoutOrder: BlockNumber = 30 * MINUTES;
	pub MaxCartLen: u32 = 5;
	pub MaxItemLen: u32 = 64;
}

impl pallet_orders::Config<ListingsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
	type CreateOrigin = MapSuccess<EnsureSigned<AccountId>, MaxCartsForRegularUsers>;
	type OrderAdminOrigin = MapSuccess<EnsureSigned<AccountId>, MaxItemsForRegularUsers>;
	type PaymentOrigin = EnsureSigned<AccountId>;
	type OrderId = u64;
	type Listings = Listings;
	type Payments = Payments;
	type Scheduler = Scheduler;
	type MaxLifetimeForCheckoutOrder = MaxLifetimeForCheckoutOrder;
	type MaxCartLen = MaxCartLen;
	type MaxItemLen = MaxItemLen;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmarks::OrdersBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;
	use pallet_orders::InventoryIdOf;

	pub struct ListingsBenchmarkHelper;

	impl pallet_listings::BenchmarkHelper<InventoryIdFor<Runtime, ListingsInstance>> for ListingsBenchmarkHelper {
		fn inventory_id() -> InventoryIdFor<Runtime, ListingsInstance> {
			InventoryId(0, 1)
		}
	}

	pub struct ListingsCatalogBenchmarkHelper;

	impl
		pallet_nfts::BenchmarkHelper<
			InventoryIdFor<Runtime, ListingsInstance>,
			ItemIdOf<Runtime, ListingsInstance>,
			sp_runtime::MultiSigner,
			sp_runtime::AccountId32,
			sp_runtime::MultiSignature,
		> for ListingsCatalogBenchmarkHelper
	{
		fn collection(i: u16) -> InventoryIdFor<Runtime, ListingsInstance> {
			InventoryId(i, 0)
		}

		fn item(i: u16) -> ItemIdOf<Runtime, ListingsInstance> {
			i.into()
		}

		fn signer() -> (sp_runtime::MultiSigner, sp_runtime::AccountId32) {
			<() as pallet_nfts::BenchmarkHelper<
				u16,
				u16,
				sp_runtime::MultiSigner,
				sp_runtime::AccountId32,
				sp_runtime::MultiSignature,
			>>::signer()
		}

		fn sign(signer: &sp_runtime::MultiSigner, message: &[u8]) -> sp_runtime::MultiSignature {
			<() as pallet_nfts::BenchmarkHelper<
				u16,
				u16,
				sp_runtime::MultiSigner,
				sp_runtime::AccountId32,
				sp_runtime::MultiSignature,
			>>::sign(signer, message)
		}
	}

	type MerchantIdOf<T, I> = <T as pallet_listings::Config<I>>::MerchantId;

	pub struct OrdersBenchmarkHelper;

	impl pallet_orders::BenchmarkHelper<Runtime, ListingsInstance> for OrdersBenchmarkHelper {
		type Balances = Balances;
		type Assets = Assets;

		fn inventory_id() -> (
			MerchantIdOf<Runtime, ListingsInstance>,
			InventoryIdOf<Runtime, ListingsInstance>,
		) {
			(0, 0)
		}

		fn item_id(i: usize) -> ItemIdOf<Runtime, ListingsInstance> {
			i as u64
		}
	}
}
