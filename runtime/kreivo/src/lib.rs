#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit.
#![recursion_limit = "256"]
#![allow(clippy::items_after_test_module)]

// // Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod apis;
mod config;
mod constants;
mod genesis_config_presets;
mod impls;
mod weights;
mod xcm_config;

use apis::*;
use config::*;

use alloc::{boxed::Box, string::String, vec, vec::Vec};
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use sp_core::crypto::KeyTypeId;

pub use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{Block as BlockT, ConvertInto},
	MultiAddress, Perbill, Percent,
};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use virto_common::CommunityId;

pub use virto_common::FungibleAssetLocation;

use frame_support::{
	ensure,
	genesis_builder_helper::{build_state, get_preset},
	parameter_types,
	traits::{
		fungibles, tokens::imbalance::ResolveTo, ConstBool, ConstU32, ConstU64, Contains, EitherOf, EnsureOriginWithArg,
	},
	weights::{constants::RocksDbWeight, Weight},
	BoundedVec, PalletId,
};
use frame_system::{limits::BlockWeights, EnsureRoot};

use pallet_xcm::EnsureXcm;
use xcm_config::{LocationConvertedConcreteId, RelayLocation, XcmOriginToTransactDispatchOrigin};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot imports
pub use weights::{BlockExecutionWeight, ExtrinsicBaseWeight};

use pallet_asset_tx_payment::ChargeAssetTxPayment;
use pallet_gas_transaction_payment::ChargeTransactionPayment as ChargeGasTxPayment;
use pallet_pass::ChargeTransactionToPassAccount as ChargeTxToPassAccount;
use pallet_skip_feeless_payment::SkipCheckIfFeeless;

// XCM Imports
use xcm::latest::prelude::BodyId;

pub use constants::{currency::*, fee::WeightToFee};

use pallet_payments::types::*;

pub use impls::{EqualOrGreatestRootCmp, ProxyType, RuntimeBlackListedCalls};

pub use parachains_common::{AccountId, AuraId, Balance, BlockNumber, Hash, Header, Nonce, Signature};
pub use runtime_constants::time::{DAYS, HOURS, MINUTES};

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, CommunityId>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

pub type ChargeTransaction = ChargeGasTxPayment<Runtime, ChargeAssetTxPayment<Runtime>>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	SkipCheckIfFeeless<Runtime, frame_system::CheckNonce<Runtime>>,
	frame_system::CheckWeight<Runtime>,
	SkipCheckIfFeeless<Runtime, ChargeTxToPassAccount<ChargeTransaction, Runtime, ()>>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("kreivo-parachain"),
	impl_name: create_runtime_str!("kreivo-parachain"),
	authoring_version: 1,
	spec_version: 117,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 11,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

// Create the runtime by composing the FRAME pallets that were previously
// configured.
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
	pub struct Runtime;

	// System support stuff.
	#[runtime::pallet_index(0)]
	pub type System = frame_system;
	#[runtime::pallet_index(1)]
	pub type ParachainSystem = cumulus_pallet_parachain_system;
	#[runtime::pallet_index(2)]
	pub type Timestamp = pallet_timestamp;
	#[runtime::pallet_index(3)]
	pub type ParachainInfo = parachain_info;
	#[runtime::pallet_index(4)]
	pub type Origins = pallet_custom_origins;
	#[runtime::pallet_index(6)]
	pub type Pass = pallet_pass;

	// Monetary stuff.
	#[runtime::pallet_index(10)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(11)]
	pub type TransactionPayment = pallet_transaction_payment;
	#[runtime::pallet_index(12)]
	pub type AssetsFreezer = pallet_assets_freezer<Instance1>;
	#[runtime::pallet_index(13)]
	pub type Assets = pallet_assets<Instance1>;
	#[runtime::pallet_index(14)]
	pub type AssetsTxPayment = pallet_asset_tx_payment;
	#[runtime::pallet_index(15)]
	pub type Vesting = pallet_vesting;
	#[runtime::pallet_index(16)]
	pub type SkipFeeless = pallet_skip_feeless_payment;
	#[runtime::pallet_index(17)]
	pub type GasTxPayment = pallet_gas_transaction_payment;
	#[runtime::pallet_index(18)]
	pub type AssetsHolder = pallet_assets_holder<Instance1>;

	// Collator support. The order of these 4 are important and shall not change.
	#[runtime::pallet_index(20)]
	pub type Authorship = pallet_authorship;
	#[runtime::pallet_index(21)]
	pub type CollatorSelection = pallet_collator_selection;
	#[runtime::pallet_index(22)]
	pub type Session = pallet_session;
	#[runtime::pallet_index(23)]
	pub type Aura = pallet_aura;
	#[runtime::pallet_index(24)]
	pub type AuraExt = cumulus_pallet_aura_ext;

	// XCM helpers.
	#[runtime::pallet_index(30)]
	pub type XcmpQueue = cumulus_pallet_xcmp_queue;
	#[runtime::pallet_index(31)]
	pub type PolkadotXcm = pallet_xcm;
	#[runtime::pallet_index(32)]
	pub type CumulusXcm = cumulus_pallet_xcm;
	#[runtime::pallet_index(33)]
	pub type MessageQueue = pallet_message_queue;

	// Utils
	#[runtime::pallet_index(42)]
	pub type Multisig = pallet_multisig;
	#[runtime::pallet_index(43)]
	pub type Utility = pallet_utility;
	#[runtime::pallet_index(44)]
	pub type Proxy = pallet_proxy;
	#[runtime::pallet_index(45)]
	pub type Scheduler = pallet_scheduler;
	#[runtime::pallet_index(46)]
	pub type Preimage = pallet_preimage;

	// Governance
	#[runtime::pallet_index(50)]
	pub type Treasury = pallet_treasury;

	// Governance: Collective
	#[runtime::pallet_index(51)]
	pub type KreivoCollective = pallet_ranked_collective<Instance1>;
	#[runtime::pallet_index(52)]
	pub type KreivoReferenda = pallet_referenda<Instance1>;

	// Virto Tooling
	#[runtime::pallet_index(60)]
	pub type Payments = pallet_payments;
	#[runtime::pallet_index(61)]
	pub type Listings = pallet_listings<Instance1>;
	#[runtime::pallet_index(62)]
	pub type ListingsCatalog = pallet_nfts<Instance1>;
	#[runtime::pallet_index(63)]
	pub type Orders = pallet_orders<Instance1>;

	// Communities at Kreivo
	#[runtime::pallet_index(71)]
	pub type Communities = pallet_communities;
	#[runtime::pallet_index(72)]
	pub type CommunityTracks = pallet_referenda_tracks<Instance2>;
	#[runtime::pallet_index(73)]
	pub type CommunityReferenda = pallet_referenda<Instance2>;
	#[runtime::pallet_index(74)]
	pub type CommunityMemberships = pallet_nfts<Instance2>;
	#[runtime::pallet_index(75)]
	pub type CommunitiesManager = pallet_communities_manager;

	// Contracts
	#[runtime::pallet_index(80)]
	pub type Contracts = pallet_contracts;
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
