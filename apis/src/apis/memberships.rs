//! # Memberships APIs
//!
//! Facilitate to manage memberships' attributes and assign new members.

use crate::apis::KreivoApisError;
use core::fmt;
use frame_support::Parameter;
use parity_scale_codec::{Codec, Encode, EncodeLike};

/// An API for managing the memberships of a group. It is assumed that the `Env`
/// context must provide the info of which the group is.
pub trait MembershipsAPI<Env> {
	type AccountId: Codec + EncodeLike + Clone + Eq + fmt::Debug;
	type MembershipId: Parameter;
	type Rank: Parameter + Copy;

	/// Assigns a membership associated to the group to [`who`].
	///
	/// Returns an error if the group doesn't have enough memberships and cannot
	/// assign a new one to [`who`].
	fn assign_membership(env: &Env, who: &Self::AccountId) -> Result<(), KreivoApisError>;

	/// Returns the first found membership of [`who`] if any in the `group`, or
	/// [`None`] otherwise (also returns `None` if there's no group).
	fn membership_of(env: &Env, who: &Self::AccountId) -> Option<Self::MembershipId>;

	/// Returns the first membership of [`who`] if any, or [`None`] otherwise.
	fn rank_of(env: &Env, id: &Self::MembershipId) -> Option<Self::Rank>;

	/// Returns the value of the attribute (with [`key`]) for [`id`] if any, or
	/// [`None`] otherwise.
	fn attribute<K: Encode, V: Parameter>(env: &Env, id: &Self::MembershipId, key: &K) -> Option<V>;

	/// Attempts setting a [`value`] for the attribute (with [`key`]) for the
	/// membership [`id`].
	fn set_attribute<K: Encode, V: Encode>(
		env: &Env,
		id: &Self::MembershipId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError>;

	/// Attempts clearing the attribute (with [`key`]) for the membership
	/// [`id`].
	fn clear_attribute<K: Encode>(env: &Env, id: &Self::MembershipId, key: &K) -> Result<(), KreivoApisError>;

	/// Looks for the first hit of a membership for [`who`] that contains an
	/// attribute (with [`key`]) which matches with [`value`]. Returns [`None`]
	/// otherwise.
	fn filter_membership<K: Encode, V: Parameter>(
		env: &Env,
		who: &Self::AccountId,
		key: &K,
		value: &V,
	) -> Option<Self::MembershipId>;
}
