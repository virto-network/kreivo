use super::*;
use frame_contrib_traits::memberships::GroupCollectionMemberships;
use frame_support::{
	derive_impl, parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU16, ConstU32, ConstU64, EitherOf, EnsureOriginWithArg, EqualPrivilegeOnly,
		VariantCountOf,
	},
	PalletId,
};
use frame_system::{EnsureNever, EnsureRoot, EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::{
	origin::{EnsureCommunity, EnsureSignedPays},
	Tally, VoteWeight,
};
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
	type RuntimeHoldReason = RuntimeHoldReason;
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

/// A track id combines a community (its group) and a sub-track.
pub type TrackId = u32;

impl pallet_referenda_tracks::Config for Test {
	type TrackId = TrackId;
	type MaxTracks = MaxTracks;
	// Tracks are created by the manager, through `do_insert`.
	type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
	type GroupManagerCreateOrigin = EnsureNeverGroup;
	type GroupManagerOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
	type RemoveOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
	type RemoveGroupOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = TracksBenchmarkHelper;
}

pub struct EnsureNeverGroup;
impl EnsureOriginWithArg<RuntimeOrigin, pallet_referenda::PalletsOriginOf<Test>> for EnsureNeverGroup {
	type Success = CommunityId;

	fn try_origin(
		o: RuntimeOrigin,
		_: &pallet_referenda::PalletsOriginOf<Test>,
	) -> Result<Self::Success, RuntimeOrigin> {
		Err(o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_: &pallet_referenda::PalletsOriginOf<Test>) -> Result<RuntimeOrigin, ()> {
		Err(())
	}
}

/// Inserts the first track of a community: `combine(community_id, 0)`.
pub struct CommunityTracks;
impl MutateTracks<Balance, BlockNumber> for CommunityTracks {
	type Id = CommunityId;
	type RuntimeOrigin = pallet_referenda::PalletsOriginOf<Test>;

	fn insert(
		id: Self::Id,
		info: pallet_referenda::TrackInfo<Balance, BlockNumber>,
		origin: Self::RuntimeOrigin,
	) -> frame_support::dispatch::DispatchResult {
		Tracks::do_insert(
			<TrackId as pallet_referenda_tracks::SplitId>::combine(id, 0),
			info,
			origin,
		)
	}
}

type Deposit = Option<(Balance, AccountId, AccountId)>;
parameter_types! {
	pub const CommunitiesPalletId: PalletId = PalletId(*b"kv/comms");
	pub const MembershipNftAttr: &'static [u8; 10] = b"membership";
	pub const TestCommunity: CommunityId = 1;
	pub const MembershipsManagerAccountId: AccountId = AccountId::new([0xee; 32]);
	pub const RetiringMembershipsAccount: AccountId = AccountId::new([0xdd; 32]);
	pub const RetirementHolderOption: Option<AccountId> = Some(AccountId::new([0xdd; 32]));
	/// What each issued membership holds, unless a test changes it.
	pub static MembershipDepositAmount: Balance = 10;
	pub const RetirementDelayInBlocks: BlockNumber = 5;
	/// Where the consumption part goes.
	pub const Treasury: AccountId = AccountId::new([0xaa; 32]);
	/// The registration payment, and its two parts, unless a test changes them.
	pub const RegistrationPayment: Balance = 100;
	pub static StoragePart: Balance = 30;
	pub static ConsumptionPart: Balance = 70;
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

/// Every community's memberships live in the community's own collection.
pub type MembershipsManager = GroupCollectionMemberships<
	Memberships,
	pallet_nfts::ItemConfig,
	CommunityAccount,
	MembershipsManagerAccountId,
	RetirementHolderOption,
>;

/// The community of a track is the group half of its id.
pub struct CommunityOfTrack;
impl sp_runtime::traits::Convert<TrackId, Option<CommunityId>> for CommunityOfTrack {
	fn convert(track: TrackId) -> Option<CommunityId> {
		Some(pallet_referenda_tracks::SplitId::split(track).0)
	}
}

/// The account of a community, which holds its stock of memberships.
pub struct CommunityAccount;
impl sp_runtime::traits::Convert<CommunityId, AccountId> for CommunityAccount {
	fn convert(community_id: CommunityId) -> AccountId {
		Communities::community_account(&community_id)
	}
}

impl pallet_communities::Config for Test {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = WeightInfo;
	type CreateOrigin = EnsureNever<Deposit>;
	type AdminOrigin = EnsureCommunity<Self>;
	type MemberMgmtOrigin = EnsureCommunity<Self>;
	type MemberOrigin = EnsureSigned<AccountId>;
	type CommunityId = CommunityId;
	type MembershipId = MembershipId;
	type MemberMgmt = MembershipsManager;
	type Polls = Referenda;
	type CommunityOfClass = CommunityOfTrack;
	type Assets = Assets;
	type AssetsFreezer = AssetsFreezer;
	type Balances = Balances;
	type BlockNumberProvider = System;
	type PalletId = CommunitiesPalletId;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

impl Config for Test {
	// Types to support community creation
	type CreateCollection = Memberships;
	type Tracks = CommunityTracks;
	type RankedCollective = Collective;
	type WeightInfo = WeightInfo;
	type RegisterOrigin = EitherOf<
		EnsureRootWithSuccess<AccountId, NoDepositOnRootRegistration>,
		EnsureSignedPays<Test, RegistrationPayment, Treasury>,
	>;
	type MembershipsManagerAccount = MembershipsManagerAccountId;
	type CommunityStoragePart = StoragePart;
	type CommunityConsumptionPart = ConsumptionPart;
	type RuntimeHoldReason = RuntimeHoldReason;
	type GroupAdminOrigin = EnsureCommunity<Self>;
	type Issuer = MembershipsManager;
	type ItemAttributes = ClearNftsAttributes<Test>;
	type RetirementHolder = RetiringMembershipsAccount;
	type MembershipDeposit = MembershipDepositAmount;
	type RetirementDelay = RetirementDelayInBlocks;
	type MaxIssuePerCall = ConstU32<8>;
}

pub fn new_test_ext() -> TestExternalities {
	let mut t = TestExternalities::new(Default::default());
	t.execute_with(|| System::set_block_number(1));
	t
}
