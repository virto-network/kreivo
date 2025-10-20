// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(not(feature = "zombienet"))]
// use frame_system::EnsureRootWithSuccess;
// #[cfg(feature = "zombienet")]
// use frame_system::EnsureSigned;

extern crate alloc;

use alloc::vec::Vec;
use pallet_balances::*;
use core::{fmt, marker::PhantomData, num::NonZero};
use pallet_revive::{
	precompiles::{
		alloy::{self},
		AddressMatcher, Error, Ext, Precompile,
	},
	AddressMapper, Origin,
};
use sp_runtime::traits::StaticLookup;
use tracing::error;
use pallet_balances::pallet::Config;

alloy::sol!("src/precompiles/IFoo.sol");
use IFoo::IFooCalls;

const LOG_TARGET: &str = "foo::precompiles";

fn revert(error: &impl fmt::Debug, message: &str) -> Error {
	error!(target: LOG_TARGET, ?error, "{}", message);
	Error::Revert(message.into())
}

#[cfg(test)]
mod tests;

pub struct FooPrecompile<T>(PhantomData<T>);

impl<Runtime> Precompile for FooPrecompile<Runtime>
where
	Runtime: Config + pallet_revive::Config,
{
	type T = Runtime;
	const MATCHER: AddressMatcher = AddressMatcher::Fixed(NonZero::new(15).unwrap());
	const HAS_CONTRACT_INFO: bool = false;
	type Interface = IFoo::IFooCalls;

	fn call(
		_address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		let origin = env.caller();
		let frame_origin = match origin {
			Origin::Root => frame_system::RawOrigin::Root.into(),
			Origin::Signed(account_id) =>
				frame_system::RawOrigin::Signed(account_id.clone()).into(),
		};

		match input {
			IFooCalls::transfer(IFoo::transferCall { to, value }) => {
				env.charge(<Runtime as Config>::WeightInfo::transfer_keep_alive())?;

				let dest = <Runtime as pallet_revive::Config>::AddressMapper::to_account_id(&to.into_array().into());

				let value = <Runtime as pallet_balances::Config>::Balance::try_from(value.as_limbs()[0])
					.map_err(|_| revert(&"Value conversion failed", "Value too large"))?;

				pallet_balances::Pallet::<Runtime>::transfer_keep_alive(
					frame_origin,
					<Runtime as frame_system::Config>::Lookup::unlookup(dest),
					value
				)
				.map_err(|error| {
					revert(
						&error,
						"Failed transfering dat assets",
					)
				})
				.map(|_| Vec::new())
			},
			IFooCalls::dontPanic(IFoo::dontPanicCall { }) => {
				Ok("42".into())
			}

		}
	}
}