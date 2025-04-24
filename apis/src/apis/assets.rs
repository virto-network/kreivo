//! # Assets APIs
//!
//! Facilitate transactions using arbitrary assets between external callers and
//! the application.
//!
//! ## Methods
//!
//! The supported methods are:
//!
//! - **[`deposit`][AssetsAPI::deposit]:** Receives an `amount` of a certain
//!   `asset` from the caller of the application. Then, deposits that amount
//!   into the balance of the application asset account.
//! - **[`transfer`][AssetsAPI::transfer]:** Transfers an `amount` of a certain
//!   `asset` to a `beneficiary`.

use crate::apis::error::KreivoApisError;
use core::fmt;
use frame_support::Parameter;
use parity_scale_codec::{Codec, EncodeLike};

/// An API for transacting with arbitrary assets.
pub trait AssetsAPI<Ext> {
	type AccountId: Codec + EncodeLike + Clone + Eq + fmt::Debug;
	type AssetId: Parameter;
	type Balance: Parameter + Copy;

	/// Returns the balance of an asset account.
	fn balance(e: &Ext, asset: Self::AssetId, who: &Self::AccountId) -> Self::Balance;

	/// Receives an `amount` of a certain `asset` from the caller of the
	/// application. Then, deposits that amount into the balance of the
	/// application asset account.
	fn deposit(e: &Ext, asset: Self::AssetId, amount: Self::Balance) -> Result<Self::Balance, KreivoApisError>;

	/// Transfers an `amount` of a certain `asset` to a `beneficiary`.
	fn transfer(
		e: &Ext,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError>;
}
