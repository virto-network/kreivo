//! Weights for `pallet_nfts` (instance `CommunityMemberships`).
//!
//! PLACEHOLDER, NOT MEASURED ON KREIVO: a copy of `pallet_nfts::weights::SubstrateWeight` from
//! `pallet-nfts` 43.0.0 (polkadot-sdk stable2606), put in the shape `frame-omni-bencher` generates,
//! so the `CommunityMemberships` instance can be bound to this file. `/cmd bench --pallet pallet_nfts`
//! overwrites it with weights measured on Kreivo's reference hardware (both instances are
//! benchmarked in one run, and the bencher writes each to `pallet_nfts_<instance>.rs`).
//!
//! The storage items below are named `Nfts::*` as in the SDK's kitchensink runtime; in Kreivo
//! they belong to `CommunityMemberships`.
//!
//! The SDK weights are Copyright (C) Parity Technologies (UK) Ltd., licensed under Apache-2.0.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_nfts`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_nfts::WeightInfo for WeightInfo<T> {
	/// Storage: `Nfts::NextCollectionId` (r:1 w:1)
	/// Proof: `Nfts::NextCollectionId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionRoleOf` (r:0 w:1)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:0 w:1)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionAccount` (r:0 w:1)
	/// Proof: `Nfts::CollectionAccount` (`max_values`: None, `max_size`: Some(68), added: 2543, mode: `MaxEncodedLen`)
	fn create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `52`
		//  Estimated: `3549`
		// Minimum execution time: 29_866_000 picoseconds.
		Weight::from_parts(30_591_000, 3549)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `Nfts::NextCollectionId` (r:1 w:1)
	/// Proof: `Nfts::NextCollectionId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionRoleOf` (r:0 w:1)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:0 w:1)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionAccount` (r:0 w:1)
	/// Proof: `Nfts::CollectionAccount` (`max_values`: None, `max_size`: Some(68), added: 2543, mode: `MaxEncodedLen`)
	fn force_create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3549`
		// Minimum execution time: 15_636_000 picoseconds.
		Weight::from_parts(16_162_000, 3549)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemMetadataOf` (r:1 w:0)
	/// Proof: `Nfts::ItemMetadataOf` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:1)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:1001 w:1000)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1000 w:1000)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionMetadataOf` (r:0 w:1)
	/// Proof: `Nfts::CollectionMetadataOf` (`max_values`: None, `max_size`: Some(294), added: 2769, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:0 w:1)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionAccount` (r:0 w:1)
	/// Proof: `Nfts::CollectionAccount` (`max_values`: None, `max_size`: Some(68), added: 2543, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[0, 1000]`.
	/// The range of component `c` is `[0, 1000]`.
	/// The range of component `a` is `[0, 1000]`.
	fn destroy(m: u32, c: u32, a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `32112 + a * (366 ±0)`
		//  Estimated: `2523990 + a * (2954 ±0)`
		// Minimum execution time: 1_304_899_000 picoseconds.
		Weight::from_parts(1_206_247_822, 2523990)
			// Standard Error: 4_591
			.saturating_add(Weight::from_parts(49_431, 0).saturating_mul(m.into()))
			// Standard Error: 4_591
			.saturating_add(Weight::from_parts(24_260, 0).saturating_mul(c.into()))
			// Standard Error: 4_591
			.saturating_add(Weight::from_parts(7_104_726, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1004_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(1005_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(a.into()))
	}
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:1)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Account` (r:0 w:1)
	/// Proof: `Nfts::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	fn mint() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `362`
		//  Estimated: `4326`
		// Minimum execution time: 48_441_000 picoseconds.
		Weight::from_parts(49_363_000, 4326)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:1)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Account` (r:0 w:1)
	/// Proof: `Nfts::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	fn force_mint() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `362`
		//  Estimated: `4326`
		// Minimum execution time: 46_608_000 picoseconds.
		Weight::from_parts(47_860_000, 4326)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Nfts::Attribute` (r:1 w:0)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:1)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemMetadataOf` (r:1 w:0)
	/// Proof: `Nfts::ItemMetadataOf` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Account` (r:0 w:1)
	/// Proof: `Nfts::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemPriceOf` (r:0 w:1)
	/// Proof: `Nfts::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemAttributesApprovalsOf` (r:0 w:1)
	/// Proof: `Nfts::ItemAttributesApprovalsOf` (`max_values`: None, `max_size`: Some(681), added: 3156, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::PendingSwapOf` (r:0 w:1)
	/// Proof: `Nfts::PendingSwapOf` (`max_values`: None, `max_size`: Some(71), added: 2546, mode: `MaxEncodedLen`)
	fn burn() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `472`
		//  Estimated: `4326`
		// Minimum execution time: 51_492_000 picoseconds.
		Weight::from_parts(52_170_000, 4326)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:0)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:1 w:0)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:0)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Account` (r:0 w:2)
	/// Proof: `Nfts::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemPriceOf` (r:0 w:1)
	/// Proof: `Nfts::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::PendingSwapOf` (r:0 w:1)
	/// Proof: `Nfts::PendingSwapOf` (`max_values`: None, `max_size`: Some(71), added: 2546, mode: `MaxEncodedLen`)
	fn transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `501`
		//  Estimated: `4326`
		// Minimum execution time: 40_207_000 picoseconds.
		Weight::from_parts(41_360_000, 4326)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:0)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Item` (r:5000 w:5000)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// The range of component `i` is `[0, 5000]`.
	fn redeposit(i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `671 + i * (108 ±0)`
		//  Estimated: `3549 + i * (3336 ±0)`
		// Minimum execution time: 14_100_000 picoseconds.
		Weight::from_parts(14_412_000, 3549)
			// Standard Error: 20_746
			.saturating_add(Weight::from_parts(17_706_123, 0).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(i.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
			.saturating_add(Weight::from_parts(0, 3336).saturating_mul(i.into()))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:1)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	fn lock_item_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `343`
		//  Estimated: `3534`
		// Minimum execution time: 18_777_000 picoseconds.
		Weight::from_parts(19_084_000, 3534)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:1)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	fn unlock_item_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `343`
		//  Estimated: `3534`
		// Minimum execution time: 18_460_000 picoseconds.
		Weight::from_parts(18_907_000, 3534)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:0)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:1)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	fn lock_collection() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `248`
		//  Estimated: `3549`
		// Minimum execution time: 15_010_000 picoseconds.
		Weight::from_parts(15_523_000, 3549)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::OwnershipAcceptance` (r:1 w:1)
	/// Proof: `Nfts::OwnershipAcceptance` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionAccount` (r:0 w:2)
	/// Proof: `Nfts::CollectionAccount` (`max_values`: None, `max_size`: Some(68), added: 2543, mode: `MaxEncodedLen`)
	fn transfer_ownership() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `399`
		//  Estimated: `3593`
		// Minimum execution time: 24_987_000 picoseconds.
		Weight::from_parts(25_668_000, 3593)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionRoleOf` (r:2 w:4)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	fn set_team() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `277`
		//  Estimated: `6078`
		// Minimum execution time: 36_287_000 picoseconds.
		Weight::from_parts(36_653_000, 6078)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionAccount` (r:0 w:2)
	/// Proof: `Nfts::CollectionAccount` (`max_values`: None, `max_size`: Some(68), added: 2543, mode: `MaxEncodedLen`)
	fn force_collection_owner() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `219`
		//  Estimated: `3549`
		// Minimum execution time: 15_143_000 picoseconds.
		Weight::from_parts(15_551_000, 3549)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:0)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:0 w:1)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	fn force_collection_config() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `184`
		//  Estimated: `3549`
		// Minimum execution time: 11_145_000 picoseconds.
		Weight::from_parts(11_395_000, 3549)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:1)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	fn lock_item_properties() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `343`
		//  Estimated: `3534`
		// Minimum execution time: 17_174_000 picoseconds.
		Weight::from_parts(17_738_000, 3534)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:0)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:1 w:1)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	fn set_attribute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `447`
		//  Estimated: `3944`
		// Minimum execution time: 49_013_000 picoseconds.
		Weight::from_parts(50_124_000, 3944)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:1 w:1)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	fn force_set_attribute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `252`
		//  Estimated: `3944`
		// Minimum execution time: 23_738_000 picoseconds.
		Weight::from_parts(24_558_000, 3944)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Nfts::Attribute` (r:1 w:1)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:0)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	fn clear_attribute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `891`
		//  Estimated: `3944`
		// Minimum execution time: 45_483_000 picoseconds.
		Weight::from_parts(46_308_000, 3944)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Nfts::Item` (r:1 w:0)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemAttributesApprovalsOf` (r:1 w:1)
	/// Proof: `Nfts::ItemAttributesApprovalsOf` (`max_values`: None, `max_size`: Some(681), added: 3156, mode: `MaxEncodedLen`)
	fn approve_item_attributes() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `289`
		//  Estimated: `4326`
		// Minimum execution time: 15_153_000 picoseconds.
		Weight::from_parts(15_686_000, 4326)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::Item` (r:1 w:0)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemAttributesApprovalsOf` (r:1 w:1)
	/// Proof: `Nfts::ItemAttributesApprovalsOf` (`max_values`: None, `max_size`: Some(681), added: 3156, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:1001 w:1000)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 1000]`.
	fn cancel_item_attributes_approval(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `668 + n * (398 ±0)`
		//  Estimated: `4326 + n * (2954 ±0)`
		// Minimum execution time: 23_720_000 picoseconds.
		Weight::from_parts(24_121_000, 4326)
			// Standard Error: 4_581
			.saturating_add(Weight::from_parts(7_030_724, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(2_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(n.into()))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:0)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemMetadataOf` (r:1 w:1)
	/// Proof: `Nfts::ItemMetadataOf` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	fn set_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `447`
		//  Estimated: `3812`
		// Minimum execution time: 40_386_000 picoseconds.
		Weight::from_parts(40_903_000, 3812)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemMetadataOf` (r:1 w:1)
	/// Proof: `Nfts::ItemMetadataOf` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:0)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	fn clear_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `757`
		//  Estimated: `3812`
		// Minimum execution time: 38_438_000 picoseconds.
		Weight::from_parts(39_323_000, 3812)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionMetadataOf` (r:1 w:1)
	/// Proof: `Nfts::CollectionMetadataOf` (`max_values`: None, `max_size`: Some(294), added: 2769, mode: `MaxEncodedLen`)
	fn set_collection_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `306`
		//  Estimated: `3759`
		// Minimum execution time: 36_043_000 picoseconds.
		Weight::from_parts(36_498_000, 3759)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionMetadataOf` (r:1 w:1)
	/// Proof: `Nfts::CollectionMetadataOf` (`max_values`: None, `max_size`: Some(294), added: 2769, mode: `MaxEncodedLen`)
	fn clear_collection_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `624`
		//  Estimated: `3759`
		// Minimum execution time: 35_275_000 picoseconds.
		Weight::from_parts(35_879_000, 3759)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	fn approve_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `318`
		//  Estimated: `4326`
		// Minimum execution time: 18_190_000 picoseconds.
		Weight::from_parts(18_717_000, 4326)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	fn cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `326`
		//  Estimated: `4326`
		// Minimum execution time: 15_088_000 picoseconds.
		Weight::from_parts(15_564_000, 4326)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	fn clear_all_transfer_approvals() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `326`
		//  Estimated: `4326`
		// Minimum execution time: 14_365_000 picoseconds.
		Weight::from_parts(14_920_000, 4326)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::OwnershipAcceptance` (r:1 w:1)
	/// Proof: `Nfts::OwnershipAcceptance` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	fn set_accept_ownership() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3517`
		// Minimum execution time: 9_845_000 picoseconds.
		Weight::from_parts(10_228_000, 3517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:1)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:0)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	fn set_collection_max_supply() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `248`
		//  Estimated: `3549`
		// Minimum execution time: 16_666_000 picoseconds.
		Weight::from_parts(17_328_000, 3549)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:1)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	fn update_mint_settings() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `231`
		//  Estimated: `3538`
		// Minimum execution time: 15_969_000 picoseconds.
		Weight::from_parts(16_717_000, 3538)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::Item` (r:1 w:0)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:0)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemPriceOf` (r:0 w:1)
	/// Proof: `Nfts::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	fn set_price() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `426`
		//  Estimated: `4326`
		// Minimum execution time: 21_833_000 picoseconds.
		Weight::from_parts(22_580_000, 4326)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemPriceOf` (r:1 w:1)
	/// Proof: `Nfts::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:0)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:1 w:0)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:0)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Account` (r:0 w:2)
	/// Proof: `Nfts::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::PendingSwapOf` (r:0 w:1)
	/// Proof: `Nfts::PendingSwapOf` (`max_values`: None, `max_size`: Some(71), added: 2546, mode: `MaxEncodedLen`)
	fn buy_item() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `613`
		//  Estimated: `4326`
		// Minimum execution time: 50_016_000 picoseconds.
		Weight::from_parts(51_155_000, 4326)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// The range of component `n` is `[0, 10]`.
	fn pay_tips(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_695_000 picoseconds.
		Weight::from_parts(2_625_514, 0)
			// Standard Error: 5_213
			.saturating_add(Weight::from_parts(1_843_482, 0).saturating_mul(n.into()))
	}
	/// Storage: `Nfts::Item` (r:2 w:0)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::PendingSwapOf` (r:0 w:1)
	/// Proof: `Nfts::PendingSwapOf` (`max_values`: None, `max_size`: Some(71), added: 2546, mode: `MaxEncodedLen`)
	fn create_swap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `402`
		//  Estimated: `7662`
		// Minimum execution time: 18_600_000 picoseconds.
		Weight::from_parts(19_453_000, 7662)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::PendingSwapOf` (r:1 w:1)
	/// Proof: `Nfts::PendingSwapOf` (`max_values`: None, `max_size`: Some(71), added: 2546, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Item` (r:1 w:0)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	fn cancel_swap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `421`
		//  Estimated: `4326`
		// Minimum execution time: 18_476_000 picoseconds.
		Weight::from_parts(19_091_000, 4326)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Nfts::Item` (r:2 w:2)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::PendingSwapOf` (r:1 w:2)
	/// Proof: `Nfts::PendingSwapOf` (`max_values`: None, `max_size`: Some(71), added: 2546, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:0)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:2 w:0)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:2 w:0)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Account` (r:0 w:4)
	/// Proof: `Nfts::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemPriceOf` (r:0 w:2)
	/// Proof: `Nfts::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	fn claim_swap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `742`
		//  Estimated: `7662`
		// Minimum execution time: 81_691_000 picoseconds.
		Weight::from_parts(84_109_000, 7662)
			.saturating_add(T::DbWeight::get().reads(9_u64))
			.saturating_add(T::DbWeight::get().writes(10_u64))
	}
	/// Storage: `Nfts::CollectionRoleOf` (r:2 w:0)
	/// Proof: `Nfts::CollectionRoleOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Item` (r:1 w:1)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemConfigOf` (r:1 w:1)
	/// Proof: `Nfts::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:10 w:10)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemMetadataOf` (r:1 w:1)
	/// Proof: `Nfts::ItemMetadataOf` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Account` (r:0 w:1)
	/// Proof: `Nfts::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 10]`.
	fn mint_pre_signed(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `464`
		//  Estimated: `6078 + n * (2954 ±0)`
		// Minimum execution time: 125_533_000 picoseconds.
		Weight::from_parts(130_187_155, 6078)
			// Standard Error: 46_370
			.saturating_add(Weight::from_parts(33_019_771, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(6_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(n.into()))
	}
	/// Storage: `Nfts::Item` (r:1 w:0)
	/// Proof: `Nfts::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::ItemAttributesApprovalsOf` (r:1 w:1)
	/// Proof: `Nfts::ItemAttributesApprovalsOf` (`max_values`: None, `max_size`: Some(681), added: 3156, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::CollectionConfigOf` (r:1 w:0)
	/// Proof: `Nfts::CollectionConfigOf` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Collection` (r:1 w:1)
	/// Proof: `Nfts::Collection` (`max_values`: None, `max_size`: Some(84), added: 2559, mode: `MaxEncodedLen`)
	/// Storage: `Nfts::Attribute` (r:10 w:10)
	/// Proof: `Nfts::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 10]`.
	fn set_attributes_pre_signed(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `496`
		//  Estimated: `4326 + n * (2954 ±0)`
		// Minimum execution time: 65_098_000 picoseconds.
		Weight::from_parts(74_929_657, 4326)
			// Standard Error: 61_212
			.saturating_add(Weight::from_parts(31_387_885, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(2_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(n.into()))
	}
}
