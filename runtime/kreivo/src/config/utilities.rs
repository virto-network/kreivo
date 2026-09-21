use super::*;

use frame_support::traits::{fungible::HoldConsideration, LinearStoragePrice, MapSuccess};
use pallet_communities::origin::AsSignedByCommunity;
use sp_runtime::traits::{BlakeTwo256, ReplaceWithDefault};

// #[runtime::pallet_index(42)]
// pub type Multisig
parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<100>;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Self>;
	type BlockNumberProvider = RelaychainData;
}

// #[runtime::pallet_index(43)]
// pub type Utility
impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::WeightInfo<Self>;
}

// #[runtime::pallet_index(44)]
// pub type Proxy
parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(0, 100);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	// One storage item; key size 32, value size 16
	pub const AnnouncementDepositBase: Balance = deposit(1, 48);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = weights::pallet_proxy::WeightInfo<Self>;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type BlockNumberProvider = RelaychainData;
}

// #[runtime::pallet_index(45)]
// pub type Scheduler
/// What the scheduler may use in `on_initialize`: 80% of a full core in the first block of a
/// core, and nothing in the blocks bundled after it.
///
/// With block bundling, only the first block of a core may use the whole core; the others
/// get a share of it. A scheduled call too heavy for this block's budget is dropped for good
/// (`PermanentlyOverweight`, preimage included) when it's the first task serviced. Servicing
/// only in first blocks keeps governance calls from landing in a small share: tasks that come
/// due in a later block wait for the next core's first block. A block with no budget leaves
/// the scheduler's `IncompleteSince` alone, so no agenda is skipped.
///
/// A block without bundle info (an older collator, or tests) is alone in its PoV, so it's
/// treated as first. The scheduler is still capped by what's left of the block's weight.
pub struct MaximumSchedulerWeight;
impl frame_support::traits::Get<Weight> for MaximumSchedulerWeight {
	fn get() -> Weight {
		let digest = frame_system::Pallet::<Runtime>::digest();
		let first_in_core = cumulus_primitives_core::CumulusDigestItem::find_block_bundle_info(&digest)
			.map_or(true, |bundle| bundle.index == 0);
		if first_in_core {
			Perbill::from_percent(80) * FULL_CORE_WEIGHT
		} else {
			Weight::zero()
		}
	}
}

/// The weight a whole relay chain core offers: 2s of execution and a 10 MiB PoV.
const FULL_CORE_WEIGHT: Weight = Weight::from_parts(
	2 * frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND,
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

#[cfg(not(feature = "runtime-benchmarks"))]
parameter_types! {
	pub const MaxScheduledPerBlock: u32 = 50;
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub const MaxScheduledPerBlock: u32 = 200;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin =
		EitherOf<EnsureRoot<AccountId>, MapSuccess<AsSignedByCommunity<Runtime>, ReplaceWithDefault<()>>>;
	type OriginPrivilegeCmp = EqualOrGreatestRootCmp;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Self>;
	type Preimages = Preimage;
	type BlockNumberProvider = RelaychainData;
}

// #[runtime::pallet_index(46)]
// pub type Preimage
parameter_types! {
	pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_preimage::WeightInfo<Self>;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}
