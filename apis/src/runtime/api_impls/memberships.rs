use crate::apis::{KreivoApisError, MembershipsAPI, MembershipsApiError};
use crate::runtime::config::{GroupInfo, MembershipOf};
use crate::Config;
use core::marker::PhantomData;
use frame_contrib_traits::memberships::{Attributes, GenericRank, InspectEnumerable, Manager, Rank};
use frame_support::Parameter;
use pallet_contracts::chain_extension::Ext;
use parity_scale_codec::Encode;

/// A helper structure that implements [`MembershipsAPI`] in the context of the
/// Runtime.
pub struct RuntimeMembershipsAPI<T>(PhantomData<T>);

impl<T, Env> MembershipsAPI<Env> for RuntimeMembershipsAPI<T>
where
	T: Config,
	Env: Ext<T = T>,
{
	type AccountId = T::AccountId;
	type MembershipId = MembershipOf<T>;
	type Rank = GenericRank;

	fn assign_membership(env: &Env, who: &Self::AccountId) -> Result<(), KreivoApisError> {
		let group = T::GroupInfo::maybe_group(env.address()).ok_or(MembershipsApiError::NoGroup)?;
		let membership = T::Memberships::group_available_memberships(&group)
			.next()
			.ok_or(MembershipsApiError::CannotAddMember)?;
		T::Memberships::assign(&group, &membership, who).map_err(|_| MembershipsApiError::CannotAddMember.into())
	}

	fn membership_of(env: &Env, who: &Self::AccountId) -> Option<Self::MembershipId> {
		let group = T::GroupInfo::maybe_group(env.address())?;
		T::Memberships::memberships_of(who, Some(group)).map(|(_, m)| m).next()
	}

	fn rank_of(env: &Env, id: &Self::MembershipId) -> Option<Self::Rank> {
		let group = T::GroupInfo::maybe_group(env.address())?;
		T::Memberships::rank_of(&group, id)
	}

	fn attribute<K: Encode, V: Parameter>(env: &Env, id: &Self::MembershipId, key: &K) -> Option<V> {
		let group = T::GroupInfo::maybe_group(env.address())?;
		T::Memberships::membership_attribute(&group, id, key)
	}

	fn set_attribute<K: Encode, V: Encode>(
		env: &Env,
		id: &Self::MembershipId,
		key: &K,
		value: &V,
	) -> Result<(), KreivoApisError> {
		let group = T::GroupInfo::maybe_group(env.address()).ok_or(MembershipsApiError::NoGroup)?;
		T::Memberships::set_membership_attribute(&group, id, key, value)
			.map_err(|_| MembershipsApiError::FailedToSetAttribute.into())
	}

	fn clear_attribute<K: Encode>(env: &Env, id: &Self::MembershipId, key: &K) -> Result<(), KreivoApisError> {
		let group = T::GroupInfo::maybe_group(env.address()).ok_or(MembershipsApiError::NoGroup)?;
		T::Memberships::clear_membership_attribute(&group, id, key)
			.map_err(|_| MembershipsApiError::FailedToSetAttribute.into())
	}

	fn filter_membership<K: Encode, V: Parameter>(
		env: &Env,
		who: &Self::AccountId,
		key: &K,
		value: &V,
	) -> Option<Self::MembershipId> {
		let group = T::GroupInfo::maybe_group(env.address())?;
		T::Memberships::memberships_of(who, Some(group))
			.find(|(g, m)| {
				let att = T::Memberships::membership_attribute::<K, V>(g, m, key);
				att.is_some_and(|v| &v == value)
			})
			.map(|(_, m)| m)
	}
}
