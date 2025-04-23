use fc_traits_memberships::NonFungiblesMemberships;
use frame_support::{
	derive_impl,
	dispatch::DispatchResult,
	parameter_types,
	traits::{
		fungible::HoldConsideration, tokens::nonfungible_v2::ItemOf, AsEnsureOriginWithArg, ConstU32, ConstU64,
		EitherOf, EnsureOriginWithArg, EqualPrivilegeOnly, Footprint, VariantCountOf,
	},
	weights::{
		constants::{WEIGHT_REF_TIME_PER_NANOS, WEIGHT_REF_TIME_PER_SECOND},
		Weight,
	},
	PalletId,
};
use frame_system::{EnsureRoot, EnsureRootWithSuccess, EnsureSigned};
use pallet_referenda::{TrackIdOf, TrackInfoOf, TracksInfo};
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{Convert, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature, Perbill,
};
pub use virto_common::{CommunityId, MembershipId};

use crate::{
	self as pallet_communities,
	origin::{EnsureCommunity, EnsureSignedPays},
	types::{Tally, VoteWeight},
	Config, DecisionMethod,
};

// Weights constants

// max block: 0.5s compute with 12s average block time
pub const MAX_BLOCK_REF_TIME: u64 = WEIGHT_REF_TIME_PER_SECOND.saturating_div(2); // https://github.com/paritytech/cumulus/blob/98e68bd54257b4039a5d5b734816f4a1b7c83a9d/parachain-template/runtime/src/lib.rs#L221
pub const MAX_BLOCK_POV_SIZE: u64 = 5 * 1024 * 1024; // https://github.com/paritytech/polkadot/blob/ba1f65493d91d4ab1787af2fd6fe880f1da90586/primitives/src/v4/mod.rs#L384
pub const MAX_BLOCK_WEIGHT: Weight = Weight::from_parts(MAX_BLOCK_REF_TIME, MAX_BLOCK_POV_SIZE);

// max extrinsics: 75% of block
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/lib.rs#L218

// max extrinsic: max total extrinsics less average on_initialize ratio and less
// base extrinsic weight
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/lib.rs#L214
pub const BASE_EXTRINSIC: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(125_000), 0); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/weights/extrinsic_weights.rs#L26

type Block = frame_system::mocking::MockBlock<Test>;
type WeightInfo = ();

pub type AccountPublic = <MultiSignature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Balance = <Test as pallet_balances::Config>::Balance;
pub type AssetId = <Test as pallet_assets::Config>::AssetId;

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
	pub type Preimage = pallet_preimage;

	#[runtime::pallet_index(10)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(11)]
	pub type Assets = pallet_assets;
	#[runtime::pallet_index(12)]
	pub type AssetsFreezer = pallet_assets_freezer;

	#[runtime::pallet_index(21)]
	pub type Referenda = pallet_referenda;
	#[runtime::pallet_index(31)]
	pub type Communities = pallet_communities;
	#[runtime::pallet_index(32)]
	pub type Tracks = pallet_referenda_tracks;
	#[runtime::pallet_index(33)]
	pub type Nfts = pallet_nfts;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type AccountData = pallet_balances::AccountData<Balance>;
}

// Monetary operations
#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig as pallet_assets::DefaultConfig)]
impl pallet_assets::Config for Test {
	type Balance = Balance;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type Freezer = AssetsFreezer;
}

impl pallet_assets_freezer::Config for Test {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeEvent = RuntimeEvent;
}

// Memberships
#[cfg(feature = "runtime-benchmarks")]
pub struct NftsBenchmarksHelper;
#[cfg(feature = "runtime-benchmarks")]
impl pallet_nfts::BenchmarkHelper<CommunityId, MembershipId, AccountPublic, AccountId, MultiSignature>
	for NftsBenchmarksHelper
{
	fn collection(_i: u16) -> CommunityId {
		COMMUNITY
	}
	fn item(i: u16) -> MembershipId {
		i as MembershipId
	}
	fn signer() -> (AccountPublic, AccountId) {
		let public = sp_io::crypto::sr25519_generate(0.into(), None);
		let account = sp_runtime::MultiSigner::Sr25519(public).into_account();
		(public.into(), account)
	}
	fn sign(signer: &AccountPublic, message: &[u8]) -> MultiSignature {
		sp_runtime::MultiSignature::Sr25519(
			sp_io::crypto::sr25519_sign(0.into(), &signer.clone().try_into().unwrap(), message).unwrap(),
		)
	}
}

parameter_types! {
	pub const RootAccount: AccountId = AccountId::new([0xff; 32]);
}
impl pallet_nfts::Config for Test {
	type ApprovalsLimit = ();
	type AttributeDepositBase = ();
	type CollectionDeposit = ();
	type CollectionId = CommunityId;
	type CreateOrigin =
		AsEnsureOriginWithArg<EitherOf<EnsureRootWithSuccess<AccountId, RootAccount>, EnsureSigned<AccountId>>>;
	type Currency = ();
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
	type ValueLimit = ConstU32<10>;
	type WeightInfo = ();

	#[cfg(feature = "runtime-benchmarks")]
	type Helper = NftsBenchmarksHelper;
}

// Governance at Communities
parameter_types! {
	pub MaximumSchedulerWeight: Weight = Weight::from_parts(MAX_BLOCK_REF_TIME, MAX_BLOCK_POV_SIZE);
	pub const MaxScheduledPerBlock: u32 = 512;
}
pub struct ConvertDeposit;
impl Convert<Footprint, u64> for ConvertDeposit {
	fn convert(a: Footprint) -> u64 {
		a.count * 2 + a.size
	}
}

parameter_types! {
	pub PreimageHoldReason: RuntimeHoldReason = pallet_preimage::HoldReason::Preimage.into();
}

impl pallet_preimage::Config for Test {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureSigned<AccountId>;
	type Consideration = HoldConsideration<AccountId, Balances, PreimageHoldReason, ConvertDeposit>;
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
	type Preimages = Preimage;
}

pub struct EnsureOriginToTrack;
impl EnsureOriginWithArg<RuntimeOrigin, TrackIdOf<Test, ()>> for EnsureOriginToTrack {
	type Success = ();

	fn try_origin(o: RuntimeOrigin, id: &TrackIdOf<Test, ()>) -> Result<Self::Success, RuntimeOrigin> {
		let track_id_for_origin: TrackIdOf<Test, ()> = Tracks::track_for(&o.clone().caller).map_err(|_| o.clone())?;
		frame_support::ensure!(&track_id_for_origin == id, o);

		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(id: &TrackIdOf<Test, ()>) -> Result<RuntimeOrigin, ()> {
		Ok(pallet_communities::Origin::<Test>::new(id.clone()).into())
	}
}

#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::SaturatedConversion;

#[cfg(feature = "runtime-benchmarks")]
pub struct TracksBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_referenda_tracks::BenchmarkHelper<Test> for TracksBenchmarkHelper {
	fn track_id(id: u32) -> TrackIdOf<Test, ()> {
		id.saturated_into()
	}
}

parameter_types! {
	pub const MaxTracks: u32 = u32::MAX;
}
impl pallet_referenda_tracks::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TrackId = CommunityId;
	type MaxTracks = MaxTracks;
	type AdminOrigin = EnsureRoot<AccountId>;
	type UpdateOrigin = EnsureOriginToTrack;
	type WeightInfo = ();

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = TracksBenchmarkHelper;
}

parameter_types! {
		pub static AlarmInterval: u64 = 1;
}
impl pallet_referenda::Config for Test {
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = pallet_balances::Pallet<Self>;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
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
	type Preimages = Preimage;
}

// Communities

parameter_types! {
	pub const CommunitiesPalletId: PalletId = PalletId(*b"kv/comms");
	pub const MembershipsManagerCollectionId: CommunityId = 0;
	pub const MembershipNftAttr: &'static [u8; 10] = b"membership";
	pub const TestCommunity: CommunityId = COMMUNITY;
}

type MembershipCollection = ItemOf<Nfts, MembershipsManagerCollectionId, AccountId>;

#[cfg(feature = "runtime-benchmarks")]
use crate::{
	types::{AssetIdOf, CommunityIdOf, MembershipIdOf, PollIndexOf},
	BenchmarkHelper,
};

#[cfg(feature = "runtime-benchmarks")]
use {
	frame_benchmarking::BenchmarkError,
	frame_support::BoundedVec,
	frame_system::pallet_prelude::{OriginFor, RuntimeCallFor},
	pallet_referenda::{BoundedCallOf, Curve, PalletsOriginOf, TrackInfo},
	parity_scale_codec::Encode,
};

#[cfg(feature = "runtime-benchmarks")]
pub struct CommunityBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelper<Test> for CommunityBenchmarkHelper {
	fn community_id() -> CommunityIdOf<Test> {
		COMMUNITY
	}

	fn community_asset_id() -> AssetIdOf<Test> {
		1u32
	}

	fn community_desired_size() -> u32 {
		u8::MAX as u32
	}

	fn initialize_memberships_collection() -> Result<(), frame_benchmarking::BenchmarkError> {
		TestEnvBuilder::initialize_memberships_manager_collection()?;
		TestEnvBuilder::initialize_community_memberships_collection(&Self::community_id())?;
		Ok(())
	}

	fn issue_membership(
		community_id: CommunityIdOf<Test>,
		membership_id: MembershipIdOf<Test>,
	) -> Result<(), frame_benchmarking::BenchmarkError> {
		use frame_support::traits::tokens::nonfungible_v2::Mutate;

		let community_account = Communities::community_account(&community_id);
		MembershipCollection::mint_into(&membership_id, &community_account, &Default::default(), true)?;

		Ok(())
	}

	fn prepare_track(track_origin: PalletsOriginOf<Test>) -> Result<(), BenchmarkError> {
		let id = Self::community_id();
		let info = TrackInfo {
			name: sp_runtime::str_array("Community"),
			max_deciding: 1,
			decision_deposit: 5,
			prepare_period: 1,
			decision_period: 5,
			confirm_period: 1,
			min_enactment_period: 1,
			min_approval: Curve::LinearDecreasing {
				length: Perbill::from_percent(100),
				floor: Perbill::from_percent(50),
				ceil: Perbill::from_percent(100),
			},
			min_support: Curve::LinearDecreasing {
				length: Perbill::from_percent(100),
				floor: Perbill::from_percent(0),
				ceil: Perbill::from_percent(100),
			},
		};

		Tracks::insert(RuntimeOrigin::root(), id, info, track_origin.clone())?;

		Ok(())
	}

	fn prepare_poll(
		origin: OriginFor<Test>,
		proposal_origin: PalletsOriginOf<Test>,
		proposal_call: RuntimeCallFor<Test>,
	) -> Result<PollIndexOf<Test>, BenchmarkError> {
		let proposal = BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(proposal_call.encode()));
		let enactment_moment = frame_support::traits::schedule::DispatchTime::After(1);
		Referenda::submit(origin.clone(), Box::new(proposal_origin), proposal, enactment_moment)?;
		Referenda::place_decision_deposit(origin, 0)?;

		System::set_block_number(2);
		Referenda::nudge_referendum(RuntimeOrigin::root(), 0)?;

		Ok(0)
	}

	fn finish_poll(index: PollIndexOf<Test>) -> Result<(), BenchmarkError> {
		System::set_block_number(8);
		Referenda::nudge_referendum(RuntimeOrigin::root(), index)?;

		frame_support::assert_ok!(Referenda::ensure_ongoing(index));

		System::set_block_number(9);
		Referenda::nudge_referendum(RuntimeOrigin::root(), index)?;

		frame_support::assert_err!(
			Referenda::ensure_ongoing(index),
			pallet_referenda::Error::<Test, ()>::NotOngoing
		);

		Ok(())
	}
}

parameter_types! {
	pub const NoPay: Option<(Balance, AccountId, AccountId)> = None;
}
type RootCreatesCommunitiesForFree = EnsureRootWithSuccess<AccountId, NoPay>;
type AnyoneElsePays = EnsureSignedPays<Test, ConstU64<10>, RootAccount>;

pub type MembershipsManager = NonFungiblesMemberships<Nfts>;

impl Config for Test {
	type PalletId = CommunitiesPalletId;
	type CommunityId = CommunityId;
	type MembershipId = MembershipId;

	type Assets = Assets;
	type AssetsFreezer = AssetsFreezer;
	type Balances = Balances;
	type ItemConfig = pallet_nfts::ItemConfig;
	type MemberMgmt = MembershipsManager;
	type Polls = Referenda;

	type CreateOrigin = EitherOf<RootCreatesCommunitiesForFree, AnyoneElsePays>;
	type AdminOrigin = EnsureCommunity<Self>;
	type MemberMgmtOrigin = EnsureCommunity<Self>;

	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = WeightInfo;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

pub const COMMUNITY: CommunityId = 1;
pub const COMMUNITY_ORIGIN: OriginCaller =
	OriginCaller::Communities(pallet_communities::Origin::<Test>::new(COMMUNITY));

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(members: &[AccountId], memberships: &[MembershipId]) -> sp_io::TestExternalities {
	TestEnvBuilder::new()
		.add_community(COMMUNITY, DecisionMethod::Membership, members, memberships, None)
		.build()
}

#[derive(Default)]
pub(crate) struct TestEnvBuilder {
	assets_config: AssetsConfig,
	balances: Vec<(AccountId, Balance)>,
	communities: Vec<CommunityId>,
	decision_methods: sp_std::collections::btree_map::BTreeMap<CommunityId, DecisionMethod<AssetId, Balance>>,
	members: Vec<(CommunityId, AccountId)>,
	memberships: Vec<(CommunityId, MembershipId)>,
	tracks: Vec<(TrackIdOf<Test, ()>, TrackInfoOf<Test>)>,
}

impl TestEnvBuilder {
	pub(crate) fn new() -> Self {
		Self::default()
	}

	pub(crate) fn add_asset(
		mut self,
		id: &AssetId,
		owner: &AccountId,
		is_sufficient: bool,
		min_balance: Balance,
		// name, symbol, decimals
		maybe_metadata: Option<(Vec<u8>, Vec<u8>, u8)>,
		maybe_accounts: Option<Vec<(AccountId, Balance)>>,
	) -> Self {
		self.assets_config
			.assets
			.push((*id, owner.clone(), is_sufficient, min_balance));

		if let Some((name, symbol, decimals)) = maybe_metadata {
			self.assets_config.metadata.push((*id, name, symbol, decimals));
		}

		self.assets_config.accounts.append(
			&mut maybe_accounts
				.unwrap_or_default()
				.into_iter()
				.map(|(account_id, balance)| (*id, account_id, balance))
				.collect(),
		);

		self
	}

	pub(crate) fn add_community(
		mut self,
		community_id: CommunityId,
		decision_method: DecisionMethod<AssetId, Balance>,
		members: &[AccountId],
		memberships: &[MembershipId],
		maybe_track: Option<TrackInfoOf<Test>>,
	) -> Self {
		self.communities.push(community_id);
		self.decision_methods.insert(community_id, decision_method);
		self.members
			.append(&mut members.iter().map(|m| (community_id, m.to_owned())).collect::<Vec<_>>());
		self.memberships.append(
			&mut memberships
				.iter()
				.map(|m| (community_id, m.to_owned()))
				.collect::<Vec<_>>(),
		);
		if let Some(track) = maybe_track {
			self.tracks.push((community_id, track));
		}

		self
	}

	pub(crate) fn with_balances(mut self, balances: &[(AccountId, Balance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let t = RuntimeGenesisConfig {
			assets: self.assets_config,
			balances: pallet_balances::GenesisConfig {
				balances: self.balances,
			},
			system: Default::default(),
		}
		.build_storage()
		.unwrap();

		let mut ext = TestExternalities::new(t);

		ext.execute_with(|| {
			System::set_block_number(1);

			Self::initialize_memberships_manager_collection().expect("collection is initialized");

			for community_id in &self.communities {
				Self::initialize_community_memberships_collection(community_id).expect("collection is initialized");

				let decision_method = self
					.decision_methods
					.get(community_id)
					.expect("should include decision_method on add_community");
				let community_origin: RuntimeOrigin = Self::create_community_origin(community_id);

				Communities::create(RuntimeOrigin::root(), community_origin.caller.clone(), *community_id)
					.expect("can add community");

				Communities::set_decision_method(community_origin.clone(), *community_id, decision_method.clone())
					.expect("can set decision info");

				let mut members = self.members.iter().filter(|(cid, _)| cid == community_id);
				let memberships = self.memberships.iter().filter(|(cid, _)| cid == community_id);

				assert!(
					self.memberships.len() >= self.members.len(),
					"there should be at least as many memberships as there are members"
				);

				for (_, membership) in memberships {
					use frame_support::traits::tokens::nonfungible_v2::Mutate;

					let account = Communities::community_account(community_id);
					MembershipCollection::mint_into(membership, &account, &Default::default(), true)
						.expect("can mint membership");

					if let Some((_, who)) = members.next() {
						Communities::add_member(community_origin.clone(), who.clone()).expect("can add member");
					}
				}

				for (_, track_info) in self.tracks.iter().filter(|(cid, _)| cid == community_id) {
					Tracks::insert(
						RuntimeOrigin::root(),
						*community_id,
						track_info.clone(),
						community_origin.caller.clone(),
					)
					.expect("can add track");
				}
			}
		});

		ext
	}

	pub(crate) fn initialize_memberships_manager_collection() -> DispatchResult {
		Nfts::do_create_collection(
			MembershipsManagerCollectionId::get(),
			RootAccount::get(),
			RootAccount::get(),
			Default::default(),
			0,
			pallet_nfts::Event::ForceCreated {
				collection: MembershipsManagerCollectionId::get(),
				owner: RootAccount::get(),
			},
		)
	}

	pub(crate) fn initialize_community_memberships_collection(community_id: &CommunityId) -> DispatchResult {
		let account = Communities::community_account(community_id);
		Nfts::do_create_collection(
			*community_id,
			account.clone(),
			account.clone(),
			Default::default(),
			0,
			pallet_nfts::Event::ForceCreated {
				collection: *community_id,
				owner: account,
			},
		)
	}

	pub fn create_community_origin(community_id: &CommunityId) -> RuntimeOrigin {
		pallet_communities::Origin::<Test>::new(*community_id).into()
	}
}
