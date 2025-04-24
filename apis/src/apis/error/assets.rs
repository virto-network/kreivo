use super::*;

#[repr(u16)]
#[derive(TypeInfo, Encode, Decode, Clone, Debug, PartialEq)]
pub enum AssetsApiError {
	CannotDeposit,
	CannotTransfer,
}

impl From<AssetsApiError> for KreivoApisError {
	fn from(error: AssetsApiError) -> Self {
		KreivoApisError::Assets(error)
	}
}

impl TryFrom<KreivoApisErrorCode> for AssetsApiError {
	type Error = ();

	fn try_from(value: KreivoApisErrorCode) -> Result<Self, Self::Error> {
		match value.0 {
			0 => Ok(AssetsApiError::CannotDeposit),
			1 => Ok(AssetsApiError::CannotTransfer),
			_ => Err(()),
		}
	}
}
