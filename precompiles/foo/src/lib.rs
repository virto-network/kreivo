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

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(not(feature = "zombienet"))]
// use frame_system::EnsureRootWithSuccess;
// #[cfg(feature = "zombienet")]
// use frame_system::EnsureSigned;

extern crate alloc;

use alloc::vec::Vec;
use core::{fmt, marker::PhantomData, num::NonZero};
use pallet_balances::pallet::{Call, Config};
// use pallet_balances::*;
use pallet_revive::precompiles::{
	alloy::{
		self,
		primitives::IntoLogData,
		sol_types::{Revert, SolCall},
	},
	AddressMapper, AddressMatcher, Error, Ext, Precompile, RuntimeCosts, H160, H256,
};
use pallet_revive::Origin;

use tracing::{error, info};

alloy::sol!("src/precompiles/IFoo.sol");
use IFoo::IFooCalls;
// use IFoo::{IFooCalls, IFooEvents};

const LOG_TARGET: &str = "foo::precompiles";

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait FooPrecompileConfig {
	/// The Address matcher used by the precompile.
	const MATCHER: AddressMatcher;
}

/// A precompile configuration that uses a prefix [`AddressMatcher`].
pub struct InlineIdConfig<const PREFIX: u16>;

impl<const P: u16> FooPrecompileConfig for InlineIdConfig<P> {
	const MATCHER: AddressMatcher = AddressMatcher::Fixed(core::num::NonZero::new(P).unwrap());
}

/// A Foo precompile.
pub struct Foo<Runtime, PrecompileConfig, Instance = ()> {
	_phantom: PhantomData<(Runtime, PrecompileConfig, Instance)>,
}

impl<Runtime, PrecompileConfig, Instance: 'static> Precompile for Foo<Runtime, PrecompileConfig, Instance>
where
	PrecompileConfig: FooPrecompileConfig,
	Runtime: crate::Config<Instance> + pallet_revive::Config,
	// alloy::primitives::U128: TryInto<<Runtime as Config<Instance>>::Balance>,
	// alloy::primitives::U128: TryFrom<<Runtime as Config<Instance>>::Balance>,
{
	type T = Runtime;
	type Interface = IFoo::IFooCalls;
	const MATCHER: AddressMatcher = PrecompileConfig::MATCHER;
	const HAS_CONTRACT_INFO: bool = false;

	fn call(address: &[u8; 20], input: &Self::Interface, env: &mut impl Ext<T = Self::T>) -> Result<Vec<u8>, Error> {
		info!("Fortytwo: ☃️☃️☃️☃️☃️");
		match input {
			// IFooCalls::transfer(call) => Self::transfer(call, env),
			IFooCalls::fortytwo(_) => Self::fortytwo()
		}
	}
}

impl<Runtime, PrecompileConfig, Instance: 'static> Foo<Runtime, PrecompileConfig, Instance>
where
	PrecompileConfig: FooPrecompileConfig,
	Runtime: crate::Config<Instance> + pallet_revive::Config,
	// Call<Runtime, Instance>: Into<<Runtime as pallet_revive::Config>::RuntimeCall>,
	// alloy::primitives::U256: TryInto<<Runtime as Config<Instance>>::Balance>,
	// alloy::primitives::U256: TryFrom<<Runtime as Config<Instance>>::Balance>,
{
	// /// Deposit an event to the runtime.
	// fn deposit_event(env: &mut impl Ext<T = Runtime>, event: IFooEvents) -> Result<(), Error> {
	// 	let (topics, data) = event.into_log_data().split();
	// 	let topics = topics.into_iter().map(|v| H256(v.0)).collect::<Vec<_>>();
	// 	env.gas_meter_mut().charge(RuntimeCosts::DepositEvent {
	// 		num_topic: topics.len() as u32,
	// 		len: topics.len() as u32,
	// 	})?;
	// 	env.deposit_event(topics, data.to_vec());
	// 	Ok(())
	// }

	// /// Execute the transfer call.
	// fn transfer(call: &IFoo::transferCall, env: &mut impl Ext<T = Runtime>) -> Result<Vec<u8>, Error> {
	// 	env.charge(<Runtime as Config<Instance>>::WeightInfo::transfer_keep_alive())?;
	// 	let origin = env.caller();
	// 	let frame_origin = match origin {
	// 		Origin::Root => frame_system::RawOrigin::Root.into(),
	// 		Origin::Signed(account_id) =>
	// 			frame_system::RawOrigin::Signed(account_id.clone()).into(),
	// 	};

	// 	let from = Self::caller(env)?;

	// 	let dest = <Runtime as pallet_revive::Config>::AddressMapper::to_account_id(&call.to.into_array().into());
	// 	let value = &call.value;

	// 	pallet_balances::Pallet::<Runtime, Instance>::transfer_keep_alive(
	// 		frame_origin,
	// 		dest,
	// 		&value,
	// 	)?;

	// 	Self::deposit_event(
	// 		env,
	// 		IFooEvents::Transfer(IFoo::Transfer {
	// 			from: from.0.into(),
	// 			to: call.to,
	// 			value: IFoo::transferCall::abi_decode(call.value),
	// 		}),
	// 	)?;

	// 	return Ok(IFoo::transferCall::abi_encode_returns(&true));
	// }

	fn fortytwo() -> Result<Vec<u8>, Error> {
		info!("Fortytwo: 👾👾👾👾👾👾👾");
		return Ok(IFoo::fortytwoCall::abi_encode_returns(&42u128));
	}
}

// pub struct FooPrecompile<T>(PhantomData<T>);

// impl<Runtime> Precompile for FooPrecompile<Runtime>
// where
// 	Runtime: Config + pallet_revive::Config,
// {
// 	type T = Runtime;
// 	const MATCHER: AddressMatcher = AddressMatcher::Fixed(NonZero::new(15).unwrap());
// 	const HAS_CONTRACT_INFO: bool = false;
// 	type Interface = IFoo::IFooCalls;

// 	fn call(
// 		_address: &[u8; 20],
// 		input: &Self::Interface,
// 		env: &mut impl Ext<T = Self::T>,
// 	) -> Result<Vec<u8>, Error> {
// 		let origin = env.caller();
// 		let frame_origin = match origin {
// 			Origin::Root => frame_system::RawOrigin::Root.into(),
// 			Origin::Signed(account_id) =>
// 				frame_system::RawOrigin::Signed(account_id.clone()).into(),
// 		};

// 		match input {
// 			IFooCalls::transfer(IFoo::transferCall { to, value }) => {
// 				env.charge(<Runtime as Config>::WeightInfo::transfer_keep_alive())?;

// 				let dest = <Runtime as pallet_revive::Config>::AddressMapper::to_account_id(&to.into_array().into());

// 				let value = <Runtime as pallet_balances::Config>::Balance::try_from(value.as_limbs()[0])
// 					.map_err(|_| revert(&"Value conversion failed", "Value too large"))?;

// 				pallet_balances::Pallet::<Runtime>::transfer_keep_alive(
// 					frame_origin,
// 					<Runtime as frame_system::Config>::Lookup::unlookup(dest),
// 					value
// 				)
// 				.map_err(|error| {
// 					revert(
// 						&error,
// 						"Failed transfering dat assets",
// 					)
// 				})
// 				.map(|_| Vec::new())
// 			},
// 			IFooCalls::fortytwo(IFoo::fortytwoCall { }) => {
// 				let fortytwo = 42u128;
// 				warn!("Called fortytwo, the answer is: {:?}", fortytwo);
// 				return Ok(IFoo::fortytwoCall::abi_encode_returns(&fortytwo));
// 			}

// 		}
// 	}
// }
