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

use frame_support::pallet_prelude::DispatchError;
use frame_support::Parameter;

/// An API for transacting with arbitrary assets.
pub trait AssetsAPI<Env> {
	type AccountId: Parameter;
	type AssetId: Parameter;
	type Balance: Parameter + Copy;

	/// Returns the balance of an asset account.
	fn balance(&self, asset: Self::AssetId, who: &Self::AccountId) -> Self::Balance;

	/// Receives an `amount` of a certain `asset` from the caller of the
	/// application. Then, deposits that amount into the balance of the
	/// application asset account.
	fn deposit(&self, asset: Self::AssetId, amount: Self::Balance) -> Result<Self::Balance, DispatchError>;

	/// Transfers an `amount` of a certain `asset` to a `beneficiary`.
	fn transfer(
		&self,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, DispatchError>;
}
