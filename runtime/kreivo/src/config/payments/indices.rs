// Copyright (C) Parity Technologies (UK) Ltd.
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
// along with Polkadot. If not, see <http://www.gnu.org/licenses/>.

//! Temporary storage for payment indices.

#[frame_support::pallet]
pub mod pallet_payment_indices {
	use frame_support::{pallet_prelude::*, traits::Hooks};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(crate) type Index<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_: BlockNumberFor<T>) {
			Index::<T>::kill()
		}
	}

	impl<T: Config> pallet_payments::GeneratePaymentId<T::AccountId> for Pallet<T> {
		type PaymentId = virto_common::PaymentId;

		fn generate(_: &T::AccountId, beneficiary: &T::AccountId) -> Option<virto_common::PaymentId> {
			let block: u32 = frame_system::Pallet::<T>::block_number();
			let idx = Index::<T>::mutate(|index| {
				let ix = *index;
				*index += 1;
				ix
			});
			Some((block, idx, beneficiary.encode().as_slice()).into())
		}
	}
}
