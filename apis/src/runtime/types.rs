use super::*;
use config::AccountIdOf;
use frame_support::pallet_prelude::DispatchError;
use frame_support::{CloneNoBound, DebugNoBound};
use pallet_contracts::chain_extension::BufInBufOutState;

mod assets;
pub use assets::*;
mod listings;
pub use listings::*;

mod memberships;
pub use memberships::*;

#[derive(CloneNoBound, DebugNoBound)]
pub enum ApiInfo<T>
where
	T: Config,
{
	Assets(AssetsApiInfo<T>),
	Listings(ListingsApiInfo<T>),
	Memberships(MembershipsApiInfo<T>),
}

impl<T, E> TryFrom<&mut Environment<'_, '_, E, BufInBufOutState>> for ApiInfo<T>
where
	T: Config,
	E: Ext<T = T>,
{
	type Error = DispatchError;

	fn try_from(env: &mut Environment<'_, '_, E, BufInBufOutState>) -> Result<Self, Self::Error> {
		match env.func_id() {
			0x0000..0x0100 => env.try_into().map(|api_info| Self::Assets(api_info)),
			0x0100..0x0200 => env.try_into().map(|api_info| Self::Listings(api_info)),
			0x0200..0x0300 => env.try_into().map(|api_info| Self::Memberships(api_info)),
			id => {
				log::error!("Called an unregistered `func_id`: {id:}");
				Err(DispatchError::Other("Unimplemented func_id"))
			}
		}
	}
}
