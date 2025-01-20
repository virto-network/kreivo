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
pub mod payments;

pub use collator_support::{ConsensusHook, SLOT_DURATION};
#[cfg(feature = "runtime-benchmarks")]
pub use currency::{ExistentialDeposit, TransactionByteFee};
pub use currency::{KreivoAssetsCall, KreivoAssetsInstance, MembershipsGasTank};
pub use governance::{pallet_custom_origins, TreasuryAccount};
pub use system::RuntimeBlockWeights;
#[cfg(feature = "runtime-benchmarks")]
pub use xcm::PriceForParentDelivery;
