use super::*;

use frame_support::traits::TransformOrigin;
use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};
use polkadot_runtime_common::xcm_sender::ExponentialPrice;

use super::currency::TransactionByteFee;

// #[runtime::pallet_index(30)]
// pub type XcmpQueue
impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	// Wrap outgoing messages in the XCM version each destination supports.
	type VersionWrapper = PolkadotXcm;
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = ConstU32<1_000>;
	type MaxActiveOutboundChannels = ConstU32<128>;
	// From https://github.com/polkadot-fellows/runtimes/blob/88da7d0abb9edbff027e57bc998a97c6ff10c18c/system-parachains/asset-hubs/asset-hub-kusama/src/lib.rs#L783:
	//
	// Most on-chain HRMP channels are configured to use 102400 bytes of max message
	// size, so we need to set the page size larger than that until we reduce the
	// channel size on-chain.
	type MaxPageSize = ConstU32<{ 103 * 1024 }>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type PriceForSiblingDelivery = PriceForSiblingDelivery;
	type WeightInfo = weights::cumulus_pallet_xcmp_queue::WeightInfo<Self>;
}

// #[runtime::pallet_index(33)]
// pub type MessageQueue
parameter_types! {
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_message_queue::WeightInfo<Self>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
		RuntimeCall,
	>;
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
	type Size = u32;
	// The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)`
	// origin:
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = ConstU32<{ 64 * 1024 }>;
	type MaxStale = ConstU32<8>;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = ();
}

// Message delivery fees, as the fellowship system chains price them: a base fee plus a
// per-byte fee, both in KSM and scaled up exponentially while the channel is congested.
// The base fees are the fellowship's, 3 CENTS. Only Root doesn't pay them; see
// `WaivedLocations` in `xcm_config.rs`.
parameter_types! {
	/// The asset used to pay message delivery fees.
	pub FeeAssetId: cumulus_primitives_core::AssetId = xcm_config::RelayLocation::get().into();
	/// The base fee for messages to the relay chain.
	pub const ToParentBaseDeliveryFee: u128 = CENTS.saturating_mul(3);
	/// The base fee for messages to sibling parachains.
	pub const ToSiblingBaseDeliveryFee: u128 = CENTS.saturating_mul(3);
}

pub type PriceForParentDelivery =
	ExponentialPrice<FeeAssetId, ToParentBaseDeliveryFee, TransactionByteFee, ParachainSystem>;

pub type PriceForSiblingDelivery =
	ExponentialPrice<FeeAssetId, ToSiblingBaseDeliveryFee, TransactionByteFee, XcmpQueue>;
