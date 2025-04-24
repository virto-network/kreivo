use super::*;

use config::Config;

use frame_support::traits::fungibles::{Inspect, Mutate};
use frame_support::traits::tokens::Preservation;

/// A helper structure that implements [`AssetsAPI`] in the context of the
/// Runtime.
pub struct RuntimeAssetsAPI<T>(PhantomData<T>);

impl<T, E> AssetsAPI<E> for RuntimeAssetsAPI<T>
where
	T: Config,
	E: Ext<T = T>,
{
	type AccountId = T::AccountId;
	type AssetId = <T::Assets as Inspect<T::AccountId>>::AssetId;
	type Balance = <T::Assets as Inspect<T::AccountId>>::Balance;

	fn balance(_: &E, asset: Self::AssetId, who: &Self::AccountId) -> Self::Balance {
		T::Assets::balance(asset, who)
	}

	fn deposit(e: &E, asset: Self::AssetId, amount: Self::Balance) -> Result<Self::Balance, KreivoApisError> {
		let caller = e
			.caller()
			.account_id()
			.map_err(|_| KreivoApisError::ExtQueryError)?
			.clone();
		T::Assets::transfer(asset, &caller, e.address(), amount, Preservation::Preserve)
			.map_err(|_| AssetsApiError::CannotDeposit.into())
	}

	fn transfer(
		e: &E,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError> {
		T::Assets::transfer(asset, e.address(), beneficiary, amount, Preservation::Preserve)
			.map_err(|_| AssetsApiError::CannotTransfer.into())
	}
}
