use parity_scale_codec::{Decode, Encode, Error};
use scale_info::TypeInfo;

mod assets;
mod listings;

pub use assets::*;
pub use listings::*;

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
	Listings(ListingsApiError),
}

impl From<KreivoApisError> for KreivoApisErrorCode {
	fn from(error: KreivoApisError) -> KreivoApisErrorCode {
		match error {
			KreivoApisError::UnknownError => Self(1),
			KreivoApisError::ExtQueryError => Self(2),
			KreivoApisError::Assets(e) => Self((1u32 << 16) | e as u16 as u32),
			KreivoApisError::Listings(e) => Self((2u32 << 16) | e as u16 as u32),
		}
	}
}

impl From<KreivoApisErrorCode> for KreivoApisError {
	fn from(value: KreivoApisErrorCode) -> Self {
		match value.0 {
			0x00000002 => KreivoApisError::ExtQueryError,
			0x00010000..0x00020000 => {
				TryFrom::<KreivoApisErrorCode>::try_from(KreivoApisErrorCode(value.0 & 0x0000ffff))
					.map(KreivoApisError::Assets)
					.unwrap_or(KreivoApisError::UnknownError)
			}
			0x00020000..0x00030000 => {
				TryFrom::<KreivoApisErrorCode>::try_from(KreivoApisErrorCode(value.0 & 0x0000ffff))
					.map(KreivoApisError::Listings)
					.unwrap_or(KreivoApisError::UnknownError)
			}
			_ => KreivoApisError::UnknownError,
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

		test_error_code_conversion!(ListingsApiError::NoMerchantId);
		test_error_code_conversion!(ListingsApiError::UnknownInventory);
		test_error_code_conversion!(ListingsApiError::FailedToCreateInventory);
		test_error_code_conversion!(ListingsApiError::ArchivedInventory);
		test_error_code_conversion!(ListingsApiError::FailedToArchiveInventory);
		test_error_code_conversion!(ListingsApiError::FailedToSetAttribute);
		test_error_code_conversion!(ListingsApiError::UnknownItem);
		test_error_code_conversion!(ListingsApiError::NotForResale);
		test_error_code_conversion!(ListingsApiError::ItemNonTransferable);
		test_error_code_conversion!(ListingsApiError::FailedToSetAttribute);
	}
}
