use super::*;

use alloc::vec;
use frame_benchmarking::v2::*;
use frame_support::traits::fungible::Mutate;
use parity_scale_codec::HasCompact;
use sp_runtime::traits::{Bounded, EnsureDiv, Hash};

type RuntimeEventFor<T> = <T as Config>::RuntimeEvent;

fn assert_has_event<T: Config>(generic_event: RuntimeEventFor<T>) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn mock_upload_code<T: Config>(app_info: &mut AppInfoFor<T>, _: &AccountIdOf<T>, code: Vec<u8>) -> DispatchResult {
	app_info
		.bump_version(T::Hashing::hash(&code))
		.ok_or(Error::<T>::CannotIncrement)?;
	Ok(())
}

fn prepare_publisher<T: Config>(who: &AccountIdOf<T>) -> DispatchResult
where
	BalanceOf<T>: Bounded,
{
	let amount = BalanceOf::<T>::max_value().ensure_div(2u32.into())?;
	<T as pallet_contracts::Config>::Currency::mint_into(who, amount)?;
	Ok(())
}

fn publish_app<T: Config>(
	origin: OriginFor<T>,
	max_instances: Option<u64>,
	price: Option<ItemPriceOf<T>>,
) -> Result<T::AppId, DispatchError>
where
	BalanceOf<T>: Bounded,
	<BalanceOf<T> as HasCompact>::Type: Parameter,
{
	let id = Pallet::<T>::generate_app_id()?;
	let publisher = T::UploadOrigin::ensure_origin(origin)?;
	prepare_publisher::<T>(&publisher)?;

	Pallet::<T>::do_publish(
		id.clone(),
		&publisher,
		vec![0u8; 32],
		max_instances,
		price.clone(),
		mock_upload_code::<T>,
	)?;
	Ok(id)
}

#[benchmarks(
where
	BalanceOf<T>: Bounded,
	<BalanceOf<T> as HasCompact>::Type: Parameter,
	ListingsAssetOf<T>: Default,
	ListingsBalanceOf<T>: Bounded,
)]
pub mod benchmarks {
	use super::*;
	use frame_contrib_traits::listings::item::ItemPrice;

	#[benchmark]
	fn publish() -> Result<(), BenchmarkError> {
		let origin = T::UploadOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("Couldn't get successful origin"))?;
		let code = vec![0u8; 32];
		let id = NextAppId::<T>::get();
		let publisher =
			T::UploadOrigin::ensure_origin(origin.clone()).map_err(|_| BenchmarkError::Stop("Invalid origin"))?;
		prepare_publisher::<T>(&publisher)?;
		let max_instances = Some(u64::MAX);
		let price = None;

		#[block]
		{
			let id = Pallet::<T>::generate_app_id()?;
			let publisher =
				T::UploadOrigin::ensure_origin(origin).map_err(|_| BenchmarkError::Stop("Invalid origin"))?;

			Pallet::<T>::do_publish(
				id.clone(),
				&publisher,
				code,
				max_instances,
				price.clone(),
				mock_upload_code::<T>,
			)?;
		}

		assert_has_event::<T>(
			Event::AppPublished {
				id,
				publisher,
				max_instances,
				price,
			}
			.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn set_parameters() -> Result<(), BenchmarkError> {
		let origin = T::UploadOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("Couldn't get successful origin"))?;
		let id = publish_app::<T>(origin.clone(), None, None)?;
		let price = ItemPrice {
			asset: ListingsAssetOf::<T>::default(),
			amount: ListingsBalanceOf::<T>::max_value(),
		};

		#[extrinsic_call]
		_(
			origin as T::RuntimeOrigin,
			id.clone(),
			Some(u64::MAX),
			Some(price.clone()),
		);

		assert_has_event::<T>(Event::<T>::AppPriceUpdated { id, price }.into());

		Ok(())
	}

	#[benchmark]
	fn publish_upgrade() -> Result<(), BenchmarkError> {
		let origin = T::UploadOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("Couldn't get successful origin"))?;
		let id = publish_app::<T>(origin.clone(), None, None)?;

		#[block]
		{
			let publisher =
				T::UploadOrigin::ensure_origin(origin.clone()).map_err(|_| BenchmarkError::Stop("Invalid origin"))?;
			Pallet::<T>::do_publish_upgrade(&publisher, id.clone(), vec![1u8; 32], mock_upload_code::<T>)?;
		}

		assert_has_event::<T>(Event::<T>::AppUpdated { id, version: 2 }.into());

		Ok(())
	}

	#[benchmark]
	fn request_license() -> Result<(), BenchmarkError> {
		let origin = T::UploadOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("Couldn't get successful origin"))?;
		let app_id = publish_app::<T>(origin.clone(), None, None)?;
		let license_id = NextLicenseId::<T>::get(app_id.clone());
		let caller = <<T as Config>::InstantiateOrigin>::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("Couldn't get successful origin"))?;

		#[extrinsic_call]
		_(caller, app_id.clone());

		assert_has_event::<T>(Event::<T>::AppLicenseEmitted { app_id, license_id }.into());

		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, sp_io::TestExternalities::default(), mock::Test);
}
