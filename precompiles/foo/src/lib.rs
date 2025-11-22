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

extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use pallet_balances::pallet::Config;
use pallet_revive::precompiles::{
	alloy::{self, sol_types::SolCall},
	AddressMatcher, Error, Ext, Precompile,
};

alloy::sol!("src/precompiles/IFoo.sol");
use IFoo::IFooCalls;

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
{
	type T = Runtime;
	type Interface = IFoo::IFooCalls;
	const MATCHER: AddressMatcher = PrecompileConfig::MATCHER;
	const HAS_CONTRACT_INFO: bool = false;

	fn call(_address: &[u8; 20], input: &Self::Interface, _env: &mut impl Ext<T = Self::T>) -> Result<Vec<u8>, Error> {
		match input {
			IFooCalls::fortytwo(_) => Self::fortytwo(),
		}
	}
}

impl<Runtime, PrecompileConfig, Instance: 'static> Foo<Runtime, PrecompileConfig, Instance>
where
	PrecompileConfig: FooPrecompileConfig,
	Runtime: crate::Config<Instance> + pallet_revive::Config,
{
	fn fortytwo() -> Result<Vec<u8>, Error> {
		return Ok(IFoo::fortytwoCall::abi_encode_returns(&42u128));
	}
}
