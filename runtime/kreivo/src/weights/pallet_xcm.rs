//! PLACEHOLDER weights for `pallet_xcm`, NOT measured on Kreivo.
//!
//! These replace `pallet_xcm::TestWeightInfo` (flat 100 µs, no proof size) until the pallet is
//! benchmarked on reference hardware. Run `/cmd bench --pallet pallet_xcm` to overwrite this
//! file with Kreivo's own weights.
//!
//! Starting point: the `pallet_xcm` weights of `people-westend` in polkadot-sdk (a system
//! parachain with the same XCM stack: `XcmpQueue`, `ParachainSystem`, `PolkadotXcm`), generated
//! with frame-omni-bencher, `--steps=50 --repeat=20`, on an `Intel(R) Xeon(R) CPU @ 2.60GHz`.
//! Exceptions, labelled below: `reserve_transfer_assets` (People has none, so its file holds
//! `Weight::MAX`) copies `transfer_assets`, and the per-size functions that file predates use
//! `TestWeightInfo`'s slopes on top of the matching flat weight.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_xcm`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_xcm::WeightInfo for WeightInfo<T> {
	fn send() -> Weight {
		Weight::from_parts(35_814_000, 0)
			.saturating_add(Weight::from_parts(0, 3709))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Kreivo teleports nothing (`XcmTeleportFilter = Nothing`); kept finite regardless.
	fn teleport_assets() -> Weight {
		Weight::from_parts(120_752_000, 0)
			.saturating_add(Weight::from_parts(0, 3709))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// PLACEHOLDER: copies `transfer_assets` (People has no reserve transfers).
	fn reserve_transfer_assets() -> Weight {
		Weight::from_parts(121_390_000, 0)
			.saturating_add(Weight::from_parts(0, 3709))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn transfer_assets() -> Weight {
		Weight::from_parts(121_390_000, 0)
			.saturating_add(Weight::from_parts(0, 3709))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// `XcmExecuteFilter = Nothing`: charged only before the call is filtered.
	fn execute() -> Weight {
		Weight::from_parts(9_822_000, 0)
			.saturating_add(Weight::from_parts(0, 1485))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	fn force_xcm_version() -> Weight {
		Weight::from_parts(8_402_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn force_default_xcm_version() -> Weight {
		Weight::from_parts(2_548_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn force_subscribe_version_notify() -> Weight {
		Weight::from_parts(38_249_000, 0)
			.saturating_add(Weight::from_parts(0, 3640))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	fn force_unsubscribe_version_notify() -> Weight {
		Weight::from_parts(44_006_000, 0)
			.saturating_add(Weight::from_parts(0, 108971))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn force_suspension() -> Weight {
		Weight::from_parts(2_542_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn migrate_supported_version() -> Weight {
		Weight::from_parts(21_089_000, 0)
			.saturating_add(Weight::from_parts(0, 15863))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn migrate_version_notifiers() -> Weight {
		Weight::from_parts(20_719_000, 0)
			.saturating_add(Weight::from_parts(0, 15867))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn already_notified_target() -> Weight {
		Weight::from_parts(25_911_000, 0)
			.saturating_add(Weight::from_parts(0, 18394))
			.saturating_add(T::DbWeight::get().reads(7))
	}
	fn notify_current_targets() -> Weight {
		Weight::from_parts(36_739_000, 0)
			.saturating_add(Weight::from_parts(0, 6059))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn notify_target_migration_fail() -> Weight {
		Weight::from_parts(18_070_000, 0)
			.saturating_add(Weight::from_parts(0, 13444))
			.saturating_add(T::DbWeight::get().reads(5))
	}
	fn migrate_version_notify_targets() -> Weight {
		Weight::from_parts(20_845_000, 0)
			.saturating_add(Weight::from_parts(0, 15874))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn migrate_and_notify_old_targets() -> Weight {
		Weight::from_parts(46_301_000, 0)
			.saturating_add(Weight::from_parts(0, 15959))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn new_query() -> Weight {
		Weight::from_parts(2_817_000, 0)
			.saturating_add(Weight::from_parts(0, 1485))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn take_response() -> Weight {
		Weight::from_parts(28_864_000, 0)
			.saturating_add(Weight::from_parts(0, 11041))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn claim_assets() -> Weight {
		Weight::from_parts(40_179_000, 0)
			.saturating_add(Weight::from_parts(0, 3489))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn add_authorized_alias() -> Weight {
		Weight::from_parts(52_716_000, 0)
			.saturating_add(Weight::from_parts(0, 3626))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn remove_authorized_alias() -> Weight {
		Weight::from_parts(54_038_000, 0)
			.saturating_add(Weight::from_parts(0, 3982))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn weigh_message() -> Weight {
		Weight::from_parts(8_639_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	// PLACEHOLDER: `weigh_message` plus `TestWeightInfo`'s per-byte slope.
	/// The range of component `n` is `[0, 8192]`.
	fn weigh_message_by_size(n: u32, ) -> Weight {
		Weight::from_parts(8_639_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(Weight::from_parts(100_000, 0).saturating_mul(n.into()))
	}
	// PLACEHOLDER: `weigh_message` plus `TestWeightInfo`'s per-byte slope.
	/// The range of component `n` is `[0, 131072]`.
	fn decode_xcm(n: u32, ) -> Weight {
		Weight::from_parts(8_639_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(Weight::from_parts(20_000, 0).saturating_mul(n.into()))
	}
	// PLACEHOLDER: `claim_assets` plus `TestWeightInfo`'s per-asset slope, and an `Assets`
	// asset and account read and written per extra asset.
	/// The range of component `n` is `[1, 20]`.
	fn claim_assets_by_size(n: u32, ) -> Weight {
		Weight::from_parts(40_179_000, 0)
			.saturating_add(Weight::from_parts(0, 3489))
			.saturating_add(Weight::from_parts(10_000_000, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2_606).saturating_mul(n.into()))
	}
}
