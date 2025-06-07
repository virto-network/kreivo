use super::*;
use frame_contrib_traits::{gas_tank::NonFungibleGasTank, memberships::NonFungiblesMemberships};
use frame_support::{
	assert_ok, derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU32, ConstU64, EitherOf, EqualPrivilegeOnly, VariantCountOf},
	PalletId,
};
use frame_system::{EnsureNever, EnsureRoot, EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::{origin::EnsureCommunity, Tally, VoteWeight};
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
pub use virto_common::{CommunityId, MembershipId};

pub use crate as pallet_communities_manager;

mod collective;
mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod runtime_benchmarks;
#[cfg(feature = "runtime-benchmarks")]
use runtime_benchmarks::*;

type Block = frame_system::mocking::MockBlock<Test>;
pub type BlockNumber = BlockNumberFor<Test>;
type WeightInfo = ();

pub type AccountPublic = <MultiSignature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Balance = <Test as pallet_balances::Config>::Balance;

// Configure a mock runtime to test the pallet.
#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeTask,
		RuntimeHoldReason,
		RuntimeFreezeReason
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;
	#[runtime::pallet_index(1)]
	pub type Scheduler = pallet_scheduler;
	#[runtime::pallet_index(2)]
	pub type Referenda = pallet_referenda;

	#[runtime::pallet_index(10)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(11)]
	pub type Assets = pallet_assets;
	#[runtime::pallet_index(12)]
	pub type AssetsFreezer = pallet_assets_freezer;

	#[runtime::pallet_index(21)]
	pub type CollectiveReferenda = pallet_referenda<Instance1>;
	#[runtime::pallet_index(22)]
	pub type Collective = pallet_ranked_collective<Instance1>;

	#[runtime::pallet_index(31)]
	pub type Communities = pallet_communities;
	#[runtime::pallet_index(32)]
	pub type Tracks = pallet_referenda_tracks;
	#[runtime::pallet_index(33)]
	pub type Memberships = pallet_nfts;
	#[runtime::pallet_index(34)]
	pub type CommunitiesManager = pallet_communities_manager;
}

parameter_types! {
	pub const RootAccount: AccountId = AccountId::new([0xff; 32]);
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type AccountData = pallet_balances::AccountData<Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config for Test {
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type Freezer = AssetsFreezer;
}

impl pallet_assets_freezer::Config for Test {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_scheduler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Self>;
	type Preimages = ();
	type BlockNumberProvider = System;
}

parameter_types! {
	pub static AlarmInterval: u64 = 1;
	pub const MaxTracks: u32 = u32::MAX;
}

impl pallet_referenda::Config for Test {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = EnsureSigned<AccountId>;
	type CancelOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Slash = ();
	type Votes = VoteWeight;
	type Tally = Tally<Test>;
	type SubmissionDeposit = ConstU64<2>;
	type MaxQueued = ConstU32<3>;
	type UndecidingTimeout = ConstU64<20>;
	type AlarmInterval = AlarmInterval;
	type Tracks = Tracks;
	type Preimages = ();
	type BlockNumberProvider = System;
}

impl pallet_referenda_tracks::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TrackId = CommunityId;
	type MaxTracks = MaxTracks;
	type AdminOrigin = EnsureRoot<AccountId>;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = TracksBenchmarkHelper;
}

type Deposit = Option<(Balance, AccountId, AccountId)>;
parameter_types! {
	pub const CommunitiesPalletId: PalletId = PalletId(*b"kv/comms");
	pub const MembershipsManagerCollectionId: CommunityId = 0;
	pub const MembershipNftAttr: &'static [u8; 10] = b"membership";
	pub const TestCommunity: CommunityId = 1;
	pub const NoDepositOnRootRegistration: Deposit = None;
}

impl pallet_nfts::Config for Test {
	type ApprovalsLimit = ();
	type AttributeDepositBase = ();
	type CollectionDeposit = ();
	type CollectionId = CommunityId;
	type CreateOrigin =
		AsEnsureOriginWithArg<EitherOf<EnsureRootWithSuccess<AccountId, RootAccount>, EnsureSigned<AccountId>>>;
	type Currency = Balances;
	type DepositPerByte = ();
	type Features = ();
	type ForceOrigin = EnsureRoot<AccountId>;
	type ItemAttributesApprovalsLimit = ();
	type ItemDeposit = ();
	type ItemId = MembershipId;
	type KeyLimit = ConstU32<64>;
	type Locker = ();
	type MaxAttributesPerCall = ();
	type MaxDeadlineDuration = ();
	type MaxTips = ();
	type MetadataDepositBase = ();
	type OffchainPublic = AccountPublic;
	type OffchainSignature = MultiSignature;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ();
	type ValueLimit = ConstU32<40>;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type BlockNumberProvider = System;
}

impl pallet_communities::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = WeightInfo;
	type CreateOrigin = EnsureNever<Deposit>;
	type AdminOrigin = EnsureCommunity<Self>;
	type MemberMgmtOrigin = EnsureCommunity<Self>;
	type CommunityId = CommunityId;
	type MembershipId = MembershipId;
	type MemberMgmt = NonFungiblesMemberships<Memberships, pallet_nfts::ItemConfig>;
	type Polls = Referenda;
	type Assets = Assets;
	type AssetsFreezer = AssetsFreezer;
	type Balances = Balances;
	type BlockNumberProvider = System;
	type PalletId = CommunitiesPalletId;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	// Types to support community creation
	type CreateCollection = Memberships;
	type MakeTank = NonFungibleGasTank<Test, System, Memberships, pallet_nfts::ItemConfig>;
	type Tracks = Tracks;
	type RankedCollective = Collective;
	type WeightInfo = WeightInfo;
	type RegisterOrigin = EnsureRootWithSuccess<AccountId, NoDepositOnRootRegistration>;

	// Types to support memberships creation
	type CreateMembershipsOrigin = EnsureRoot<AccountId>;
	type MembershipId = MembershipId;
	type MembershipsManagerCollectionId = MembershipsManagerCollectionId;
	type MembershipsManagerOwner = RootAccount;
	type ItemConfig = pallet_nfts::ItemConfig;

	type CreateMemberships = Memberships;
}

pub fn new_test_ext() -> TestExternalities {
	let mut t = TestExternalities::new(Default::default());

	t.execute_with(|| {
		assert_ok!(Memberships::create_collection_with_id(
			MembershipsManagerCollectionId::get(),
			&RootAccount::get(),
			&RootAccount::get(),
			&pallet_nfts::CollectionConfig {
				settings: Default::default(),
				max_supply: None,
				mint_settings: Default::default(),
			},
		));
	});

	t
}
