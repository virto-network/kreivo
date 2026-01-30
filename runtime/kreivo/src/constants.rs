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

	/// `pallet_revive` requires this specific `WeightToFee` implementation.
	///
	/// This is needed because we make certain assumptions about how weight
	/// is mapped to fees. Enforced at compile time.
	pub type WeightToFee = pallet_revive::evm::fees::BlockRatioFee<
		// p
		{ super::currency::CENTS },
		// q
		{ 100 * ExtrinsicBaseWeight::get().ref_time() as u128 },
		crate::Runtime,
		Balance,
	>;
}

pub mod locations {
	pub const ASSET_HUB_ID: u32 = 1000;
}
