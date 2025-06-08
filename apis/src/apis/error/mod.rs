use num_enum::TryFromPrimitive;
use parity_scale_codec::{Decode, Encode, Error};
use scale_info::TypeInfo;

mod assets;
mod listings;
mod memberships;

pub use assets::*;
pub use listings::*;
pub use memberships::*;

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
	Memberships(MembershipsApiError),
}

impl From<KreivoApisError> for KreivoApisErrorCode {
	fn from(error: KreivoApisError) -> KreivoApisErrorCode {
		let inner = |e: u16| e as u32;
		Self(match error {
			KreivoApisError::UnknownError => 1,
			KreivoApisError::ExtQueryError => 2,
			KreivoApisError::Assets(e) => 0x00010000 | inner(e as u16),
			KreivoApisError::Listings(e) => 0x00020000 | inner(e as u16),
			KreivoApisError::Memberships(e) => 0x00030000 | inner(e as u16),
		})
	}
}

impl From<KreivoApisErrorCode> for KreivoApisError {
	fn from(value: KreivoApisErrorCode) -> Self {
		let inner = (value.0 & 0x0000ffff) as u16;
		match value.0 {
			0x00000002 => Some(KreivoApisError::ExtQueryError),
			0x00010000..0x00020000 => TryFrom::<u16>::try_from(inner).ok().map(KreivoApisError::Assets),
			0x00020000..0x00030000 => TryFrom::<u16>::try_from(inner).ok().map(KreivoApisError::Listings),
			0x00030000..0x00040000 => TryFrom::<u16>::try_from(inner).ok().map(KreivoApisError::Memberships),
			_ => None,
		}
		.unwrap_or(KreivoApisError::UnknownError)
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

		test_error_code_conversion!(MembershipsApiError::NoGroup);
		test_error_code_conversion!(MembershipsApiError::UnknownMembership);
		test_error_code_conversion!(MembershipsApiError::CannotAddMember);
		test_error_code_conversion!(MembershipsApiError::FailedToSetAttribute);
	}
}
