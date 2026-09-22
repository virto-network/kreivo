//! System support stuff.

use super::*;

use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use frame_contrib_traits::authn::{composite_authenticator, util::AuthorityFromPalletId, Challenge, Challenger};
use frame_support::traits::{AsEnsureOriginWithArg, LinearStoragePrice};
use frame_support::{
	derive_impl,
	dispatch::DispatchClass,
	traits::{fungible::HoldConsideration, Consideration, Footprint},
	weights::constants::{BlockExecutionWeight, ExtrinsicBaseWeight},
	PalletId,
};
use frame_system::{limits::BlockLength, EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::origin::AsSignedByCommunity;
use pallet_pass::FirstItemsAreFree;
use parachains_common::{AVERAGE_ON_INITIALIZE_RATIO, NORMAL_DISPATCH_RATIO};
use polkadot_runtime_common::BlockHashCount;
pub use runtime_constants::async_backing_params::{BLOCK_PROCESSING_VELOCITY, RELAY_PARENT_OFFSET};
use sp_core::ConstU128;
use sp_runtime::{
	traits::{AccountIdConversion, LookupError, StaticLookup},
	DispatchError,
};

/// BLAKE2-256. `sp_core` no longer re-exports it, and `sp-io` is optional here.
fn blake2_256(data: &[u8]) -> [u8; 32] {
	<frame_support::Blake2_256 as frame_support::StorageHasher>::hash(data)
}

/// Blocks are at most 5 MiB; `Normal` extrinsics get `NORMAL_DISPATCH_RATIO` of it.
pub(crate) const MAX_BLOCK_LENGTH: u32 = 5 * 1024 * 1024;

// #[runtime::pallet_index(0)]
// pub type System

/// Parachain blocks we aim to build per relay chain slot. With block bundling, a core's
/// execution time and PoV are split among the blocks it carries.
pub type TargetBlockRate = ConstU32<BLOCK_PROCESSING_VELOCITY>;

/// The weight a block may use: the share of a core it gets, given the cores the parachain has
/// and [`TargetBlockRate`]. The first block of a core may take the whole core when it needs
/// to (e.g. a runtime upgrade), which the [`DynamicMaxBlockWeight`] extension arranges.
///
/// [`DynamicMaxBlockWeight`]: cumulus_pallet_parachain_system::block_weight::DynamicMaxBlockWeight
pub type MaximumBlockWeight =
	cumulus_pallet_parachain_system::block_weight::MaxParachainBlockWeight<Runtime, TargetBlockRate>;

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength = BlockLength::builder()
		.max_length(MAX_BLOCK_LENGTH)
		.modify_max_length_for_class(DispatchClass::Normal, |max| *max = NORMAL_DISPATCH_RATIO * MAX_BLOCK_LENGTH)
		.build();
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MaximumBlockWeight::get());
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MaximumBlockWeight::get());
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MaximumBlockWeight`.
			weights.reserved = Some(
				MaximumBlockWeight::get() - NORMAL_DISPATCH_RATIO * MaximumBlockWeight::get()
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

#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig)]
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
	type MaxConsumers = ConstU32<16>;
	type SystemWeightInfo = weights::frame_system::WeightInfo<Self>;
	// Sets the block's weight mode (a share of the core, or the whole core) before inherents.
	type PreInherents =
		cumulus_pallet_parachain_system::block_weight::DynamicMaxBlockWeightHooks<Runtime, TargetBlockRate>;
}

// #[runtime::pallet_index(1)]
// pub type ParachainSystem
parameter_types! {
	pub ReservedXcmpWeight: Weight = MaximumBlockWeight::get().saturating_div(4);
	pub ReservedDmpWeight: Weight = MaximumBlockWeight::get().saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

pub type RelaychainData = cumulus_pallet_parachain_system::RelaychainDataProvider<Runtime>;

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
	type RelayParentOffset = ConstU32<RELAY_PARENT_OFFSET>;
	// V3 candidate scheduling stays disabled until collators and the relay chain support it.
	type SchedulingSignatureVerifier = ();
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

	fn generate(cx: &Self::Context, xtc: &impl ExtrinsicContext) -> Challenge {
		log::trace!(target: "authn", "BlockHashChallenger::generate({cx:?}, {:?})", xtc.as_ref());
		log::trace!(target: "authn", "\t -> ({:?}",
			blake2_256(&[&System::block_hash(cx).0, xtc.as_ref()].concat()));
		blake2_256(&[&System::block_hash(cx).0, xtc.as_ref()].concat())
	}

	fn check_challenge(cx: &Self::Context, xtc: &impl ExtrinsicContext, challenge: &[u8]) -> Option<()> {
		(*cx >= System::block_number().saturating_sub(PAST_BLOCKS)).then_some(())?;
		Self::generate(cx, xtc).eq(challenge).then_some(())
	}
}

// Challenges are parachain block hashes, so their lifetime is in parachain blocks.
pub type KreivoChallenger = BlockHashChallenger<{ 30 * runtime_constants::time::parachain::MINUTES }>;
pub type WebAuthn = pass_webauthn::Authenticator<KreivoChallenger, AuthorityFromPalletId<PassPalletId>>;
pub type SubstrateKey = pass_substrate_keys::Authenticator<KreivoChallenger, AuthorityFromPalletId<PassPalletId>>;

composite_authenticator!(
	pub Pass<AuthorityFromPalletId<PassPalletId>> {
		WebAuthn,
		SubstrateKey,
	}
);

#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo)]
pub struct SkipConsideration<C>(Option<C>);

const ACCOUNT_IS_ROOT: fn(&AccountId) -> bool = |acct| acct == &TreasuryAccount::get();
const ACCOUNT_IS_COMMUNITY: fn(&AccountId) -> bool = |acct| {
	PalletId::try_from_sub_account::<CommunityId>(acct)
		.is_some_and(|(id, _)| id == communities::CommunityPalletId::get())
};

impl<C> Consideration<AccountId, Footprint> for SkipConsideration<C>
where
	C: Consideration<AccountId, Footprint>,
{
	fn new(who: &AccountId, new: Footprint) -> Result<Self, DispatchError> {
		if ACCOUNT_IS_ROOT(who) || ACCOUNT_IS_COMMUNITY(who) {
			Ok(Self(None))
		} else {
			C::new(who, new).map(Some).map(Self)
		}
	}

	fn update(self, who: &AccountId, new: Footprint) -> Result<Self, DispatchError> {
		if let Some(c) = self.0 {
			c.update(who, new).map(Some).map(Self)
		} else {
			Ok(self)
		}
	}

	fn drop(self, who: &AccountId) -> Result<(), DispatchError> {
		if let Some(c) = self.0 {
			c.drop(who)
		} else {
			Ok(())
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(who: &AccountId, new: Footprint) {
		C::ensure_successful(who, new);
	}
}
parameter_types! {
	pub AccountRegistrationReason: RuntimeHoldReason = RuntimeHoldReason::Pass(pallet_pass::HoldReason::AccountRegistration);
	pub AccountDevicesReason: RuntimeHoldReason = RuntimeHoldReason::Pass(pallet_pass::HoldReason::AccountDevices);
	pub SessionKeysReason: RuntimeHoldReason = RuntimeHoldReason::Pass(pallet_pass::HoldReason::SessionKeys);
}

impl pallet_pass::Config for Runtime {
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_pass::WeightInfo<Self>;
	type RegisterOrigin = EitherOf<
		// Root can create pass accounts.
		EnsureRootWithSuccess<Self::AccountId, TreasuryAccount>,
		EitherOf<
			// Communities can create pass accounts.
			AsEnsureOriginWithArg<AsSignedByCommunity<Runtime>>,
			// Anyone can create pass accounts.
			AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>,
		>,
	>;
	type AddressGenerator = ();
	type Balances = Balances;
	type Authenticator = PassAuthenticator;
	type Scheduler = Scheduler;
	type BlockNumberProvider = RelaychainData;
	type RegistrarConsideration = SkipConsideration<
		HoldConsideration<
			AccountId,
			Balances,
			AccountRegistrationReason,
			LinearStoragePrice<ConstU128<EXISTENTIAL_DEPOSIT>, ConstU128<MILLICENTS>, Balance>,
		>,
	>;
	// The first two devices and session keys are free, e.g. a phone and a laptop.
	// `FirstItemsAreFree` keeps the stored ticket as `Option<C>`, the same as `FirstItemIsFree`,
	// so existing `DeviceConsiderations`/`SessionKeyConsiderations` entries still decode.
	type DeviceConsideration = FirstItemsAreFree<
		ConstU32<2>,
		HoldConsideration<
			AccountId,
			Balances,
			AccountDevicesReason,
			LinearStoragePrice<ConstU128<MILLICENTS>, ConstU128<{ MILLICENTS / 10 }>, Balance>,
		>,
	>;
	type SessionKeyConsideration = FirstItemsAreFree<
		ConstU32<2>,
		HoldConsideration<
			AccountId,
			Balances,
			SessionKeysReason,
			LinearStoragePrice<ConstU128<MILLICENTS>, ConstU128<{ MILLICENTS / 10 }>, Balance>,
		>,
	>;
	type PalletId = PassPalletId;
	type MaxSessionDuration = ConstU32<{ 15 * MINUTES }>;
	type MaxDevicesPerAccount = ConstU32<100>;
	type MaxSessionsPerAccount = ConstU32<10>;
}

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarks {
	use super::*;
	use frame_benchmarking::BenchmarkError;

	impl frame_system_benchmarking::Config for Runtime {
		fn setup_set_code_requirements(code: &Vec<u8>) -> Result<(), BenchmarkError> {
			ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
			Ok(())
		}

		fn verify_set_code() {
			System::assert_last_event(
				cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into(),
			);
		}
	}
}

/// `pallet-pass` benchmarks get their inputs from the authenticators (WebAuthn, the first in the
/// `Pass` composite); they only need a context that the challenger accepts.
#[cfg(feature = "runtime-benchmarks")]
impl<const PAST_BLOCKS: BlockNumber> frame_contrib_traits::authn::ChallengerBenchmarkHelper
	for BlockHashChallenger<PAST_BLOCKS>
{
	fn benchmark_context() -> Self::Context {
		System::block_number()
	}
}
