#![cfg_attr(not(feature = "std"), no_std)]

//! # Contracts Store Pallet
//!
//! ## Overview
//!
//! This pallet provides a way to publish and instantiate smart contracts.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - [publish][Pallet::publish] publishes new applications, given an authorized
//!   origin.
//! - [set_parameters][Pallet::set_parameters] sets the parameters of an
//!   existing application.
//! - [publish_upgrade][Pallet::publish_upgrade] publishes the code version of
//!   an existing application.
//! - [request_license][Pallet::request_license] requests a license for
//!   instantiating an application.
//! - [instantiate][Pallet::instantiate] instantiates an application.
//! - [upgrade][Pallet::upgrade] upgrades an application.
//! - [transfer_license][Pallet::transfer_license] transfers a license to
//!   another account.
//! - [transfer_ownership][Pallet::transfer_ownership] transfers the ownership
//!   of an application.
//! - [set_price][Pallet::set_price] sets the price for an application.
//! - [set_max_instances][Pallet::set_max_instances] sets the maximum number of
//!   instances for an application.
//! - [set_determinism][Pallet::set_determinism] sets the determinism for an
//!   application.
//!
//! ## Contract Fixtures
//!
//! This crate uses a copy of
//! [pallet-contract-fixtures](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/contract-fixtures)
//! to provide a set of contracts that can be used for testing.

extern crate alloc;
extern crate core;

use alloc::vec::Vec;
use frame_contrib_traits::listings::{item::Item, InspectInventory, InspectItem, InventoryLifecycle, MutateItem};
use frame_support::{pallet_prelude::*, traits::Incrementable};
use frame_system::pallet_prelude::*;
use pallet_contracts::{Code, CodeUploadReturnValue, CollectEvents, DebugInfo, Determinism, InstantiateReturnValue};
use parity_scale_codec::HasCompact;
use sp_runtime::traits::StaticLookup;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub mod weights;

pub use pallet::*;
pub use types::*;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub const CONTRACT_MERCHANT_ID: [u8; 20] = *b"CONTRACT_MERCHANT_ID";

	#[pallet::config]
	pub trait Config: pallet_contracts::Config + frame_system::Config {
		// Primitives: Some overarching types that are aggregated in the system.

		/// Overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The weight information for this pallet.
		type WeightInfo: WeightInfo;

		// Origins: Types that manage authorization rules to allow or deny some caller
		// origins to execute a method.

		/// An origin to allowed to request copies of an application, and
		/// instantiate it once proven they own a copy of the application.
		type InstantiateOrigin: EnsureOrigin<
			Self::RuntimeOrigin,
			Success = (Self::AccountId, ListingsMerchantIdOf<Self>),
		>;

		// Types: A set of parameter types that the pallet uses to handle information.

		/// An unique identification for an application.
		type AppId: Parameter + MaxEncodedLen + Default + Incrementable;
		/// An unique identification for a license of the application.
		type LicenseId: Parameter + MaxEncodedLen + Default + Incrementable;

		// Dependencies: The external components this pallet depends on.
		/// The `Listings` component of a `Marketplace` system.
		type Listings: InventoryLifecycle<Self::AccountId, InventoryId = Self::AppId>
			+ InspectItem<
				Self::AccountId,
				MerchantId = ListingsMerchantIdOf<Self>,
				InventoryId = Self::AppId,
				ItemId = Self::LicenseId,
			> + MutateItem<Self::AccountId>;

		// Parameters: A set of constant parameters to configure limits.

		/// The `MerchantId` associated to the contracts store.
		#[pallet::constant]
		type ContractsStoreMerchantId: Get<ListingsMerchantIdOf<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// Errors inform users that something worked or went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The specified app is not found.
		AppNotFound,
		/// The caller does not have permissions to mutate the specified app.
		NoPermission,
		/// The given change of the price is invalid.
		InvalidPriceChange,
		/// Incrementing a parameter failed.
		CannotIncrement,
		/// The maximum amount of licenses for an application has been already
		/// issued. Please contact the publisher of this application.
		MaxLicensesExceeded,
		/// It is not possible to issue a license, due to a problem issuing the
		/// item that represents the license. Contact the publisher of this
		/// application.
		CannotIssueLicense,
		/// The specified license is not found.
		LicenseNotFound,
		/// The address associated to an app license is not a valid instance.
		AppInstanceNotFound,
		/// The contract was not instantiated due to being reverted.
		ContractReverted,
		/// The application instance is up to date.
		AppInstanceUpToDate,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new app has been published in the store.
		AppPublished {
			id: T::AppId,
			publisher: AccountIdOf<T>,
			max_instances: Option<u64>,
			price: Option<ItemPriceOf<T>>,
		},
		/// A new app has been published in the store.
		AppUpdated { id: T::AppId, version: u32 },
		/// The price of an application has been updated.
		AppPriceUpdated { id: T::AppId, price: ItemPriceOf<T> },
		/// A new license of the app has been emitted and is ready to be
		/// acquired.
		AppLicenseEmitted {
			app_id: T::AppId,
			license_id: LicenseIdFor<T>,
		},
		/// An app has been instanced. This follows the instantiation a new
		/// contract using the application `CodeHash`, using a valid app
		/// license.
		AppInstantiated {
			app_id: T::AppId,
			license_id: T::LicenseId,
			caller: T::AccountId,
		},
	}

	/// The next `AppId` to be used when publishing a new app.
	#[pallet::storage]
	pub type NextAppId<T: Config> = StorageValue<_, T::AppId, ValueQuery>;

	/// The next `LicenseId` to be used when emitting a new license for an app.
	#[pallet::storage]
	pub type NextLicenseId<T: Config> = StorageMap<_, Blake2_128Concat, T::AppId, T::LicenseId, ValueQuery>;

	/// The information of registered apps.
	#[pallet::storage]
	pub type Apps<T: Config> = StorageMap<_, Blake2_128Concat, T::AppId, AppInfoFor<T>>;

	/// The `MerchantId` associated to a contract account.
	#[pallet::storage]
	pub type ContractMerchantId<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ListingsMerchantIdOf<T>>;

	/// The contract account of an app instance.
	#[pallet::storage]
	pub type ContractAccount<T: Config> = StorageMap<_, Blake2_128Concat, (T::AppId, T::LicenseId), AccountIdOf<T>>;

	#[pallet::call(weight(<T as Config>::WeightInfo))]
	impl<T: Config> Pallet<T>
	where
		<BalanceOf<T> as HasCompact>::Type: Parameter,
	{
		/// Publish a new application,
		#[pallet::call_index(0)]
		#[pallet::weight(
			<T as Config>::WeightInfo::publish()
				.saturating_add(<<T as pallet_contracts::Config>::WeightInfo as pallet_contracts::WeightInfo>::upload_code_determinism_enforced(code.len() as u32))
		)]
		pub fn publish(
			origin: OriginFor<T>,
			code: Vec<u8>,
			max_instances: Option<u64>,
			price: Option<ItemPriceOf<T>>,
		) -> DispatchResult {
			let publisher = T::UploadOrigin::ensure_origin(origin)?;
			let id = Self::generate_app_id()?;

			Self::do_publish(id, &publisher, code, max_instances, price, Self::upload_code)
		}

		/// Sets the price for an existing application.
		///
		/// The caller must be a valid [`UploadOrigin`][T::UploadOrigin], and
		/// the account derived from it must be publisher of the application.
		#[pallet::call_index(1)]
		pub fn set_parameters(
			origin: OriginFor<T>,
			app_id: T::AppId,
			max_instances: Option<u64>,
			price: Option<ItemPriceOf<T>>,
		) -> DispatchResult {
			let who = T::UploadOrigin::ensure_origin(origin)?;
			Apps::<T>::try_mutate(app_id.clone(), |maybe_app| {
				let Some(app) = maybe_app else {
					Err(Error::<T>::AppNotFound)?
				};

				ensure!(app.publisher == who, Error::<T>::NoPermission);

				app.max_instances = max_instances;
				// Can't remove the price for an app.
				if let Some(price) = price {
					app.price = Some(price.clone());
					Self::deposit_event(Event::<T>::AppPriceUpdated { id: app_id, price })
				}

				Ok(())
			})
		}

		/// Publishes the code version of an existing application. Then, every
		/// app instance can upgrade to the latest version.
		///
		/// The caller must be a valid [`UploadOrigin`][T::UploadOrigin], and
		/// the account derived from it must be publisher of the application.
		///
		/// This would call a migration to set the new code on for every app
		/// instance.
		#[pallet::call_index(2)]
		#[pallet::weight(
			<T as Config>::WeightInfo::publish_upgrade()
				.saturating_add(<<T as pallet_contracts::Config>::WeightInfo as pallet_contracts::WeightInfo>::upload_code_determinism_enforced(code.len() as u32))
		)]
		pub fn publish_upgrade(origin: OriginFor<T>, app_id: T::AppId, code: Vec<u8>) -> DispatchResult {
			let who = &T::UploadOrigin::ensure_origin(origin)?;
			Self::do_publish_upgrade(who, app_id, code, Self::upload_code)
		}

		/// Request a license for instantiating an application.
		///
		/// The caller must be a valid
		/// [`InstantiateOrigin`][T::InstantiateOrigin].
		///
		/// When successful, a new license would be issued, available for
		/// purchase or transferred to the caller, if the application is free.
		#[pallet::call_index(3)]
		pub fn request_license(origin: OriginFor<T>, app_id: T::AppId) -> DispatchResult {
			let (who, _) = &<<T as Config>::InstantiateOrigin>::ensure_origin(origin)?;
			Apps::<T>::try_mutate(app_id.clone(), |app_info| {
				let Some(app) = app_info else {
					Err(Error::<T>::AppNotFound)?
				};
				let inventory_id = (T::ContractsStoreMerchantId::get(), app_id.clone());
				let license_id = Self::generate_license_id(app_id.clone())?;

				match app.max_instances {
					Some(max_instances) if app.instances == max_instances => Err(Error::<T>::MaxLicensesExceeded),
					_ => {
						app.instances += 1;
						Ok(())
					}
				}?;

				T::Listings::publish(&inventory_id, &license_id, b"".to_vec(), app.price.clone())?;

				if app.price.is_none() {
					T::Listings::transfer(&inventory_id, &license_id, who)?;
				}

				Self::deposit_event(Event::<T>::AppLicenseEmitted { app_id, license_id });

				Ok(())
			})
		}

		#[pallet::call_index(4)]
		#[pallet::weight(
			<<T as pallet_contracts::Config>::WeightInfo as pallet_contracts::WeightInfo>::instantiate(
				data.len() as u32,
				salt.len() as u32,
			)
				// + app info + item info
				.saturating_add(<T as frame_system::Config>::DbWeight::get().reads(3))
		)]
		pub fn instantiate(
			origin: OriginFor<T>,
			app_id: T::AppId,
			license_id: T::LicenseId,
			#[pallet::compact] value: BalanceOf<T>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> DispatchResult {
			let (caller, merchant_id) = <<T as Config>::InstantiateOrigin>::ensure_origin(origin)?;
			let inventory_id = (T::ContractsStoreMerchantId::get(), app_id.clone());

			let AppInfo { code_hash, .. } = Apps::<T>::get(&app_id).ok_or(Error::<T>::AppNotFound)?;
			let Item { owner, .. } =
				T::Listings::item(&inventory_id, &license_id).ok_or(Error::<T>::LicenseNotFound)?;

			ensure!(caller == owner, Error::<T>::NoPermission);

			let InstantiateReturnValue { result, account_id } = Contracts::<T>::bare_instantiate(
				caller.clone(),
				value,
				Weight::MAX, // TODO: Replace with something reasonable.
				None,        // Again, charging the uploader with the deposit for instantiating the contract.
				Code::Existing(code_hash),
				data,
				salt,
				DebugInfo::Skip,
				CollectEvents::Skip,
			)
			.result?;

			if result.did_revert() {
				Err(Error::<T>::ContractReverted)?
			}

			ContractAccount::<T>::insert((app_id.clone(), license_id.clone()), account_id.clone());
			ContractMerchantId::<T>::insert(account_id, merchant_id);

			Self::deposit_event(Event::<T>::AppInstantiated {
				app_id,
				license_id,
				caller,
			});

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(
			<<T as pallet_contracts::Config>::WeightInfo as pallet_contracts::WeightInfo>::set_code()
				// + app info + item info + instance hash
				.saturating_add(<T as frame_system::Config>::DbWeight::get().reads(4))
		)]
		pub fn upgrade(origin: OriginFor<T>, app_id: T::AppId, license_id: T::LicenseId) -> DispatchResult {
			let (who, _) = &<<T as Config>::InstantiateOrigin>::ensure_origin(origin)?;

			let AppInfo { code_hash, .. } = Apps::<T>::get(&app_id).ok_or(Error::<T>::AppNotFound)?;
			let inventory_id = (T::ContractsStoreMerchantId::get(), app_id.clone());
			let Item { ref owner, .. } =
				T::Listings::item(&inventory_id, &license_id).ok_or(Error::<T>::LicenseNotFound)?;

			ensure!(who == owner, Error::<T>::NoPermission);

			let contract_account =
				ContractAccount::<T>::get(&(app_id, license_id)).ok_or(Error::<T>::AppInstanceNotFound)?;

			let instance_hash = Contracts::<T>::code_hash(&contract_account).ok_or(Error::<T>::AppInstanceNotFound)?;

			ensure!(code_hash != instance_hash, Error::<T>::AppInstanceUpToDate);

			Contracts::<T>::set_code(
				frame_system::Origin::<T>::Root.into(),
				T::Lookup::unlookup(contract_account),
				code_hash,
			)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn generate_app_id() -> Result<T::AppId, DispatchError> {
		NextAppId::<T>::try_mutate(|next_id| {
			let id = next_id.clone();
			*next_id = id.increment().ok_or(Error::<T>::CannotIncrement)?;
			Ok(id)
		})
	}

	fn generate_license_id(app_id: T::AppId) -> Result<T::LicenseId, DispatchError> {
		NextLicenseId::<T>::try_mutate(app_id, |next_id| {
			let id = next_id.clone();
			*next_id = id.increment().ok_or(Error::<T>::CannotIncrement)?;
			Ok(id)
		})
	}

	fn do_publish(
		id: T::AppId,
		publisher: &AccountIdOf<T>,
		code: Vec<u8>,
		max_instances: Option<u64>,
		price: Option<ItemPriceOf<T>>,
		upload_code: impl FnOnce(&mut AppInfoFor<T>, &AccountIdOf<T>, Vec<u8>) -> DispatchResult,
	) -> DispatchResult {
		Apps::<T>::try_mutate_exists(id.clone(), |app| -> DispatchResult {
			let mut app_info = AppInfo {
				code_hash: Default::default(),
				publisher: publisher.clone(),
				max_instances,
				instances: 0,
				price: price.clone(),
				version: 0,
			};

			upload_code(&mut app_info, publisher, code)?;
			*app = Some(app_info);

			let inventory_id = (T::ContractsStoreMerchantId::get(), id.clone());
			T::Listings::create(inventory_id, publisher)?;

			Self::deposit_event(Event::AppPublished {
				id,
				publisher: publisher.clone(),
				max_instances,
				price,
			});

			Ok(())
		})
	}

	fn do_publish_upgrade(
		who: &AccountIdOf<T>,
		app_id: T::AppId,
		code: Vec<u8>,
		upload_code: impl FnOnce(&mut AppInfoFor<T>, &AccountIdOf<T>, Vec<u8>) -> DispatchResult,
	) -> DispatchResult {
		Apps::<T>::try_mutate(app_id.clone(), |maybe_app| {
			let Some(app_info) = maybe_app else {
				Err(Error::<T>::AppNotFound)?
			};

			ensure!(&app_info.publisher == who, Error::<T>::NoPermission);
			upload_code(app_info, who, code)?;

			Self::deposit_event(Event::<T>::AppUpdated {
				id: app_id,
				version: app_info.version,
			});
			Ok(())
		})
	}

	/// Uploads the code of an app, and increases the version of such app.
	///
	/// To achieve this as briefly as possible, we take two considerations:
	///
	/// 1. No deposit limit: publishers must be aware of this.
	/// 2. Enforced determinism: every contract must be executable on-chain.
	fn upload_code(app_info: &mut AppInfoFor<T>, publisher: &AccountIdOf<T>, code: Vec<u8>) -> DispatchResult {
		// Uploads the code: if successful, would return a new `CodeHash` for the
		// application.
		let CodeUploadReturnValue { code_hash, .. } =
			Contracts::<T>::bare_upload_code(publisher.clone(), code, None, Determinism::Enforced)?;
		app_info.bump_version(code_hash).ok_or(Error::<T>::CannotIncrement)?;
		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	pub fn maybe_merchant_id(who: &T::AccountId) -> Option<ListingsMerchantIdOf<T>> {
		ContractMerchantId::<T>::get(who)
	}
}
