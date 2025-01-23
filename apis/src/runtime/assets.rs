use super::*;

use core::marker::PhantomData;

use apis::AssetsAPI;
use frame_support::sp_runtime::DispatchError;
use frame_support::traits::tokens::Preservation;
use pallet_contracts::chain_extension::Ext;

/// A helper structure that implements [`KreivoAPI`] in the context of the
/// Runtime.
pub struct RuntimeKreivoAssetsAPI<'a, T, E, Assets>(PhantomData<(T, Assets)>, &'a E);

impl<'a, T, E, Assets> RuntimeKreivoAssetsAPI<'a, T, E, Assets> {
	pub fn new(ext: &'a E) -> Self {
		Self(PhantomData, ext)
	}

	fn ext(&self) -> &E {
		&self.1
	}
}

impl<'a, T, E, A> From<&'a E> for RuntimeKreivoAssetsAPI<'a, T, E, A>
where
	E: Ext<T = T>,
{
	fn from(ext: &'a E) -> Self {
		Self::new(ext)
	}
}

impl<T, E, Assets> AssetsAPI<E> for RuntimeKreivoAssetsAPI<'_, T, E, Assets>
where
	T: pallet_contracts::Config,
	E: Ext<T = T>,
	Assets: frame_support::traits::fungibles::Inspect<T::AccountId>
		+ frame_support::traits::fungibles::Mutate<T::AccountId>,
{
	type AccountId = T::AccountId;
	type AssetId = Assets::AssetId;
	type Balance = Assets::Balance;

	fn deposit(&self, asset: Self::AssetId, amount: Self::Balance) -> Result<Self::Balance, DispatchError> {
		let caller = self.ext().caller().account_id()?.clone();
		Assets::transfer(asset, &caller, self.ext().address(), amount, Preservation::Preserve)
	}

	fn transfer(
		&self,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, DispatchError> {
		Assets::transfer(asset, self.ext().address(), beneficiary, amount, Preservation::Preserve)
	}
}