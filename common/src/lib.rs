#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "nightly", feature(ascii_char))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod multilocation_asset_id;
mod payment_id;

pub use multilocation_asset_id::{FungibleAssetLocation, NetworkId, Para};
pub use payment_id::PaymentId;

#[cfg(feature = "runtime")]
pub use multilocation_asset_id::runtime::AsFungibleAssetLocation;

pub type CommunityId = u16;
pub type MembershipId = u32;

pub mod listings {
	pub type InventoryId = u32;
	pub type ItemId = u64;
}
