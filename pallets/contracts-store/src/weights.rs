#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn publish() -> Weight;
	fn set_parameters() -> Weight;
	fn publish_upgrade() -> Weight;
	fn request_license() -> Weight;
}

/// Weights for pallet_contracts_store using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn publish() -> Weight {
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(14))
	}
	fn set_parameters() -> Weight {
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(14))
	}
	fn publish_upgrade() -> Weight {
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(14))
	}
	fn request_license() -> Weight {
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(14))
	}
}

impl WeightInfo for () {
	fn publish() -> Weight {
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(14))
	}
	fn set_parameters() -> Weight {
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(14))
	}
	fn publish_upgrade() -> Weight {
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(14))
	}
	fn request_license() -> Weight {
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(14))
	}
}