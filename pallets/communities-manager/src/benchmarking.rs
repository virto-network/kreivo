//! Benchmarking setup for pallet-communities
use super::*;

use frame_benchmarking::v2::*;

use frame_support::{dispatch::DispatchResult, traits::fungible::Mutate};
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;

type RuntimeEventFor<T> = <T as Config>::RuntimeEvent;

// Since `periodicity` is arbitrary, we assume `DAYS` is a nominal day for 6s
// block.
const DAYS: u32 = 14_400;

fn block_weight<T: frame_system::Config>() -> Weight {
	<T as frame_system::Config>::BlockWeights::get().max_block / 2
}

fn assert_has_event<T: Config>(generic_event: RuntimeEventFor<T>) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn setup_account<T: Config>(who: &AccountIdOf<T>) -> DispatchResult
where
	NativeBalanceOf<T>: From<u64>,
{
	let initial_balance: NativeBalanceOf<T> = 1_000_000_000_000_000u64.into();
	T::Balances::mint_into(who, initial_balance)?;
	Ok(())
}

#[benchmarks(
where
	RuntimeEventFor<T>: From<pallet_communities::Event<T>>,
	NativeBalanceOf<T>: From<u64>,
	CommunityIdOf<T>: One,
	<T as Config>::MembershipId: From<u32>,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register() -> Result<(), BenchmarkError> {
		// setup code
		let first_member: AccountIdOf<T> = account("founder", 0, 0);
		setup_account::<T>(&first_member)?;

		let community_id: CommunityIdOf<T> = One::one();

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			community_id,
			BoundedVec::truncate_from(b"Test Community".into()),
			T::Lookup::unlookup(first_member.clone()),
			None,
			None,
		);

		// verification code
		assert_has_event::<T>(Event::<T>::CommunityRegistered { id: community_id }.into());
		Ok(())
	}

	#[benchmark]
	fn create_memberships(q: Linear<1, 1024>) -> Result<(), BenchmarkError> {
		setup_account::<T>(&T::MembershipsManagerOwner::get())?;

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			q.saturated_into(),
			100u32.into(),
			300_000_000_000u64.into(),
			TankConfig {
				capacity: Some(block_weight::<T>()),
				periodicity: Some((7 * DAYS).into()),
			},
			Some(u32::MAX.into()),
		);

		// verification code
		assert_has_event::<T>(
			Event::<T>::MembershipsCreated {
				starting_at: 100u32.into(),
				amount: q.saturated_into(),
			}
			.into(),
		);
		Ok(())
	}

	#[benchmark]
	fn set_gas_tank() -> Result<(), BenchmarkError> {
		setup_account::<T>(&T::MembershipsManagerOwner::get())?;

		// Setup code
		Pallet::<T>::create_memberships(
			RawOrigin::Root.into(),
			1,
			1u32.into(),
			0u64.into(),
			TankConfig::default(),
			None,
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			T::MembershipsManagerCollectionId::get(),
			1u32.into(),
			TankConfig {
				capacity: Some(block_weight::<T>()),
				periodicity: Some((7 * DAYS).into()),
			},
		);

		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, sp_io::TestExternalities::new(Default::default()), mock::Test);
}
