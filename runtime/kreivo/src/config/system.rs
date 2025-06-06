//! System support stuff.

use super::*;

use cumulus_pallet_parachain_system::{DefaultCoreSelector, RelayNumberMonotonicallyIncreases};
use frame_contrib_traits::authn::{composite_authenticator, util::AuthorityFromPalletId, Challenge, Challenger};
use frame_support::traits::{AsEnsureOriginWithArg, LinearStoragePrice};
use frame_support::{
	derive_impl,
	dispatch::DispatchClass,
	traits::{fungible::HoldConsideration, Consideration, Footprint},
	PalletId,
};
use frame_system::{limits::BlockLength, EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::origin::AsSignedByCommunity;
use pallet_pass::FirstItemIsFree;
use parachains_common::{AVERAGE_ON_INITIALIZE_RATIO, NORMAL_DISPATCH_RATIO};
use polkadot_runtime_common::BlockHashCount;
use sp_core::{blake2_256, ConstU128};
use sp_runtime::{
	traits::{AccountIdConversion, LookupError, StaticLookup},
	DispatchError,
};

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
}

// #[runtime::pallet_index(1)]
// pub type ParachainSystem
parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
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

	fn generate(cx: &Self::Context, xtc: &impl ExtrinsicContext) -> Challenge {
		blake2_256(&[&System::block_hash(cx).0, xtc.as_ref()].concat())
	}

	fn check_challenge(cx: &Self::Context, xtc: &impl ExtrinsicContext, challenge: &[u8]) -> Option<()> {
		(*cx >= System::block_number().saturating_sub(PAST_BLOCKS)).then_some(())?;
		Self::generate(cx, xtc).eq(challenge).then_some(())
	}
}

pub type WebAuthn =
	pass_webauthn::Authenticator<BlockHashChallenger<{ 30 * MINUTES }>, AuthorityFromPalletId<PassPalletId>>;

composite_authenticator!(
	pub Pass<AuthorityFromPalletId<PassPalletId>> {
		WebAuthn,
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
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
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
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Authenticator = PassAuthenticator;
	#[cfg(feature = "runtime-benchmarks")]
	type Authenticator = benchmarks::PassAuthenticator;
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
	type DeviceConsideration = FirstItemIsFree<
		HoldConsideration<
			AccountId,
			Balances,
			AccountDevicesReason,
			LinearStoragePrice<ConstU128<MILLICENTS>, ConstU128<{ MILLICENTS / 10 }>, Balance>,
		>,
	>;
	type SessionKeyConsideration = FirstItemIsFree<
		HoldConsideration<
			AccountId,
			Balances,
			SessionKeysReason,
			LinearStoragePrice<ConstU128<MILLICENTS>, ConstU128<{ MILLICENTS / 10 }>, Balance>,
		>,
	>;
	type PalletId = PassPalletId;
	type MaxSessionDuration = ConstU32<{ 15 * MINUTES }>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmarks::PassBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;
	use frame_benchmarking::BenchmarkError;
	use sp_core::U256;

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

	pub type Dummy = frame_contrib_traits::authn::util::dummy::Dummy<AuthorityFromPalletId<PassPalletId>>;
	composite_authenticator!(
	pub Pass<AuthorityFromPalletId<PassPalletId>> {
			WebAuthn,
			Dummy,
		}
	);

	#[frame_support::storage_alias]
	type LastDeviceId = StorageValue<Pass, U256, frame_support::pallet_prelude::ValueQuery>;

	pub struct PassBenchmarkHelper;

	impl PassBenchmarkHelper {
		fn next_id() -> DeviceId {
			LastDeviceId::mutate(|id| {
				let device_id = id.clone();
				*id = device_id.saturating_add(U256::one());
				Decode::decode(&mut &id.encode()[..]).expect("U256 is encoded as [u8; 32]")
			})
		}
	}

	impl pallet_pass::BenchmarkHelper<Runtime> for PassBenchmarkHelper {
		fn device_attestation(_: &impl ExtrinsicContext) -> pallet_pass::DeviceAttestationOf<Runtime, ()> {
			PassDeviceAttestation::Dummy(frame_contrib_traits::authn::util::dummy::DummyAttestation::new(
				true,
				Self::next_id(),
			))
		}

		fn credential(
			user_id: HashedUserId,
			_: DeviceId,
			_: &impl ExtrinsicContext,
		) -> pallet_pass::CredentialOf<Runtime, ()> {
			PassCredential::Dummy(frame_contrib_traits::authn::util::dummy::DummyCredential::new(
				true, user_id,
			))
		}
	}
}
