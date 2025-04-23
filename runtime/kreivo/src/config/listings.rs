use super::*;

use pallet_listings::{InventoryId, InventoryIdFor, ItemIdOf};
use sp_runtime::traits::{AccountIdConversion, Verify};

#[cfg(not(feature = "runtime-benchmarks"))]
use frame_system::EnsureNever;

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
		Ok(RuntimeOrigin::signed(Communities::community_account(&community_id)))
	}
}

pub type KreivoInventoryId = u32;
pub type ItemSKU = u64;

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
	type InventoryId = KreivoInventoryId;
	type ItemSKU = ItemSKU;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmarks::ListingsBenchmarkHelper<Self, ListingsInstance>;
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
	type Helper = benchmarks::ListingsCatalogBenchmarkHelper<Self, ListingsInstance>;
	type WeightInfo = ();
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;
	use core::marker::PhantomData;

	pub struct ListingsBenchmarkHelper<T, I>(PhantomData<(T, I)>);

	impl<T, I: 'static> pallet_listings::BenchmarkHelper<InventoryIdFor<T, I>> for ListingsBenchmarkHelper<T, I>
	where
		T: pallet_listings::Config<I>,
		<T as pallet_listings::Config<I>>::MerchantId: From<u16>,
		<T as pallet_listings::Config<I>>::InventoryId: From<u16>,
	{
		fn inventory_id() -> InventoryIdFor<T, I> {
			InventoryId(0.into(), 1.into())
		}
	}

	pub struct ListingsCatalogBenchmarkHelper<T, I>(PhantomData<(T, I)>);

	impl<T, I: 'static>
		pallet_nfts::BenchmarkHelper<
			InventoryIdFor<T, I>,
			ItemIdOf<T, I>,
			sp_runtime::MultiSigner,
			sp_runtime::AccountId32,
			sp_runtime::MultiSignature,
		> for ListingsCatalogBenchmarkHelper<T, I>
	where
		T: pallet_nfts::Config<I> + pallet_listings::Config<I>,
		<T as pallet_listings::Config<I>>::MerchantId: From<u16>,
		<T as pallet_listings::Config<I>>::InventoryId: From<u16>,
		<T as pallet_listings::Config<I>>::ItemSKU: From<u16>,
	{
		fn collection(i: u16) -> InventoryIdFor<T, I> {
			InventoryId(i.into(), 0.into())
		}

		fn item(i: u16) -> ItemIdOf<T, I> {
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
}
