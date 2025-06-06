
//! Autogenerated weights for `pallet_vesting`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 45.0.0
//! DATE: 2025-01-27, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `virto-us3`, CPU: `Intel(R) Xeon(R) Silver 4216 CPU @ 2.10GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: 1024

// Executed Command:
// /home/devops/.cargo/bin/frame-omni-bencher
// v1
// benchmark
// pallet
// --runtime
// target/release/wbuild/kreivo-runtime/kreivo_runtime.compact.compressed.wasm
// --pallet
// pallet_vesting
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output
// ./runtime/kreivo/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_vesting`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_vesting::WeightInfo for WeightInfo<T> {
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[1, 28]`.
	fn vest_locked(l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `205 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 83_240_000 picoseconds.
		Weight::from_parts(127_044_307, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 31_518
			.saturating_add(Weight::from_parts(148_458, 0).saturating_mul(l.into()))
			// Standard Error: 56_077
			.saturating_add(Weight::from_parts(714_701, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[1, 28]`.
	fn vest_unlocked(l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `205 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 84_033_000 picoseconds.
		Weight::from_parts(140_188_748, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 34_176
			.saturating_add(Weight::from_parts(225_224, 0).saturating_mul(l.into()))
			// Standard Error: 60_805
			.saturating_add(Weight::from_parts(425_122, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[1, 28]`.
	fn vest_other_locked(l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `308 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 83_855_000 picoseconds.
		Weight::from_parts(148_984_334, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 34_398
			.saturating_add(Weight::from_parts(17_313, 0).saturating_mul(l.into()))
			// Standard Error: 61_200
			.saturating_add(Weight::from_parts(294_210, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[1, 28]`.
	fn vest_other_unlocked(l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `308 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 90_104_000 picoseconds.
		Weight::from_parts(143_014_078, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 35_305
			.saturating_add(Weight::from_parts(280_981, 0).saturating_mul(l.into()))
			// Standard Error: 62_813
			.saturating_add(Weight::from_parts(583_979, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[0, 27]`.
	fn vested_transfer(_l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `379 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 166_745_000 picoseconds.
		Weight::from_parts(288_219_170, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 135_114
			.saturating_add(Weight::from_parts(488_726, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[0, 27]`.
	fn force_vested_transfer(l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `482 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 170_009_000 picoseconds.
		Weight::from_parts(252_122_151, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 65_671
			.saturating_add(Weight::from_parts(503_665, 0).saturating_mul(l.into()))
			// Standard Error: 116_840
			.saturating_add(Weight::from_parts(1_356_825, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[2, 28]`.
	fn not_unlocking_merge_schedules(l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `306 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 88_388_000 picoseconds.
		Weight::from_parts(147_316_554, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 33_926
			.saturating_add(Weight::from_parts(112_220, 0).saturating_mul(l.into()))
			// Standard Error: 62_653
			.saturating_add(Weight::from_parts(395_613, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[2, 28]`.
	fn unlocking_merge_schedules(l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `306 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 91_047_000 picoseconds.
		Weight::from_parts(153_956_970, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 37_302
			.saturating_add(Weight::from_parts(179_792, 0).saturating_mul(l.into()))
			// Standard Error: 68_888
			.saturating_add(Weight::from_parts(303_397, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Vesting::Vesting` (r:1 w:1)
	/// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1057), added: 3532, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(4658), added: 7133, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[0, 49]`.
	/// The range of component `s` is `[2, 28]`.
	fn force_remove_vesting_schedule(l: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `379 + l * (25 ±0) + s * (36 ±0)`
		//  Estimated: `8123`
		// Minimum execution time: 102_620_000 picoseconds.
		Weight::from_parts(166_846_437, 0)
			.saturating_add(Weight::from_parts(0, 8123))
			// Standard Error: 37_159
			.saturating_add(Weight::from_parts(6_289, 0).saturating_mul(l.into()))
			// Standard Error: 68_623
			.saturating_add(Weight::from_parts(261_818, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
