use super::*;

use crate::apis::KreivoApisErrorCode;
use config::{AccountIdOf, AssetIdOf, BalanceOf};
use ink::chain_extension;

type Environment = KreivoApiEnvironment;

#[chain_extension(extension = 0)]
pub trait ChainExtension {
	type ErrorCode = KreivoApisErrorCode;

	// Assets
	#[allow(non_snake_case)]
	#[ink(function = 0x0000, handle_status = false)]
	fn assets__balance(asset: AssetIdOf<Environment>, who: AccountIdOf<Environment>) -> BalanceOf<Environment>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0001)]
	fn assets__deposit(
		asset: AssetIdOf<Environment>,
		amount: BalanceOf<Environment>,
	) -> Result<BalanceOf<Environment>, KreivoApisErrorCode>;

	#[allow(non_snake_case)]
	#[ink(function = 0x0002)]
	fn assets__transfer(
		asset: AssetIdOf<Environment>,
		amount: BalanceOf<Environment>,
		beneficiary: BalanceOf<Environment>,
	) -> Result<BalanceOf<Environment>, KreivoApisErrorCode>;
}

impl ink::env::chain_extension::FromStatusCode for KreivoApisErrorCode {
	fn from_status_code(code: u32) -> Result<(), Self> {
		if code == 0 {
			Ok(())
		} else {
			Err(code.into())
		}
	}
}
