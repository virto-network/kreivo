use super::{
	AccountId, AllPalletsWithSystem, Assets, Balance, Balances, FungibleAssetLocation, KreivoAssetsInstance,
	ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeHoldReason, RuntimeOrigin,
	TreasuryAccount, WeightToFee, XcmpQueue,
};
use virto_common::AsFungibleAssetLocation;

use crate::constants::locations::ASSET_HUB_ID;
use core::marker::PhantomData;
use frame_support::traits::fungible::HoldConsideration;
use frame_support::traits::LinearStoragePrice;
use frame_support::{
	parameter_types,
	traits::{
		tokens::imbalance::{ResolveAssetTo, ResolveTo},
		ConstU32, Contains, ContainsPair, Everything, Get, Nothing, PalletInfoAccess,
	},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use parachains_common::xcm_config::{
	AliasAccountId32FromSiblingSystemChain, AssetFeeAsExistentialDepositMultiplier, ParentRelayOrSiblingParachains,
};
use polkadot_parachain_primitives::primitives::Sibling;
use sp_runtime::traits::ConvertInto;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AliasChildLocation, AliasOriginRootUsingFilter, AllowExplicitUnpaidExecutionFrom,
	AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom, Case, ConvertedConcreteId,
	DenyReserveTransferToRelayChain, DenyThenTry, DescribeAllTerminal, DescribeFamily, EnsureXcmOrigin,
	ExternalConsensusLocationsConverterFor, FrameTransactionalProcessor, FungibleAdapter, FungiblesAdapter,
	HashedDescription, IsConcrete, LocalMint, MintLocation, ParentIsPreset, RelayChainAsNative, SendXcmFeeToAccount,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, StartsWith, TakeWeightCredit, TrailingSetTopicAsId, UsingComponents, WeightInfoBounds,
	WithComputedOrigin, WithUniqueTopic, XcmFeeManagerFromComponents,
};
use xcm_executor::traits::JustTry;
use xcm_executor::XcmExecutor;

mod communities;
#[cfg(test)]
mod tests;
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
	// Kusama (or, on `paseo`, the Paseo relay, which identifies as `Polkadot` like Paseo Asset
	// Hub does) followed by our para id. Reanchoring, `WithComputedOrigin` and bridged-account
	// conversion all depend on this being a valid universal location.
	pub UniversalLocation: InteriorLocation = [
		GlobalConsensus(RelayNetwork::get().expect("RelayNetwork is always set; qed")),
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
	AccountId32FromRelayOrAssetHub<RelayNetwork, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
	// Origins from other consensus systems (e.g. the escrow on Polkadot Asset Hub, reaching us
	// over the bridge) get a deterministic account, so fees can be refunded and trapped assets
	// claimed.
	ExternalConsensusLocationsConverterFor<UniversalLocation, AccountId>,
	// Any other location in our consensus (e.g. an account on a sibling parachain) gets a
	// hashed account, so it can pay fees, `Transact` and claim trapped assets. It comes last,
	// so the 1:1 relay/Asset Hub mapping and community accounts above keep their addresses.
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
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

pub type Barrier = TrailingSetTopicAsId<
	DenyThenTry<
		// The relay chain is no reserve for anything: reject reserve-based transfers to it.
		DenyReserveTransferToRelayChain,
		(
			TakeWeightCredit,
			// Responses to queries we made (e.g. XCM version discovery).
			AllowKnownQueryResponses<PolkadotXcm>,
			WithComputedOrigin<
				(
					AllowTopLevelPaidExecutionFrom<Everything>,
					// Parent and its exec plurality get free execution, also after a trusted alias.
					AllowExplicitUnpaidExecutionFrom<ParentOrParentsExecutivePlurality, CheapTrustedAliasers>,
					// Version subscriptions from the relay and siblings, so they can learn our XCM version.
					AllowSubscriptionsFrom<ParentRelayOrSiblingParachains>,
				),
				UniversalLocation,
				ConstU32<8>,
			>,
		),
	>,
>;

pub type AssetTransactors = (FungibleTransactor, FungiblesTransactor);

parameter_types! {
	pub AssetHubLocation: Location = Location::new(1, [Parachain(ASSET_HUB_ID)]);
	pub Ksm: Location = Location::new(1, Here);
	// NOTE: meaningless on the `paseo` build, where `Polkadot` is our own consensus.
	pub Dot: Location = Location::new(2, [GlobalConsensus(Polkadot)]);
	pub PolkadotConsensus: Location = Location::new(2, [GlobalConsensus(Polkadot)]);
}

/// Locations within the Polkadot consensus system.
pub struct PolkadotOrigins;
impl Contains<Location> for PolkadotOrigins {
	fn contains(location: &Location) -> bool {
		location.starts_with(&PolkadotConsensus::get())
	}
}

/// Aliases that don't need an on-chain authorization:
/// - origins may alias into their own children;
/// - (Kusama) Asset Hub may hand over an origin preserved from the Polkadot side of the bridge
///   (`InitiateTransfer { preserve_origin: true }`), as Kusama Asset Hub itself allows Polkadot
///   Asset Hub to do. Asset Hub cannot alias arbitrary Kusama origins;
/// - an account on a sibling system chain may alias the same account here, the counterpart of
///   the 1:1 `AccountId32` mapping in `LocationToAccountId`.
pub type CheapTrustedAliasers = (
	AliasChildLocation,
	AliasOriginRootUsingFilter<AssetHubLocation, PolkadotOrigins>,
	AliasAccountId32FromSiblingSystemChain,
);

/// All aliases we accept: the ones above, plus those an account authorized on chain with
/// `PolkadotXcm::add_authorized_alias` (which holds `AuthorizedAliasConsideration`).
pub type TrustedAliasers = (CheapTrustedAliasers, pallet_xcm::AuthorizedAliasers<Runtime>);

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

parameter_types! {
	/// KSM, reserved on Asset Hub since the Kusama Asset Hub migration.
	pub KsmFromAssetHub: (AssetFilter, Location) =
		(Wild(AllOf { id: AssetId(Ksm::get()), fun: WildFungible }), AssetHubLocation::get());
	/// DOT, reserved on (Kusama) Asset Hub.
	pub DotFromAssetHub: (AssetFilter, Location) =
		(Wild(AllOf { id: AssetId(Dot::get()), fun: WildFungible }), AssetHubLocation::get());
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
		ResolveAssetTo<TreasuryAccount, Assets>,
	>,
	// Everything else
	UsingComponents<WeightToFee, RelayLocation, AccountId, Balances, ResolveTo<TreasuryAccount, Balances>>,
);

/// Asset Hub is the only reserve we trust: for its own assets, for KSM and for DOT. KSM with
/// the relay chain as reserve is no longer accepted (it moved to Asset Hub), nor are other
/// chains' native tokens.
pub type Reserves = (
	ReserveAssetsFrom<AssetHubLocation>,
	Case<KsmFromAssetHub>,
	Case<DotFromAssetHub>,
);

parameter_types! {
	pub RootLocation: Location = Location::here();
}

/// Senders that pay no delivery fees: Root (governance) only. Communities can be created by
/// anyone, so a waiver for them would hand out free outbound messages.
pub type WaivedLocations = frame_support::traits::Equals<RootLocation>;

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
	type Aliasers = TrustedAliasers;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = WeightInfoBounds<crate::weights::xcm::KreivoXcmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
	type Trader = Traders;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetLocker = ();
	type AssetExchanger = ();
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager =
		XcmFeeManagerFromComponents<WaivedLocations, SendXcmFeeToAccount<AssetTransactors, TreasuryAccount>>;
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = PolkadotXcm;
}

parameter_types! {
	/// Asset ids whose supply on Kreivo is minted against bridged messages, not backed by a
	/// reserve we hold elsewhere. Extended by the bridge work.
	pub BridgeBackedAssets: alloc::vec::Vec<Location> = alloc::vec![
		// USDC and USDT, as minted by Phase 0 and the bridge.
		Location::new(1, [Parachain(ASSET_HUB_ID), PalletInstance(50), GeneralIndex(1337)]),
		Location::new(1, [Parachain(ASSET_HUB_ID), PalletInstance(50), GeneralIndex(1984)]),
	];
}

/// Rejects reserve transfers that include a [`BridgeBackedAssets`] id. Moving those through
/// Asset Hub would burn them here and draw on a sovereign balance there that does not back them.
pub struct NotBridgeBacked;
impl Contains<(Location, alloc::vec::Vec<Asset>)> for NotBridgeBacked {
	fn contains((_, assets): &(Location, alloc::vec::Vec<Asset>)) -> bool {
		let bridge_backed = BridgeBackedAssets::get();
		!assets.iter().any(|asset| bridge_backed.contains(&asset.id.0))
	}
}

/// Only communities are allowed to dispatch xcm messages. Root can always send as `Here`
/// (`EnsureXcmOrigin` falls back to it).
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
pub type XcmRouter = WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, crate::config::PriceForParentDelivery>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;

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
	type XcmReserveTransferFilter = NotBridgeBacked;
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
		vec, UNITS,
	};
	use frame_benchmarking::BenchmarkError;
	use xcm::prelude::Assets as XcmAssets;
	use xcm_executor::AssetsInHolding;

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

		fn worst_case_holding(depositable_count: u32) -> AssetsInHolding {
			let mut holding =
				pallet_xcm_benchmarks::generate_holding_assets(MaxAssetsIntoHolding::get() - depositable_count - 1);
			// The trader benchmarks (`buy_execution`, `pay_fees`, `refund_surplus`) pay in the
			// relay asset (see `worst_case_for_trader`), which the generic helper doesn't hold:
			// it fills holding with `Here` and `GeneralIndex` assets only.
			holding.fungible.insert(
				AssetId(RelayLocation::get()),
				alloc::boxed::Box::new(pallet_xcm_benchmarks::MockCredit(u128::MAX)),
			);
			holding
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
