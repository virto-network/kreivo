//! System support stuff.

use super::*;

use frame_support::{derive_impl, dispatch::DispatchClass, traits::EnsureOrigin, PalletId};
use frame_system::{limits::BlockLength, EnsureRootWithSuccess};
use sp_runtime::traits::{LookupError, StaticLookup};

use cumulus_pallet_parachain_system::{DefaultCoreSelector, RelayNumberMonotonicallyIncreases};
use parachains_common::{AVERAGE_ON_INITIALIZE_RATIO, NORMAL_DISPATCH_RATIO};
use polkadot_runtime_common::BlockHashCount;

use frame_contrib_traits::authn::{composite_authenticator, util::AuthorityFromPalletId, Challenge, Challenger};
use pallet_communities::origin::AsSignedByCommunity;

// #[runtime::pallet_index(0)]
// pub type System
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	sp_weights::constants::WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 2;
}

pub struct CommunityLookup;
impl StaticLookup for CommunityLookup {
	type Source = Address;
	type Target = AccountId;
	fn lookup(s: Self::Source) -> Result<Self::Target, LookupError> {
		match s {
			MultiAddress::Id(i) => Ok(i),
			MultiAddress::Index(i) => Ok(Communities::community_account(&i)),
			_ => Err(LookupError),
		}
	}
	fn unlookup(t: Self::Target) -> Self::Source {
		MultiAddress::Id(t)
	}
}

#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	type Lookup = CommunityLookup;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	type Block = Block;
	type Nonce = Nonce;
	/// Maximum number of block number to block hash mappings to keep (oldest
	/// pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate
	/// prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SystemWeightInfo = weights::frame_system::WeightInfo<Self>;
}

// #[runtime::pallet_index(1)]
// pub type ParachainSystem
parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type WeightInfo = weights::cumulus_pallet_parachain_system::WeightInfo<Self>;
	type ConsensusHook = ConsensusHook;
	type SelectCore = DefaultCoreSelector<Self>;
}

// #[runtime::pallet_index(2)]
// pub type Timestamp
impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<0>;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Self>;
}

// #[runtime::pallet_index(3)]
// pub type ParachainInfo
impl parachain_info::Config for Runtime {}

// #[runtime::pallet_index(4)]
// pub type Origins
impl pallet_custom_origins::Config for Runtime {}

// #[runtime::pallet_index(6)]
// pub type Pass
parameter_types! {
	pub PassPalletId: PalletId = PalletId(*b"kreivo_p");
	pub NeverPays: Option<pallet_pass::DepositInformation<Runtime>> = None;
}

/// A [`Challenger`][`frame_contrib_traits::authn::Challenger`] which verifies
/// the block hash of a block of a given block that's within the last
/// `PAST_BLOCKS`.
pub struct BlockHashChallenger<const PAST_BLOCKS: BlockNumber>;

impl<const PAST_BLOCKS: BlockNumber> Challenger for BlockHashChallenger<PAST_BLOCKS> {
	type Context = BlockNumber;

	fn generate(cx: &Self::Context) -> Challenge {
		System::block_hash(cx).0
	}

	fn check_challenge(cx: &Self::Context, challenge: &[u8]) -> Option<()> {
		(*cx >= System::block_number().saturating_sub(PAST_BLOCKS)).then_some(())?;
		Self::generate(cx).eq(challenge).then_some(())
	}
}

pub type WebAuthn =
	pass_webauthn::Authenticator<BlockHashChallenger<{ 30 * MINUTES }>, AuthorityFromPalletId<PassPalletId>>;
#[cfg(feature = "runtime-benchmarks")]
pub type Dummy = frame_contrib_traits::authn::util::dummy::Dummy<AuthorityFromPalletId<PassPalletId>>;

#[cfg(not(feature = "runtime-benchmarks"))]
composite_authenticator!(
	pub Pass<AuthorityFromPalletId<PassPalletId>> {
		WebAuthn,
	}
);

#[cfg(feature = "runtime-benchmarks")]
composite_authenticator!(
	pub Pass<AuthorityFromPalletId<PassPalletId>> {
		WebAuthn,
		Dummy,
	}
);

/// Communities don't need to pay deposit fees to create a `pass` account
pub struct CommunitiesDontDeposit;

impl<OuterOrigin> EnsureOriginWithArg<OuterOrigin, HashedUserId> for CommunitiesDontDeposit
where
	OuterOrigin: frame_support::traits::OriginTrait
		+ From<frame_system::RawOrigin<AccountId>>
		+ From<pallet_communities::Origin<Runtime>>
		+ Clone
		+ Into<Result<frame_system::RawOrigin<AccountId>, OuterOrigin>>
		+ Into<Result<pallet_communities::Origin<Runtime>, OuterOrigin>>,
{
	type Success = Option<pallet_pass::DepositInformation<Runtime>>;

	fn try_origin(o: OuterOrigin, _: &HashedUserId) -> Result<Self::Success, OuterOrigin> {
		AsSignedByCommunity::<Runtime>::try_origin(o)?;
		Ok(None)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_: &HashedUserId) -> Result<OuterOrigin, ()> {
		use pallet_communities::BenchmarkHelper;
		let community_id = communities::CommunityBenchmarkHelper::community_id();
		Ok(
			frame_system::RawOrigin::Signed(pallet_communities::Pallet::<Runtime>::community_account(&community_id))
				.into(),
		)
	}
}

impl pallet_pass::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type WeightInfo = weights::pallet_pass::WeightInfo<Self>;
	type Authenticator = PassAuthenticator; // WebAuthn;
	type PalletsOrigin = OriginCaller;
	type PalletId = PassPalletId;
	type MaxSessionDuration = ConstU32<{ 15 * MINUTES }>;
	type RegisterOrigin = EitherOf<
		// Root never pays
		EnsureRootWithSuccess<Self::AccountId, NeverPays>,
		EitherOf<
			// 	// Communities never pay
			CommunitiesDontDeposit,
			// Signed users must deposit ED for creating a pass account
			pallet_pass::EnsureSignedPays<
				Runtime,
				<Runtime as pallet_balances::Config>::ExistentialDeposit,
				TreasuryAccount,
			>,
		>,
	>;
	type Scheduler = Scheduler;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = PassBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct PassBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_pass::BenchmarkHelper<Runtime> for PassBenchmarkHelper {
	fn register_origin() -> frame_system::pallet_prelude::OriginFor<Runtime> {
		RuntimeOrigin::root()
	}

	fn device_attestation(_: DeviceId) -> pallet_pass::DeviceAttestationOf<Runtime, ()> {
		PassDeviceAttestation::Dummy(frame_contrib_traits::authn::util::dummy::DummyAttestation::new(true))
	}

	fn credential(_: HashedUserId) -> pallet_pass::CredentialOf<Runtime, ()> {
		PassCredential::Dummy(frame_contrib_traits::authn::util::dummy::DummyCredential::new(true))
	}
}
