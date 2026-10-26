//! Configure FRAME pallets to include in runtime.

use super::*;

pub mod call_matcher;
pub(crate) mod collator_support;
pub mod currency;
pub mod system;
pub(crate) mod utilities;
mod xcm;
// Kreivo Governance
pub mod collective;
pub mod communities;
pub mod governance;
// Virto toolchain
pub mod contracts;
pub(crate) mod listings_orders;
pub mod payments;
pub mod usage_subscription;

pub use {
	collator_support::{ConsensusHook, RELAY_CHAIN_SLOT_DURATION_MILLIS},
	communities::Memberships,
	currency::{KreivoAssetsCall, KreivoAssetsInstance},
	governance::{pallet_custom_origins, TreasuryAccount},
	payments::pallet_payment_indices,
	system::{RelaychainData, RuntimeBlockWeights},
	xcm::PriceForParentDelivery,
};

#[cfg(feature = "runtime-benchmarks")]
pub use currency::ExistentialDeposit;
#[cfg(feature = "runtime-benchmarks")]
pub use xcm::PriceForSiblingDelivery;
