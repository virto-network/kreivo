use super::apis::*;

use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::DefaultNoBound;
use pallet_contracts::chain_extension::{ChainExtension, Environment, Ext, InitState, RetVal};

mod config;
pub use config::{Config, MerchantIdInfo};

mod impls;
use impls::*;

mod types;
use types::*;

mod api_impls {
	use super::*;
	mod assets;
	mod listings;
	pub use assets::*;
	pub use listings::*;
}
use api_impls::*;

/// A helper structure that implements [`KreivoAPI`] in the context of the
/// Runtime.
struct RuntimeKreivoAPI<T>(PhantomData<T>);

impl<T, E> KreivoAPI<E> for RuntimeKreivoAPI<T>
where
	T: Config,
	E: Ext<T = T>,
{
	type Assets = RuntimeAssetsAPI<T>;
	type Listings = RuntimeListingsAPI<T>;
}

/// A [`ChainExtension`] that implements the [`KreivoAPI`]s.
#[derive(DefaultNoBound)]
pub struct KreivoChainExtensions<T>(PhantomData<T>);

impl<T> ChainExtension<T> for KreivoChainExtensions<T>
where
	T: Config,
{
	fn call<E: Ext<T = T>>(
		&mut self,
		env: Environment<E, InitState>,
	) -> pallet_contracts::chain_extension::Result<RetVal> {
		let mut env = env.buf_in_buf_out();

		let request: ApiInfo<_> = ApiInfo::<T>::try_from(&mut env)?;

		let result = match request.clone() {
			ApiInfo::Assets(ref api_info) => api_info.call(env.ext()),
			ApiInfo::Listings(ref api_info) => api_info.call(env.ext()),
		};

		log::trace!(
			target: "chainx",
			"call({request:#?}) -> {result:#?}",
		);

		match result {
			Ok(result) => {
				env.write(&result, false, None)?;
				Ok(RetVal::Converging(0))
			}
			Err(error) => {
				let error_code: KreivoApisErrorCode = error.into();
				Ok(RetVal::Converging(error_code.into()))
			}
		}
	}
}
