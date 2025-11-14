use super::{
	AccountId, AllPalletsWithSystem, Assets, Balance, Balances, FungibleAssetLocation, KreivoAssetsInstance,
	ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeHoldReason, RuntimeOrigin,
	Treasury, TreasuryAccount, WeightToFee, XcmpQueue,
};
use virto_common::AsFungibleAssetLocation;

use crate::constants::locations::ASSET_HUB_ID;
use core::marker::PhantomData;
use frame_support::traits::fungible::HoldConsideration;
use frame_support::traits::LinearStoragePrice;
use frame_support::{
	parameter_types,
	traits::{
		tokens::imbalance::ResolveTo, ConstU32, Contains, ContainsPair, Everything, Get, Nothing, PalletInfoAccess,
	},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use parachains_common::xcm_config::AssetFeeAsExistentialDepositMultiplier;
use polkadot_parachain_primitives::primitives::Sibling;
use sp_runtime::traits::ConvertInto;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowTopLevelPaidExecutionFrom, ConvertedConcreteId,
	EnsureXcmOrigin, FrameTransactionalProcessor, FungibleAdapter, FungiblesAdapter, IsConcrete, LocalMint,
	MintLocation, NativeAsset, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation,
	StartsWith, TakeWeightCredit, UsingComponents, WeightInfoBounds, WithComputedOrigin,
};
use xcm_executor::traits::JustTry;
use xcm_executor::XcmExecutor;

mod communities;
mod with_external_assets;

use communities::*;

#[cfg(not(feature = "paseo"))]
parameter_types! {
	pub const RelayNetwork: Option<NetworkId> = Some(Kusama);
}
#[cfg(feature = "paseo")]
parameter_types! {
	pub const RelayNetwork: Option<NetworkId> = Some(Polkadot);
}

parameter_types! {
	pub const RelayLocation: Location = Location::parent();
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub CheckAccount: (AccountId, MintLocation) = (PolkadotXcm::check_account(), MintLocation::Local);
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub AssetsPalletLocation: Location =
		PalletInstance(<Assets as PalletInfoAccess>::index() as u8).into();
	pub UniversalLocation: InteriorLocation = [
		GlobalConsensus(Polkadot),
		GlobalConsensus(Kusama),
		Parachain(ParachainInfo::parachain_id().into()),
	].into();

}

/// Type for specifying how a `Location` can be converted into an
/// `AccountId`. This is used when determining ownership of accounts for asset
/// transacting and when attempting to use XCM `Transact` in order to determine
/// the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Here (Parachain) origin converts to a given `AccountId`.
	HereConvertsTo<TreasuryAccount>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Plurality origins convert to community AccountId via the `Communities::community_account`.
	PluralityConvertsToCommunityAccountId,
	// For incoming relay `Account32` origins, alias directly to `AccountId`.
	AccountId32FromRelay<RelayNetwork, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type LocationConvertedConcreteId = xcm_builder::MatchedConvertedConcreteId<
	FungibleAssetLocation,
	Balance,
	(StartsWith<AssetHubLocation>, StartsWith<Dot>),
	AsFungibleAssetLocation,
	JustTry,
>;

/// Means for transacting the native currency on this chain.
pub type FungibleTransactor = FungibleAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Convert an XCM Location into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Balances`.
	CheckAccount,
>;

/// Means for transacting assets besides the native currency on this chain.
pub type FungiblesTransactor = with_external_assets::FungiblesAdapterForExternalAssets<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a registered fungible asset matching the given location or name
	// Assets not found in AssetRegistry will not be used
	ConvertedConcreteId<FungibleAssetLocation, Balance, AsFungibleAssetLocation, JustTry>,
	// Convert an XCM Location into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We only want to allow teleports of known assets. We use non-zero issuance as an indication
	// that this asset is known.
	LocalMint<parachains_common::impls::NonZeroIssuance<AccountId, Assets>>,
	// The account to use for tracking teleports.
	CheckingAccount,
	// The account who owns the newly created assets
	TreasuryAccount,
>;

/// This is the type we use to convert an (incoming) XCM origin into a local
/// `Origin` instance, ready for dispatching a transaction with Xcm's
/// `Transact`. There is an `OriginKind` which can biases the kind of local
/// `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub XcmAssetFeesReceiver: AccountId = Treasury::account_id();
}

pub struct ParentOrParentsExecutivePlurality;
impl Contains<Location> for ParentOrParentsExecutivePlurality {
	fn contains(t: &Location) -> bool {
		matches!(
			t.unpack(),
			(1, [])
				| (
					1,
					[Plurality {
						id: BodyId::Executive,
						..
					}],
				),
		)
	}
}

pub type Barrier = (
	TakeWeightCredit,
	WithComputedOrigin<
		(
			AllowTopLevelPaidExecutionFrom<Everything>,
			AllowExplicitUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
			// ^^^ Parent and its exec plurality get free execution
		),
		UniversalLocation,
		ConstU32<8>,
	>,
);

pub type AssetTransactors = (FungibleTransactor, FungiblesTransactor);

parameter_types! {
	pub AssetHubLocation: Location = Location::new(1, [Parachain(ASSET_HUB_ID)]);
	pub Ksm: Location = Location::new(1, Here);
	pub Dot: Location = Location::new(2, [GlobalConsensus(Polkadot)]);
}

//- From PR https://github.com/paritytech/cumulus/pull/936
fn matches_prefix(prefix: &Location, loc: &Location) -> bool {
	prefix.parent_count() == loc.parent_count()
		&& loc.len() >= prefix.len()
		&& prefix
			.interior()
			.iter()
			.zip(loc.interior().iter())
			.all(|(prefix_junction, junction)| prefix_junction == junction)
}
pub struct ReserveAssetsFrom<O>(PhantomData<O>);
impl<Origin: Get<Location>> ContainsPair<Asset, Location> for ReserveAssetsFrom<Origin> {
	fn contains(asset: &Asset, origin: &Location) -> bool {
		log::trace!(target: "xcm_config::ReserveAssetsFrom", "origin ({origin:?}) should be {:?}, and asset ({asset:?}) match prefix with the origin", Origin::get());
		&Origin::get() == origin && matches_prefix(&Origin::get(), &asset.id.0)
	}
}

pub struct ReserveAssetInOrigin<I, L>(PhantomData<(I, L)>);
impl<Id: Get<Location>, Origin: Get<Location>> ContainsPair<Asset, Location> for ReserveAssetInOrigin<Id, Origin> {
	fn contains(asset: &Asset, origin: &Location) -> bool {
		log::trace!(target: "xcm_config::ReserveAssetInOrigin", "origin ({origin:?}) should be {:?}, and asset ({asset:?}) should be {:?}", Origin::get(), Id::get());
		&Origin::get() == origin && matches!(&asset.id, AssetId(ref id) if id == &Id::get())
	}
}

pub type AssetFeeAsExistentialDepositMultiplierFeeCharger = AssetFeeAsExistentialDepositMultiplier<
	Runtime,
	WeightToFee,
	pallet_assets::BalanceToAssetBalance<Balances, Runtime, ConvertInto, KreivoAssetsInstance>,
	KreivoAssetsInstance,
>;

pub type Traders = (
	cumulus_primitives_utility::TakeFirstAssetTrader<
		AccountId,
		AssetFeeAsExistentialDepositMultiplierFeeCharger,
		LocationConvertedConcreteId,
		Assets,
		cumulus_primitives_utility::XcmFeesTo32ByteAccount<FungiblesTransactor, AccountId, XcmAssetFeesReceiver>,
	>,
	// Everything else
	UsingComponents<WeightToFee, RelayLocation, AccountId, Balances, ResolveTo<TreasuryAccount, Balances>>,
);

pub type Reserves = (
	NativeAsset,
	ReserveAssetsFrom<AssetHubLocation>,
	ReserveAssetInOrigin<Ksm, AssetHubLocation>,
	ReserveAssetInOrigin<Dot, AssetHubLocation>,
);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type XcmEventEmitter = PolkadotXcm;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = Reserves;
	// Teleporting is disabled.
	type IsTeleporter = ();
	type Aliasers = Nothing;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = WeightInfoBounds<crate::weights::xcm::KreivoXcmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
	type Trader = Traders;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetLocker = ();
	type AssetExchanger = ();
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = ();
}

/// Only communities are allowed to dispatch xcm messages
pub type CanSendXcmMessages = (
	pallet_communities::Origin<Runtime>,
	SignedByCommunityToPlurality<Runtime>,
);

/// Only signed origins are allowed to execute xcm transactions
pub type CanExecuteXcmTransactions = (
	pallet_communities::Origin<Runtime>,
	SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>,
);

/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

parameter_types! {
	pub const DepositPerItem: Balance = crate::deposit(1, 0);
	pub const DepositPerByte: Balance = crate::deposit(0, 1);
	pub const AuthorizeAliasHoldReason: RuntimeHoldReason = RuntimeHoldReason::PolkadotXcm(pallet_xcm::HoldReason::AuthorizeAlias);
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type AuthorizedAliasConsideration = HoldConsideration<
		AccountId,
		Balances,
		AuthorizeAliasHoldReason,
		LinearStoragePrice<DepositPerItem, DepositPerByte, Balance>,
	>;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, CanSendXcmMessages>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, CanExecuteXcmTransactions>;
	type XcmExecuteFilter = Nothing;
	// ^ Disable dispatchable execute on the XCM pallet.
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = WeightInfoBounds<crate::weights::xcm::KreivoXcmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;

	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type AdminOrigin = EnsureRoot<AccountId>;
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type WeightInfo = pallet_xcm::TestWeightInfo;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;

	use crate::{
		config::{ExistentialDeposit, PriceForParentDelivery},
		vec, Vec, UNITS,
	};
	use frame_benchmarking::BenchmarkError;
	use pallet_xcm_benchmarks::asset_instance_from;
	use xcm::prelude::Assets as XcmAssets;

	parameter_types! {
		pub ExistentialDepositAsset: Option<Asset> = Some((
			RelayLocation::get(),
			ExistentialDeposit::get()
		).into());
	}

	impl pallet_xcm_benchmarks::Config for Runtime {
		type XcmConfig = XcmConfig;
		type AccountIdConverter = LocationToAccountId;
		type DeliveryHelper = cumulus_primitives_utility::ToParentDeliveryHelper<
			XcmConfig,
			ExistentialDepositAsset,
			PriceForParentDelivery,
		>;

		fn valid_destination() -> Result<Location, BenchmarkError> {
			Ok(RelayLocation::get())
		}

		fn worst_case_holding(depositable_count: u32) -> XcmAssets {
			// A mix of fungible, non-fungible, and concrete assets.
			let holding_non_fungibles = MaxAssetsIntoHolding::get() / 2 - depositable_count;
			let holding_fungibles = holding_non_fungibles.saturating_sub(1);
			let fungibles_amount: u128 = 100;

			(0..holding_fungibles)
				.map(|i| {
					Asset {
						id: AssetId(GeneralIndex(i as u128).into()),
						fun: Fungible(fungibles_amount * (i + 1) as u128), // non-zero amount
					}
				})
				.chain(core::iter::once(Asset {
					id: AssetId(Here.into()),
					fun: Fungible(u128::MAX),
				}))
				.chain(core::iter::once(Asset {
					id: AssetId(RelayLocation::get()),
					fun: Fungible(1_000_000 * UNITS),
				}))
				.chain((0..holding_non_fungibles).map(|i| Asset {
					id: AssetId(GeneralIndex(i as u128).into()),
					fun: NonFungible(asset_instance_from(i)),
				}))
				.collect::<Vec<_>>()
				.into()
		}
	}

	parameter_types! {
		pub const TrustedTeleporter: Option<(Location, Asset)> = Some((
			RelayLocation::get(),
			Asset { fun: Fungible(UNITS), id: AssetId(RelayLocation::get()) },
		));
		pub const CheckedAccount: Option<(AccountId, MintLocation)> = None;
		pub const TrustedReserve: Option<(Location, Asset)> = None;

	}

	impl pallet_xcm_benchmarks::fungible::Config for Runtime {
		type TransactAsset = Balances;

		type CheckedAccount = CheckedAccount;
		type TrustedTeleporter = TrustedTeleporter;
		type TrustedReserve = TrustedReserve;

		fn get_asset() -> Asset {
			(RelayLocation::get(), UNITS).into()
		}
	}

	impl pallet_xcm_benchmarks::generic::Config for Runtime {
		type RuntimeCall = RuntimeCall;
		type TransactAsset = Balances;

		fn worst_case_response() -> (u64, Response) {
			(0u64, Response::Version(Default::default()))
		}

		fn worst_case_asset_exchange() -> Result<(XcmAssets, XcmAssets), BenchmarkError> {
			Err(BenchmarkError::Skip)
		}

		fn universal_alias() -> Result<(Location, Junction), BenchmarkError> {
			Err(BenchmarkError::Skip)
		}

		fn transact_origin_and_runtime_call() -> Result<(Location, RuntimeCall), BenchmarkError> {
			Ok((
				RelayLocation::get(),
				frame_system::Call::remark_with_event { remark: vec![] }.into(),
			))
		}

		fn subscribe_origin() -> Result<Location, BenchmarkError> {
			Ok(RelayLocation::get())
		}

		fn claimable_asset() -> Result<(Location, Location, XcmAssets), BenchmarkError> {
			let origin = RelayLocation::get();
			let assets: XcmAssets = (AssetId(RelayLocation::get()), 1_000 * UNITS).into();
			let ticket = Here.into();
			Ok((origin, ticket, assets))
		}

		fn unlockable_asset() -> Result<(Location, Location, Asset), BenchmarkError> {
			Err(BenchmarkError::Skip)
		}

		fn export_message_origin_and_destination() -> Result<(Location, NetworkId, InteriorLocation), BenchmarkError> {
			Err(BenchmarkError::Skip)
		}

		fn alias_origin() -> Result<(Location, Location), BenchmarkError> {
			Err(BenchmarkError::Skip)
		}

		fn worst_case_for_trader() -> Result<(Asset, WeightLimit), BenchmarkError> {
			Ok((
				Asset {
					id: AssetId(RelayLocation::get()),
					fun: Fungible(1_000 * UNITS),
				},
				Limited(Weight::from_parts(5000, 5000)),
			))
		}
	}
}
