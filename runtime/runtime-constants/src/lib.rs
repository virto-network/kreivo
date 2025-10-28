// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod genesis_presets;
pub mod weights;

/// Money matters.
pub mod currency {
	use polkadot_primitives::Balance;

	/// The existential deposit.
	pub const EXISTENTIAL_DEPOSIT: Balance = CENTS;

	#[cfg(not(feature = "paseo"))]
	pub const UNITS: Balance = 1_000_000_000_000;
	#[cfg(feature = "paseo")]
	pub const UNITS: Balance = 10_000_000_000;

	#[cfg(not(feature = "paseo"))]
	pub const QUID: Balance = UNITS / 30;

	#[cfg(not(feature = "paseo"))]
	pub const CENTS: Balance = QUID / 100;
	#[cfg(feature = "paseo")]
	pub const CENTS: Balance = UNITS / 100;

	#[cfg(not(feature = "paseo"))]
	pub const GRAND: Balance = 1_000 * QUID;
	#[cfg(feature = "paseo")]
	pub const GRAND: Balance = 1_000 * UNITS;

	pub const MILLICENTS: Balance = CENTS / 1_000;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 2_000 * CENTS + (bytes as Balance) * 100 * MILLICENTS
	}
}

pub mod async_backing_params {
	use polkadot_primitives::Moment;

	/// Build with an offset of 1 behind the relay chain best block.
	#[cfg(not(feature = "try-runtime"))]
	pub const RELAY_PARENT_OFFSET: u32 = 2;
	#[cfg(feature = "try-runtime")]
	pub const RELAY_PARENT_OFFSET: u32 = 0;
	/// Maximum number of blocks simultaneously accepted by the Runtime, not yet
	/// included into the relay chain.
	pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = (2 + RELAY_PARENT_OFFSET) * BLOCK_PROCESSING_VELOCITY + 1;
	/// The upper limit of how many parachain blocks are processed by the relay chain per
	/// parent. Limits the number of blocks authored per slot. This determines the minimum
	/// block time of the parachain:
	#[cfg(feature = "paseo")]
	pub const BLOCK_PROCESSING_VELOCITY: u32 = 3;
	#[cfg(not(feature = "paseo"))]
	pub const BLOCK_PROCESSING_VELOCITY: u32 = 12;
	/// Relay chain slot duration, in milliseconds.
	pub const RELAY_CHAIN_SLOT_DURATION_MILLIS: Moment = 6_000;
}

/// Time and blocks.
pub mod time {
	use crate::async_backing_params::{BLOCK_PROCESSING_VELOCITY, RELAY_CHAIN_SLOT_DURATION_MILLIS};
	use polkadot_primitives::{BlockNumber, Moment};

	pub const MILLISECS_PER_BLOCK: Moment = RELAY_CHAIN_SLOT_DURATION_MILLIS / BLOCK_PROCESSING_VELOCITY as u64;
	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
	pub const WEEKS: BlockNumber = DAYS * 7;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe
	// blocks. The choice of is done in accordance to the slot duration and expected
	// target block time, for safely resisting network delays of maximum two
	// seconds. <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}

/// Fee-related.
pub mod fee {
	use crate::weights::ExtrinsicBaseWeight;
	use frame_support::weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial};
	use polkadot_primitives::Balance;
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	/// Handles converting a weight scalar to a fee value, based on the scale
	/// and granularity of the node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, `MAXIMUM_BLOCK_WEIGHT`]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some
	/// examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to
	///     be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Kusama, extrinsic base weight (smallest non-zero weight) is mapped to 1/10
			// CENT:
			let p = super::currency::CENTS;
			let q = 10 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

/// System Parachains.
pub mod system_parachain {
	use polkadot_primitives::Id;
	use xcm_builder::IsChildSystemParachain;

	/// Asset Hub parachain ID.
	pub const ASSET_HUB_ID: u32 = 1000;
	/// Encointer parachain ID.
	pub const ENCOINTER_ID: u32 = 1001;
	/// Bridge Hub parachain ID.
	pub const BRIDGE_HUB_ID: u32 = 1002;
	/// People parachain ID.
	pub const PEOPLE_ID: u32 = 1004;
	/// Brokerage parachain ID.
	pub const CORETIME_ID: u32 = 1005;

	pub type SystemParachains = IsChildSystemParachain<Id>;
}

/// Kusama Treasury pallet instance.
pub const TREASURY_PALLET_ID: u8 = 18;

#[cfg(test)]
mod tests {
	use super::{
		currency::{CENTS, MILLICENTS},
		fee::WeightToFee,
	};
	use crate::weights::ExtrinsicBaseWeight;
	use frame_support::weights::WeightToFee as WeightToFeeT;
	use polkadot_runtime_common::MAXIMUM_BLOCK_WEIGHT;

	#[test]
	// Test that the fee for `MAXIMUM_BLOCK_WEIGHT` of weight has sane bounds.
	fn full_block_fee_is_correct() {
		// A full block should cost between 1,000 and 10,000 CENTS.
		let full_block = WeightToFee::weight_to_fee(&MAXIMUM_BLOCK_WEIGHT);
		assert!(full_block >= 1_000 * CENTS);
		assert!(full_block <= 10_000 * CENTS);
	}

	#[test]
	// This function tests that the fee for `ExtrinsicBaseWeight` of weight is
	// correct
	fn extrinsic_base_fee_is_correct() {
		// `ExtrinsicBaseWeight` should cost 1/10 of a CENT
		println!("Base: {}", ExtrinsicBaseWeight::get());
		let x = WeightToFee::weight_to_fee(&ExtrinsicBaseWeight::get());
		let y = CENTS / 10;
		assert!(x.max(y) - x.min(y) < MILLICENTS);
	}
}
