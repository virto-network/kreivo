use crate::apis::{KreivoAPI, KreivoApisError, MembershipsAPI};
use crate::runtime::impls::ChainExtensionDispatch;
use crate::runtime::types::MembershipsApiInfo;
use crate::runtime::RuntimeKreivoAPI;
use crate::Config;
use alloc::vec::Vec;
use frame_support::traits::ConstU32;
use frame_support::BoundedVec;
use pallet_contracts::chain_extension::Ext;
use parity_scale_codec::Encode;

type MembershipsAPIOf<T, E> = <RuntimeKreivoAPI<T> as KreivoAPI<E>>::Memberships;

impl<T, E> ChainExtensionDispatch<E> for MembershipsApiInfo<T>
where
	T: Config,
	E: Ext<T = T>,
{
	fn call(&self, ext: &E) -> Result<Vec<u8>, KreivoApisError> {
		match self {
			MembershipsApiInfo::AssignMembership { who } => {
				Ok(MembershipsAPIOf::<T, E>::assign_membership(ext, who).encode())
			}
			MembershipsApiInfo::MembershipOf { who } => Ok(MembershipsAPIOf::<T, E>::membership_of(ext, who).encode()),
			MembershipsApiInfo::RankOf { id } => Ok(MembershipsAPIOf::<T, E>::rank_of(ext, id).encode()),
			MembershipsApiInfo::Attribute { id, key } => {
				Ok(MembershipsAPIOf::<T, E>::attribute::<_, BoundedVec<u8, ConstU32<256>>>(ext, id, key).encode())
			}
			MembershipsApiInfo::SetAttribute { id, key, value } => {
				MembershipsAPIOf::<T, E>::set_attribute(ext, id, key, value).map(|v| v.encode())
			}
			MembershipsApiInfo::ClearAttribute { id, key } => {
				MembershipsAPIOf::<T, E>::clear_attribute(ext, id, key).map(|v| v.encode())
			}
			MembershipsApiInfo::FilterMembership { who, key, value } => {
				Ok(MembershipsAPIOf::<T, E>::filter_membership(ext, who, key, value).encode())
			}
		}
	}
}
