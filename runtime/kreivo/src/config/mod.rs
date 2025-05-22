//! Configure FRAME pallets to include in runtime.

use super::*;

mod collator_support;
pub mod currency;
pub mod system;
mod utilities;
mod xcm;
// Kreivo Governance
pub mod collective;
pub mod communities;
pub mod governance;
// Virto toolchain
pub mod contracts;
mod listings_orders;
pub mod payments;

pub use {
	collator_support::{ConsensusHook, SLOT_DURATION},
	currency::{KreivoAssetsCall, KreivoAssetsInstance, MembershipsGasTank},
	governance::{pallet_custom_origins, TreasuryAccount},
	system::{RelaychainData, RuntimeBlockWeights},
};

#[cfg(feature = "runtime-benchmarks")]
pub use {
	currency::{ExistentialDeposit, TransactionByteFee},
	xcm::PriceForParentDelivery,
};
