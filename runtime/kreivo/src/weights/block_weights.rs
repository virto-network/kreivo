
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 60.0.0
//! DATE: 2026-09-24 (Y/M/D)
//! HOSTNAME: `virto-bench-satwwyb7exyl`, CPU: `AMD EPYC-Milan Processor`
//!
//! SHORT-NAME: `block`, LONG-NAME: `BlockExecution`, RUNTIME: `kreivo-parachain`
//! WARMUPS: `10`, REPEAT: `100`
//! WEIGHT-PATH: `./runtime/kreivo/src/weights/`
//! WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1.0`, WEIGHT-ADD: `0`

// Executed Command:
//   frame-omni-bencher
//   v1
//   benchmark
//   overhead
//   --runtime
//   /home/runner/actions-runner/_work/_temp/runtime/kreivo_runtime.compact.compressed.wasm
//   --genesis-builder
//   runtime
//   --genesis-builder-preset
//   development
//   --weight-path
//   ./runtime/kreivo/src/weights/
//   --warmup
//   10
//   --repeat
//   100
//   --para-id
//   2281

use sp_core::parameter_types;
use sp_weights::{constants::WEIGHT_REF_TIME_PER_NANOS, Weight};

parameter_types! {
	/// Weight of executing an empty block.
	/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
	///
	/// Stats nanoseconds:
	///   Min, Max: 853_823, 995_530
	///   Average:  919_269
	///   Median:   920_373
	///   Std-Dev:  23284.07
	///
	/// Percentiles nanoseconds:
	///   99th: 980_541
	///   95th: 953_912
	///   75th: 929_922
	pub const BlockExecutionWeight: Weight =
		Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(919_269), 4_144);
}

#[cfg(test)]
mod test_weights {
	use sp_weights::constants;

	/// Checks that the weight exists and is sane.
	// NOTE: If this test fails but you are sure that the generated values are fine,
	// you can delete it.
	#[test]
	fn sane() {
		let w = super::BlockExecutionWeight::get();

		// At least 100 µs.
		assert!(
			w.ref_time() >= 100u64 * constants::WEIGHT_REF_TIME_PER_MICROS,
			"Weight should be at least 100 µs."
		);
		// At most 50 ms.
		assert!(
			w.ref_time() <= 50u64 * constants::WEIGHT_REF_TIME_PER_MILLIS,
			"Weight should be at most 50 ms."
		);
	}
}
