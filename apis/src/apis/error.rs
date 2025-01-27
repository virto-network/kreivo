use parity_scale_codec::{Decode, Encode, Error};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Debug, Clone, Copy)]
pub struct KreivoApisErrorCode(u32);

impl From<KreivoApisErrorCode> for u32 {
	fn from(value: KreivoApisErrorCode) -> Self {
		value.0
	}
}

impl From<u32> for KreivoApisErrorCode {
	fn from(value: u32) -> Self {
		Self(value)
	}
}

impl From<Error> for KreivoApisErrorCode {
	fn from(_: Error) -> Self {
		panic!("encountered unexpected invalid SCALE encoding")
	}
}

#[derive(TypeInfo, Encode, Decode, Clone, Debug, PartialEq)]
pub enum KreivoApisError {
	UnknownError,
	ExtQueryError,
	Assets(AssetsApiError),
}

impl From<KreivoApisError> for KreivoApisErrorCode {
	fn from(error: KreivoApisError) -> KreivoApisErrorCode {
		match error {
			KreivoApisError::UnknownError => Self(1),
			KreivoApisError::ExtQueryError => Self(2),
			KreivoApisError::Assets(e) => Self(1u32 << 16 | e as u16 as u32),
		}
	}
}

impl From<AssetsApiError> for KreivoApisError {
	fn from(error: AssetsApiError) -> Self {
		KreivoApisError::Assets(error)
	}
}

impl From<KreivoApisErrorCode> for KreivoApisError {
	fn from(value: KreivoApisErrorCode) -> Self {
		match value.0 {
			0x00000002 => KreivoApisError::ExtQueryError,
			0x00010000..0x0001ffff => {
				TryFrom::<KreivoApisErrorCode>::try_from(KreivoApisErrorCode(value.0 & 0x0000ffff))
					.map(KreivoApisError::Assets)
					.unwrap_or(KreivoApisError::UnknownError)
			}
			_ => KreivoApisError::UnknownError,
		}
	}
}

#[repr(u16)]
#[derive(TypeInfo, Encode, Decode, Clone, Debug, PartialEq)]
pub enum AssetsApiError {
	CannotDeposit,
	CannotTransfer,
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

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! test_error_code_conversion {
		($error: expr) => {{
			let error: KreivoApisError = $error.into();
			let error_code: KreivoApisErrorCode = error.clone().into();
			let error_back: KreivoApisError = error_code.clone().into();
			assert_eq!(error, error_back);
		}};
	}

	#[test]
	fn convert_from_error_to_error_code_back_to_error_works() {
		test_error_code_conversion!(KreivoApisError::UnknownError);
		test_error_code_conversion!(KreivoApisError::ExtQueryError);

		test_error_code_conversion!(AssetsApiError::CannotDeposit);
		test_error_code_conversion!(AssetsApiError::CannotTransfer);
	}
}
