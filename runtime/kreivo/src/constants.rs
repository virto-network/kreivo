// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod currency {
	use polkadot_core_primitives::Balance;
	use runtime_constants as constants;

	/// The existential deposit. Set to 1/10 of its parent Relay Chain.
	pub const EXISTENTIAL_DEPOSIT: Balance = constants::currency::EXISTENTIAL_DEPOSIT / 10;

	pub const UNITS: Balance = constants::currency::UNITS;
	pub const CENTS: Balance = constants::currency::CENTS;
	pub const GRAND: Balance = constants::currency::GRAND;
	pub const MILLICENTS: Balance = constants::currency::MILLICENTS;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		// map to 1/100 of what the kusama relay chain charges (v9020)
		constants::currency::deposit(items, bytes) / 100
	}
}

/// Fee-related.
pub mod fee {
	use frame_support::weights::constants::ExtrinsicBaseWeight;
	use polkadot_core_primitives::Balance;

	use core::marker::PhantomData;
	use frame_support::{
		traits::Get,
		weights::{Weight, WeightToFee as WeightToFeeT},
	};
	use sp_runtime::{FixedPointNumber, FixedU128, SaturatedConversion, Saturating};

	/// The block limits that fees are priced against: 2s of `ref_time` and a 5 MiB PoV, Kreivo's
	/// fixed block limits before block bundling. Block limits are now a share of a core, and
	/// change with the cores the parachain has, so fees don't follow them.
	pub const FEE_REFERENCE_BLOCK_WEIGHT: Weight = Weight::from_parts(
		2 * frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND,
		5 * 1024 * 1024,
	);

	/// Charges `P / Q` per unit of `ref_time`, and proof size at the same price scaled by the
	/// `ref_time` to `proof_size` ratio of `Limits`; a transaction pays for whichever is larger.
	///
	/// This is `pallet_revive::evm::fees::BlockRatioFee`, which Kreivo used while it had
	/// pallet-revive, with the ratio taken from fixed `Limits` instead of the block weights.
	pub struct BlockRatioFee<const P: u128, const Q: u128, Limits>(PhantomData<Limits>);

	impl<const P: u128, const Q: u128, Limits: Get<Weight>> BlockRatioFee<P, Q, Limits> {
		const REF_TIME_TO_FEE: FixedU128 = {
			assert!(P > 0 && Q > 0);
			FixedU128::from_rational(P, Q)
		};

		fn proof_size_to_fee() -> FixedU128 {
			let max_weight = Limits::get();
			let ratio = FixedU128::from_rational(max_weight.ref_time().into(), max_weight.proof_size().into());
			Self::REF_TIME_TO_FEE.saturating_mul(ratio)
		}
	}

	impl<const P: u128, const Q: u128, Limits: Get<Weight>> WeightToFeeT for BlockRatioFee<P, Q, Limits> {
		type Balance = Balance;

		fn weight_to_fee(weight: &Weight) -> Balance {
			let ref_time_fee = Self::REF_TIME_TO_FEE.saturating_mul_int(Balance::saturated_from(weight.ref_time()));
			let proof_size_fee =
				Self::proof_size_to_fee().saturating_mul_int(Balance::saturated_from(weight.proof_size()));
			ref_time_fee.max(proof_size_fee)
		}
	}

	pub type WeightToFee = BlockRatioFee<
		// p
		{ super::currency::CENTS },
		// q
		{ 100 * ExtrinsicBaseWeight::get().ref_time() as u128 },
		FeeReferenceBlockWeight,
	>;

	frame_support::parameter_types! {
		pub const FeeReferenceBlockWeight: Weight = FEE_REFERENCE_BLOCK_WEIGHT;
	}
}

pub mod locations {
	pub const ASSET_HUB_ID: u32 = 1000;
}
