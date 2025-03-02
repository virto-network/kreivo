use super::*;

use pallet_listings::{InventoryId, ItemType};
use sp_runtime::traits::{AccountIdConversion, Verify};

#[cfg(not(feature = "runtime-benchmarks"))]
use frame_system::EnsureNever;

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

impl pallet_listings::Config<ListingsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Assets = Assets;
	type CreateInventoryOrigin = EnsureCommunity;
	type InventoryAdminOrigin = EnsureCommunity;
	type MerchantId = CommunityId;
	type InventoryId = u32;
	type ItemSKU = u64;
}

// #[runtime::pallet_index(62)]
// pub type ListingsCatalog
impl pallet_nfts::Config<ListingsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = InventoryId<CommunityId, u32>;
	type ItemId = ItemType<u64>;
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
	type StringLimit = ConstU32<256>;
	type KeyLimit = ConstU32<64>;
	type ValueLimit = ConstU32<256>;
	type ApprovalsLimit = ();
	type ItemAttributesApprovalsLimit = ();
	type MaxTips = ();
	type MaxDeadlineDuration = ();
	type MaxAttributesPerCall = ();
	type Features = ();
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = CommunitiesCatalogBenchmarkHelper<Self, ListingsInstance>;
	type WeightInfo = ();
}

#[cfg(feature = "runtime-benchmarks")]
pub struct CommunitiesCatalogBenchmarkHelper<T, I>(core::marker::PhantomData<(T, I)>);

#[cfg(feature = "runtime-benchmarks")]
impl<T, I: 'static>
	pallet_nfts::BenchmarkHelper<
		InventoryId<CommunityId, u32>,
		ItemType<u64>,
		sp_runtime::MultiSigner,
		sp_runtime::AccountId32,
		sp_runtime::MultiSignature,
	> for CommunitiesCatalogBenchmarkHelper<T, I>
where
	T: pallet_nfts::Config<I>,
{
	fn collection(i: u16) -> InventoryId<CommunityId, u32> {
		InventoryId(i, 0)
	}

	fn item(i: u16) -> ItemType<u64> {
		ItemType::Unit(i.into())
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
