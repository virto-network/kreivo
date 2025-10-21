use super::*;

#[repr(u16)]
#[derive(TypeInfo, Encode, Decode, Clone, Debug, PartialEq, TryFromPrimitive)]
pub enum MembershipsApiError {
	/// The contract does not have an associated `Group`, then it's not
	/// possible to use the Memberships APIs.
	NoGroup,
	/// The specified membership is not found.
	UnknownMembership,
	/// There are no available memberships, thus a new member cannot
	/// be added.
	CannotAddMember,
	/// It is not possible to set an attribute on a membership.
	FailedToSetAttribute,
}

impl From<MembershipsApiError> for KreivoApisError {
	fn from(error: MembershipsApiError) -> Self {
		KreivoApisError::Memberships(error)
	}
}
