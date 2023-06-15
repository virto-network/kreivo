use super::{
	AccountId, AllPalletsWithSystem, AssetIdForTrustBackedAssets, AssetRegistry, Assets, Balance, Balances,
	ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, WeightToFee,
	XcmpQueue,
};
use crate::constants::fee::default_fee_per_second;
use frame_support::{
	match_types, parameter_types,
	traits::{ConstU32, ContainsPair, Everything, Get, Nothing, PalletInfoAccess},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
/* use parachains_common::xcm_config::{DenyReserveTransferToRelayChain,
 * DenyThenTry}; */
use polkadot_parachain::primitives::Sibling;
use sp_std::marker::PhantomData;
use virto_common::impls::{AsAssetMultiLocation, ConvertedRegisteredAssetId, DealWithFees};
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowTopLevelPaidExecutionFrom, CurrencyAdapter,
	EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds, FungiblesAdapter, IsConcrete, LocalMint, MintLocation,
	NativeAsset, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
	WithComputedOrigin,
};
use xcm_executor::traits::JustTry;
use xcm_executor::XcmExecutor;

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: Option<NetworkId> = None;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub CheckAccount: (AccountId, MintLocation) = (PolkadotXcm::check_account(), MintLocation::Local);
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub TrustBackedAssetsPalletLocation: MultiLocation =
		PalletInstance(<Assets as PalletInfoAccess>::index() as u8).into();
	pub UniversalLocation: InteriorMultiLocation = (
		//TODO: to change the NetworkId to Kusama once we finish testing on Rococo.
		GlobalConsensus(NetworkId::Rococo),
		Parachain(ParachainInfo::parachain_id().into()),
	).into();

}

/// Type for specifying how a `MultiLocation` can be converted into an
/// `AccountId`. This is used when determining ownership of accounts for asset
/// transacting and when attempting to use XCM `Transact` in order to determine
/// the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type TrustBackedAssetsConvertedConcreteId =
	assets_common::TrustBackedAssetsConvertedConcreteId<TrustBackedAssetsPalletLocation, Balance>;

/// Means for transacting assets besides the native currency on this chain.
pub type FungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a registered fungible asset matching the given location or name
	// Assets not found in AssetRegistry will not be used
	ConvertedRegisteredAssetId<
		AssetIdForTrustBackedAssets,
		Balance,
		AsAssetMultiLocation<AssetIdForTrustBackedAssets, AssetRegistry>,
		JustTry,
	>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We only want to allow teleports of known assets. We use non-zero issuance as an indication
	// that this asset is known.
	LocalMint<parachains_common::impls::NonZeroIssuance<AccountId, Assets>>,
	// The account to use for tracking teleports.
	CheckingAccount,
>;

/// Means for transacting the native currency on this chain.
pub type CurrencyTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Balances`.
	CheckAccount,
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

match_types! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
	};
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

pub type AssetTransactors = (CurrencyTransactor, FungiblesTransactor);

parameter_types! {
	pub UsdtPerSecond: (xcm::v3::AssetId, u128, u128) = (
		MultiLocation::new(1, X3(Parachain(1000), PalletInstance(50), GeneralIndex(1984))).into(),
		default_fee_per_second() * 10,
		0
	);
	pub StatemineLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(1000)));
	// ALWAYS ensure that the index in PalletInstance stays up-to-date with
	// Statemine's Assets pallet index
	pub StatemineAssetsPalletLocation: MultiLocation =
		MultiLocation::new(1, X2(Parachain(1000), PalletInstance(50)));
}

//- From PR https://github.com/paritytech/cumulus/pull/936
fn matches_prefix(prefix: &MultiLocation, loc: &MultiLocation) -> bool {
	prefix.parent_count() == loc.parent_count()
		&& loc.len() >= prefix.len()
		&& prefix
			.interior()
			.iter()
			.zip(loc.interior().iter())
			.all(|(prefix_junction, junction)| prefix_junction == junction)
}
pub struct ReserveAssetsFrom<T>(PhantomData<T>);
impl<T: Get<MultiLocation>> ContainsPair<MultiAsset, MultiLocation> for ReserveAssetsFrom<T> {
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		let prefix = T::get();
		log::trace!(target: "xcm::AssetsFrom", "prefix: {:?}, origin: {:?}", prefix, origin);
		&prefix == origin
			&& match asset {
				MultiAsset {
					id: xcm::latest::AssetId::Concrete(asset_loc),
					fun: Fungible(_a),
				} => matches_prefix(&prefix, asset_loc),
				_ => false,
			}
	}
}

pub type Traders = (
	// USDT
	FixedRateOfFungible<UsdtPerSecond, ()>,
	// Everything else
	UsingComponents<WeightToFee, RelayLocation, AccountId, Balances, DealWithFees<Runtime>>,
);

pub type Reserves = (NativeAsset, ReserveAssetsFrom<StatemineLocation>);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = Reserves;
	type IsTeleporter = (); // Teleporting is disabled.
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = Traders;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	// ^ Disable dispatchable execute on the XCM pallet.
	// Needs to be `Everything` for local testing.
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
	type AdminOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}