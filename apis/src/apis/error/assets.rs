use super::*;

#[repr(u16)]
#[derive(TypeInfo, Encode, Decode, Clone, Debug, PartialEq, TryFromPrimitive)]
pub enum AssetsApiError {
	CannotDeposit,
	CannotTransfer,
}

impl From<AssetsApiError> for KreivoApisError {
	fn from(error: AssetsApiError) -> Self {
		KreivoApisError::Assets(error)
	}
}
