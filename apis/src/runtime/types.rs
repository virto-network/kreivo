use frame_support::sp_runtime::DispatchError;
use frame_support::traits::fungibles;
use pallet_contracts::chain_extension::{BufInBufOutState, Environment, Ext};

pub enum ApiInfo<T, Assets>
where
	T: frame_system::Config,
	Assets: fungibles::Inspect<T::AccountId>,
{
	Assets(AssetsApiInfo<T, Assets>),
}

pub enum AssetsApiInfo<T: frame_system::Config, F: fungibles::Inspect<T::AccountId>> {
	Balance {
		asset: F::AssetId,
		who: T::AccountId,
	},
	Deposit {
		asset: F::AssetId,
		amount: F::Balance,
	},
	Transfer {
		asset: F::AssetId,
		amount: F::Balance,
		beneficiary: T::AccountId,
	},
}

impl<T, Assets, E> TryFrom<&mut Environment<'_, '_, E, BufInBufOutState>> for ApiInfo<T, Assets>
where
	T: frame_system::Config,
	Assets: fungibles::Inspect<T::AccountId>,
	E: Ext<T = T>,
{
	type Error = DispatchError;

	fn try_from(env: &mut Environment<'_, '_, E, BufInBufOutState>) -> Result<Self, Self::Error> {
		match env.func_id() {
			0x0000 => {
				let (asset, who) = env.read_as()?;
				Ok(Self::Assets(AssetsApiInfo::Balance { asset, who }))
			}
			0x0001 => {
				let (asset, amount) = env.read_as()?;
				Ok(Self::Assets(AssetsApiInfo::Deposit { asset, amount }))
			}
			0x0002 => {
				let (asset, amount, beneficiary) = env.read_as()?;
				Ok(Self::Assets(AssetsApiInfo::Transfer {
					asset,
					amount,
					beneficiary,
				}))
			}
			id => {
				log::error!("Called an unregistered `func_id`: {id:}");
				Err(DispatchError::Other("Unimplemented func_id"))
			}
		}
	}
}
