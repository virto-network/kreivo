use super::*;

mod assets;
mod types;

use assets::*;
use types::*;

use apis::{AssetsAPI, KreivoAPI, KreivoApisErrorCode};
use core::marker::PhantomData;
use frame_support::pallet_prelude::Encode;
use frame_support::DefaultNoBound;
use pallet_contracts::chain_extension::{ChainExtension, Environment, Ext, InitState, RetVal};

/// A helper structure that implements [`KreivoAPI`] in the context of the
/// Runtime.
struct RuntimeKreivoAPI<T, E, Assets> {
	__phantom: PhantomData<(T, E)>,
	assets: Assets,
}

impl<'a, T, E: 'a, Assets: AssetsAPI<E> + From<&'a E>> RuntimeKreivoAPI<T, E, Assets> {
	pub fn new(ext: &'a E) -> Self {
		Self {
			__phantom: PhantomData,
			assets: ext.into(),
		}
	}
}

impl<'a, T, E, Assets> KreivoAPI<E> for RuntimeKreivoAPI<T, E, RuntimeKreivoAssetsAPI<'a, T, E, Assets>>
where
	T: pallet_contracts::Config,
	E: Ext<T = T>,
	Assets: frame_support::traits::fungibles::Mutate<T::AccountId>,
{
	type Assets = RuntimeKreivoAssetsAPI<'a, T, E, Assets>;

	fn assets(&self) -> &Self::Assets {
		&self.assets
	}
}

/// A [`ChainExtension`] that implements the [`KreivoAPI`]s.
#[derive(DefaultNoBound)]
pub struct KreivoChainExtensions<T, Assets>(PhantomData<(T, Assets)>);

impl<T, A> ChainExtension<T> for KreivoChainExtensions<T, A>
where
	T: pallet_contracts::Config,
	A: frame_support::traits::fungibles::Mutate<T::AccountId>,
{
	fn call<E: Ext<T = T>>(
		&mut self,
		env: Environment<E, InitState>,
	) -> pallet_contracts::chain_extension::Result<RetVal> {
		let mut env = env.buf_in_buf_out();

		let result = match ApiInfo::<T, A>::try_from(&mut env)? {
			ApiInfo::Assets(api_info) => match api_info {
				AssetsApiInfo::Balance { asset, who } => {
					let api = RuntimeKreivoAPI::<T, E, RuntimeKreivoAssetsAPI<T, E, A>>::new(env.ext());
					Ok(api.assets().balance(asset, &who).encode())
				}
				AssetsApiInfo::Deposit { asset, amount } => {
					let api = RuntimeKreivoAPI::<T, E, RuntimeKreivoAssetsAPI<T, E, A>>::new(env.ext());
					api.assets().deposit(asset, amount).map(|v| v.encode())
				}
				AssetsApiInfo::Transfer {
					asset,
					amount,
					beneficiary,
				} => {
					let api = RuntimeKreivoAPI::<T, E, RuntimeKreivoAssetsAPI<T, E, A>>::new(env.ext());
					api.assets().transfer(asset, amount, &beneficiary).map(|v| v.encode())
				}
			},
		};

		match result {
			Ok(result) => {
				env.write(&result, false, None)?;
				Ok(RetVal::Converging(0))
			}
			Err(error) => {
				let error_code: KreivoApisErrorCode = error.into();
				Ok(RetVal::Converging(error_code.into()))
			},
		}
	}
}
