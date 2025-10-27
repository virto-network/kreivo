use super::*;

use frame_support::sp_runtime::DispatchError;
use frame_support::traits::fungibles;
use pallet_contracts::chain_extension::{BufInBufOutState, Environment, Ext};

pub type AssetIdOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::AssetId;
pub type AssetBalanceOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::Balance;

#[derive(Encode, Decode, Clone, DebugNoBound)]
pub enum AssetsApiInfo<T: Config> {
	Balance {
		asset: AssetIdOf<T>,
		who: AccountIdOf<T>,
	},
	Deposit {
		asset: AssetIdOf<T>,
		amount: AssetBalanceOf<T>,
	},
	Transfer {
		asset: AssetIdOf<T>,
		amount: AssetBalanceOf<T>,
		beneficiary: T::AccountId,
	},
}

impl<T, E> TryFrom<&mut Environment<'_, '_, E, BufInBufOutState>> for AssetsApiInfo<T>
where
	T: Config,
	E: Ext<T = T>,
{
	type Error = DispatchError;

	fn try_from(env: &mut Environment<'_, '_, E, BufInBufOutState>) -> Result<Self, Self::Error> {
		match env.func_id() {
			0x0000 => {
				let (asset, who) = env.read_as()?;
				Ok(AssetsApiInfo::Balance { asset, who })
			}
			0x0001 => {
				let (asset, amount) = env.read_as()?;
				Ok(AssetsApiInfo::Deposit { asset, amount })
			}
			0x0002 => {
				let (asset, amount, beneficiary) = env.read_as()?;
				Ok(AssetsApiInfo::Transfer {
					asset,
					amount,
					beneficiary,
				})
			}
			id => {
				log::error!("Called an unregistered `func_id`: {id:}");
				Err(DispatchError::Other("Unimplemented func_id"))
			}
		}
	}
}
