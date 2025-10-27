use super::*;

use crate::runtime::config::MembershipOf;
use frame_support::pallet_prelude::ConstU32;
use frame_support::sp_runtime::DispatchError;
use frame_support::BoundedVec;
use pallet_contracts::chain_extension::{BufInBufOutState, Environment, Ext};

#[derive(Encode, Decode, Clone, DebugNoBound)]
pub enum MembershipsApiInfo<T: Config> {
	AssignMembership {
		who: AccountIdOf<T>,
	},
	MembershipOf {
		who: AccountIdOf<T>,
	},
	RankOf {
		id: MembershipOf<T>,
	},
	Attribute {
		id: MembershipOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
	},
	SetAttribute {
		id: MembershipOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
		value: BoundedVec<u8, ConstU32<256>>,
	},
	ClearAttribute {
		id: MembershipOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
	},
	FilterMembership {
		who: AccountIdOf<T>,
		key: BoundedVec<u8, ConstU32<256>>,
		value: BoundedVec<u8, ConstU32<256>>,
	},
}

impl<T, E> TryFrom<&mut Environment<'_, '_, E, BufInBufOutState>> for MembershipsApiInfo<T>
where
	T: Config,
	E: Ext<T = T>,
{
	type Error = DispatchError;

	fn try_from(env: &mut Environment<'_, '_, E, BufInBufOutState>) -> Result<Self, Self::Error> {
		match env.func_id() {
			0x0200 => {
				let who = env.read_as()?;
				Ok(MembershipsApiInfo::AssignMembership { who })
			}
			0x0201 => {
				let who = env.read_as()?;
				Ok(MembershipsApiInfo::MembershipOf { who })
			}
			0x0202 => {
				let id = env.read_as()?;
				Ok(MembershipsApiInfo::RankOf { id })
			}
			0x0203 => {
				let (id, key) = env.read_as()?;
				Ok(MembershipsApiInfo::Attribute { id, key })
			}
			0x0204 => {
				let (id, key, value) = env.read_as()?;
				Ok(MembershipsApiInfo::SetAttribute { id, key, value })
			}
			0x0205 => {
				let (id, key) = env.read_as()?;
				Ok(MembershipsApiInfo::ClearAttribute { id, key })
			}
			0x0206 => {
				let (who, key, value) = env.read_as()?;
				Ok(MembershipsApiInfo::FilterMembership { who, key, value })
			}
			id => {
				log::error!("Called an unregistered `func_id`: {id:}");
				Err(DispatchError::Other("Unimplemented func_id"))
			}
		}
	}
}
