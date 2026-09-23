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
	type WeightInfo = crate::weights::pallet_xcm::WeightInfo<Runtime>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;

	use crate::{
		config::{ExistentialDeposit, PriceForParentDelivery, PriceForSiblingDelivery},
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
			// `AliasChildLocation`: a sibling parachain may alias any location beneath itself,
			// which is what the origin-preservation flows rely on (see `xcm_config::tests`).
			let origin = Location::new(1, [Parachain(ASSET_HUB_ID)]);
			let target = Location::new(
				1,
				[
					Parachain(ASSET_HUB_ID),
					AccountId32 {
						network: None,
						id: [1u8; 32],
					},
				],
			);
			Ok((origin, target))
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

	parameter_types! {
		pub AssetHubParaId: cumulus_primitives_core::ParaId = ASSET_HUB_ID.into();
	}

	/// The `Assets` pallet instance on Asset Hub.
	const ASSET_HUB_ASSETS_PALLET: u8 = 50;
	/// First `GeneralIndex` of the Asset Hub assets the benchmarks use. Clear of the
	/// [`BridgeBackedAssets`] (1337, 1984), which can't be reserve-transferred.
	const BENCHMARK_ASSETS_BASE_INDEX: u32 = 1_000_000;

	/// An Asset Hub asset (other than KSM) that Asset Hub is the reserve of.
	fn asset_hub_asset(index: u32) -> Location {
		Location::new(
			1,
			[
				Parachain(ASSET_HUB_ID),
				PalletInstance(ASSET_HUB_ASSETS_PALLET),
				GeneralIndex(index.into()),
			],
		)
	}

	fn account_location(who: &AccountId) -> Location {
		Junction::AccountId32 {
			network: None,
			id: who.clone().into(),
		}
		.into()
	}

	/// Deposits `asset` into `who` the way a reserve transfer from Asset Hub does: through the
	/// asset transactor, which also creates an Asset Hub asset the first time it sees it.
	fn deposit_as_reserve_transfer(asset: Asset, who: &Location) {
		use xcm_executor::traits::TransactAsset;
		let context = XcmContext {
			origin: None,
			message_id: XcmHash::default(),
			topic: None,
		};
		let holding = AssetTransactors::mint_asset(&asset, &context).expect("the asset transactor handles the asset");
		AssetTransactors::deposit_asset(holding, who, Some(&context))
			.map_err(|(_, error)| error)
			.expect("the account can receive the asset");
	}

	impl pallet_xcm::benchmarking::Config for Runtime {
		// Asset Hub is where Kreivo sends: it is the reserve of KSM and of every asset we accept.
		type DeliveryHelper = polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper<
			XcmConfig,
			ExistentialDepositAsset,
			PriceForSiblingDelivery,
			AssetHubParaId,
			ParachainSystem,
		>;

		fn reachable_dest() -> Option<Location> {
			Some(AssetHubLocation::get())
		}

		fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
			// Kreivo teleports nothing: `IsTeleporter = ()` and `XcmTeleportFilter = Nothing`.
			None
		}

		fn reserve_transferable_asset_and_dest() -> Option<(Asset, Location)> {
			use frame_support::traits::fungible::Mutate;
			use xcm_executor::traits::ConvertLocation;

			// An Asset Hub asset going back to Asset Hub, its reserve (`DestinationReserve`).
			// Not KSM: `reserve_transfer_assets` and `transfer_assets` refuse reserve transfers
			// of the network's native asset (`InvalidAssetUnknownReserve`), which can only leave
			// through `transfer_assets_using_type_and_then`. The benchmark mints the asset to the
			// sender itself.
			let amount: Balance = 1_000_000;
			let asset: Asset = (asset_hub_asset(BENCHMARK_ASSETS_BASE_INDEX), amount).into();
			let dest = AssetHubLocation::get();

			// Kreivo burns what it sends to the reserve, but the benchmark then withdraws the
			// amount from the destination's sovereign account, as it would after a local-reserve
			// transfer. Fund that account so the check holds; the transfer measured is the real
			// one. The KSM keeps the account alive, as Asset Hub assets aren't sufficient.
			let asset_hub_sovereign = LocationToAccountId::convert_location(&dest)?;
			Balances::set_balance(&asset_hub_sovereign, ExistentialDeposit::get());
			deposit_as_reserve_transfer(asset.clone(), &dest);

			Some((asset, dest))
		}

		fn set_up_complex_asset_transfer() -> Option<(XcmAssets, u32, Location, alloc::boxed::Box<dyn FnOnce()>)> {
			use frame_support::traits::fungible::Mutate;
			use sp_runtime::traits::MaybeEquivalence;

			// Kreivo teleports nothing, so the most involved transfer `transfer_assets` makes is
			// several Asset Hub assets going back to their reserve, paying fees in one of them:
			// both `DestinationReserve`. KSM can't be one of them (see
			// `reserve_transferable_asset_and_dest`), though it pays for delivery.
			let dest = AssetHubLocation::get();
			let who: AccountId = frame_benchmarking::whitelisted_caller();
			let who_location = account_location(&who);

			// KSM for the delivery fees (and to keep the account, as the assets aren't sufficient).
			let balance = UNITS;
			Balances::set_balance(&who, balance);

			let initial_amount: Balance = 1_000_000;
			let fee_location = asset_hub_asset(BENCHMARK_ASSETS_BASE_INDEX);
			let fee_id = AsFungibleAssetLocation::convert(&fee_location)?;
			let fee_amount: Balance = 100_000;
			let asset_location = asset_hub_asset(BENCHMARK_ASSETS_BASE_INDEX + 1);
			let asset_id = AsFungibleAssetLocation::convert(&asset_location)?;
			let asset_amount: Balance = 100_000;
			for location in [&fee_location, &asset_location] {
				deposit_as_reserve_transfer((location.clone(), initial_amount).into(), &who_location);
			}
			assert_eq!(Assets::balance(fee_id.clone(), &who), initial_amount);
			assert_eq!(Assets::balance(asset_id.clone(), &who), initial_amount);

			let fee_asset: Asset = (fee_location, fee_amount).into();
			let assets: XcmAssets = vec![fee_asset.clone(), (asset_location, asset_amount).into()].into();
			let fee_index = assets.inner().iter().position(|asset| asset.id == fee_asset.id)? as u32;

			let verify = alloc::boxed::Box::new(move || {
				// The fee asset went down by at least the fees sent along.
				assert!(Assets::balance(fee_id, &who) <= initial_amount - fee_amount);
				// The other asset went down by exactly the amount transferred.
				assert_eq!(Assets::balance(asset_id, &who), initial_amount - asset_amount);
				// Delivery was paid in KSM.
				assert!(Balances::free_balance(&who) < balance);
			});

			Some((assets, fee_index, dest, verify))
		}

		fn get_asset() -> Asset {
			use frame_support::traits::fungible::Mutate;

			// KSM, the only asset of the `Balances` transactor. The claimer exists already, as
			// the owner of trapped assets does.
			let who: AccountId = frame_benchmarking::whitelisted_caller();
			Balances::set_balance(&who, ExistentialDeposit::get());
			(RelayLocation::get(), UNITS).into()
		}

		fn get_assets(n: u32) -> XcmAssets {
			use frame_support::traits::fungibles::Create;
			use sp_runtime::traits::MaybeEquivalence;

			// The `Assets` transactor also deposits every Asset Hub asset, so claims can hold
			// many distinct ones: KSM plus `n - 1` of those.
			let mut assets = vec![Self::get_asset()];
			assets.extend((1..n).map(|i| {
				let location = asset_hub_asset(BENCHMARK_ASSETS_BASE_INDEX + i);
				// The transactor would create each asset on first sight, as not sufficient. Each
				// such asset takes a consumer reference from the claimer, and `MaxConsumers` (16)
				// would run out before `MAX_ITEMS_IN_ASSETS` (20). Create them as sufficient
				// instead: the claim does the same work, minus the consumer bookkeeping.
				let id = AsFungibleAssetLocation::convert(&location).expect("an Asset Hub asset location; qed");
				<Assets as Create<AccountId>>::create(id, TreasuryAccount::get(), true, 1)
					.expect("the asset doesn't exist yet");
				(location, 1_000_000u128).into()
			}));
			assets.into()
		}

		fn batch_call(calls: alloc::vec::Vec<RuntimeCall>) -> Option<RuntimeCall> {
			Some(RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }))
		}
	}
}
