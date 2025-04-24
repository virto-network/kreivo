//! Benchmarking setup for pallet-communities
use super::*;

use self::{
	types::{
		AccountIdOf, AssetIdOf, CommunityIdOf, DecisionMethodFor, MembershipIdOf, NativeBalanceOf, PalletsOriginOf,
		PollIndexOf, RuntimeCallFor, Vote,
	},
	CommunityDecisionMethod, DecisionMethod, Event, FreezeReason, Pallet as Communities,
};
use frame_benchmarking::v2::*;
use frame_contrib_traits::memberships::{Inspect, Rank};
use frame_support::traits::{
	fungible::{InspectFreeze, Mutate},
	fungibles::Mutate as FunsMutate,
	OriginTrait,
};
use frame_system::{
	pallet_prelude::{BlockNumberFor, OriginFor},
	RawOrigin,
};
use sp_runtime::traits::{Hash, StaticLookup};

type RuntimeEventFor<T> = <T as Config>::RuntimeEvent;

fn assert_has_event<T: Config>(generic_event: RuntimeEventFor<T>) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn setup_account<T: Config>(name: &'static str, index: u32, seed: u32) -> Result<AccountIdOf<T>, BenchmarkError> {
	let who = frame_benchmarking::account(name, index, seed);

	let initial_balance: NativeBalanceOf<T> = 1_000_000_000_000_000u128
		.try_into()
		.map_err(|_| BenchmarkError::Stop("could not mint balance for a new account"))?;

	T::Balances::mint_into(&who, initial_balance)?;

	Ok(who)
}

fn setup_accounts<T: Config>() -> Result<Vec<AccountIdOf<T>>, BenchmarkError> {
	let size = T::BenchmarkHelper::community_desired_size();
	let mut accounts = vec![];

	for i in 0..size {
		let who = setup_account::<T>("community_benchmarking", i, 0)?;
		accounts.push(who);
	}

	Ok(accounts)
}

fn community_params<T: Config>(
	maybe_decision_method: Option<DecisionMethodFor<T>>,
) -> (CommunityIdOf<T>, DecisionMethodFor<T>, OriginFor<T>, PalletsOriginOf<T>)
where
	OriginFor<T>: From<Origin<T>>,
{
	let community_id = T::BenchmarkHelper::community_id();

	let decision_method = maybe_decision_method.unwrap_or(DecisionMethod::Rank);
	let admin_origin: OriginFor<T> = Origin::<T>::new(community_id).into();
	let admin_origin_caller: PalletsOriginOf<T> = admin_origin.clone().into_caller();

	(community_id, decision_method, admin_origin, admin_origin_caller)
}

/// Creates a community, setting a [DecisionMethod], returning
/// its ID as well as the caller origin, and origin caller.
fn create_community<T: Config>(
	origin: OriginFor<T>,
	maybe_decision_method: Option<DecisionMethodFor<T>>,
) -> Result<(CommunityIdOf<T>, OriginFor<T>), BenchmarkError>
where
	OriginFor<T>: From<Origin<T>>,
{
	T::BenchmarkHelper::initialize_memberships_collection()?;
	let (community_id, decision_method, admin_origin, admin_origin_caller) =
		community_params::<T>(maybe_decision_method);

	Pallet::<T>::create(origin.clone(), admin_origin_caller, community_id)?;
	Pallet::<T>::set_decision_method(admin_origin.clone(), community_id, decision_method)?;

	Ok((community_id, admin_origin))
}

type Member<T> = (AccountIdOf<T>, MembershipIdOf<T>);

/// Initializes the memberships of a community built for benchmarking
/// purposes.
///
/// Then, returns a list of tuples, each one containing a member's
/// [AccountId] and their corresponding
fn setup_members<T>(origin: OriginFor<T>, community_id: CommunityIdOf<T>) -> Result<Vec<Member<T>>, BenchmarkError>
where
	T: Config,
	T::MembershipId: From<u32>,
{
	let members_with_memberships = setup_accounts::<T>()?
		.into_iter()
		.enumerate()
		.map(|(i, account_id)| (account_id, MembershipIdOf::<T>::from(i as u32)));

	for (who, membership_id) in members_with_memberships.clone() {
		T::BenchmarkHelper::issue_membership(community_id, membership_id)?;

		let who = T::Lookup::unlookup(who.clone());
		Pallet::<T>::add_member(origin.clone(), who.clone())?;
		Pallet::<T>::promote(origin.clone(), membership_id)?;
	}

	Ok(members_with_memberships.collect())
}

fn prepare_track_and_prepare_poll<T: Config>(
	track_origin: PalletsOriginOf<T>,
	submitter: AccountIdOf<T>,
) -> Result<PollIndexOf<T>, BenchmarkError>
where
	RuntimeCallFor<T>: From<crate::Call<T>>,
{
	T::BenchmarkHelper::prepare_track(track_origin.clone())?;

	let new_member = T::Lookup::unlookup(frame_benchmarking::account("community_benchmarking", 0, 0));
	T::BenchmarkHelper::prepare_poll(
		RawOrigin::Signed(submitter).into(),
		track_origin.clone(),
		crate::Call::<T>::add_member { who: new_member }.into(),
	)
}

#[benchmarks(
	where
		OriginFor<T>: From<Origin<T>> + From<frame_system::Origin<T>>,
		RuntimeEventFor<T>: From<frame_system::Event<T>>,
		AssetBalanceOf<T>: From<u64>,
		AssetIdOf<T>: From<u32>,
		MembershipIdOf<T>: From<u32>,
		BlockNumberFor<T>: From<u32>
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create() {
		// setup code
		let (id, _, _, origin) = community_params::<T>(None);

		#[extrinsic_call]
		_(RawOrigin::Root, origin.clone(), id);

		// verification code
		assert_has_event::<T>(Event::CommunityCreated { id, origin }.into());
	}

	#[benchmark]
	fn set_admin_origin() -> Result<(), BenchmarkError> {
		// setup code
		let (id, _, _, community_origin) = community_params::<T>(None);

		let community_account = Communities::<T>::community_account(&id);
		let signed_origin: <T as Config>::RuntimeOrigin = RawOrigin::Signed(community_account.clone()).into();
		let signed_origin_caller: PalletsOriginOf<T> = signed_origin.into_caller();

		Communities::<T>::create(RawOrigin::Root.into(), signed_origin_caller, id)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(community_account), community_origin.clone());

		// verification code
		assert_eq!(CommunityIdFor::<T>::get(community_origin.clone()), Some(id));
		assert_has_event::<T>(
			Event::AdminOriginSet {
				id,
				origin: community_origin,
			}
			.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn set_decision_method() -> Result<(), BenchmarkError> {
		// setup code
		let (id, decision_method, _, admin_origin) = community_params::<T>(None);
		Communities::<T>::create(RawOrigin::Root.into(), admin_origin.clone(), id)?;
		CommunityDecisionMethod::<T>::set(id, decision_method);

		#[extrinsic_call]
		_(
			admin_origin,
			id,
			DecisionMethod::CommunityAsset(T::BenchmarkHelper::community_asset_id(), 10u64.into()),
		);

		// verification code
		assert_has_event::<T>(Event::DecisionMethodSet { id }.into());

		Ok(())
	}

	#[benchmark]
	fn add_member() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin) = create_community::<T>(RawOrigin::Root.into(), None)?;

		let who: AccountIdOf<T> = frame_benchmarking::account("community_benchmarking", 0, 0);
		let membership_id = MembershipIdOf::<T>::from(0);

		T::BenchmarkHelper::issue_membership(id, membership_id)?;

		#[extrinsic_call]
		_(origin.into_caller(), T::Lookup::unlookup(who.clone()));

		// verification code
		assert_has_event::<T>(
			Event::MemberAdded {
				who: who.clone(),
				membership_id,
			}
			.into(),
		);
		assert!(T::MemberMgmt::check_membership(&who, &membership_id).is_some());

		Ok(())
	}

	#[benchmark]
	fn remove_member() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin): (CommunityIdOf<T>, OriginFor<T>) = create_community::<T>(RawOrigin::Root.into(), None)?;

		let who: AccountIdOf<T> = frame_benchmarking::account("community_benchmarking", 0, 0);
		let membership_id = MembershipIdOf::<T>::from(0);

		T::BenchmarkHelper::issue_membership(id, membership_id)?;

		Communities::<T>::add_member(origin.clone(), T::Lookup::unlookup(who.clone()))?;

		#[extrinsic_call]
		_(origin.into_caller(), T::Lookup::unlookup(who.clone()), membership_id);

		// verification code
		assert_has_event::<T>(
			Event::MemberRemoved {
				who: who.clone(),
				membership_id,
			}
			.into(),
		);
		assert!(T::MemberMgmt::check_membership(&who, &membership_id).is_none());

		Ok(())
	}

	#[benchmark]
	fn promote() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin): (CommunityIdOf<T>, OriginFor<T>) = create_community::<T>(RawOrigin::Root.into(), None)?;

		let who: AccountIdOf<T> = frame_benchmarking::account("community_benchmarking", 0, 0);
		let membership_id = MembershipIdOf::<T>::from(0);

		T::BenchmarkHelper::issue_membership(id, membership_id)?;

		Communities::<T>::add_member(origin.clone(), T::Lookup::unlookup(who.clone()))?;

		#[extrinsic_call]
		_(origin.into_caller(), membership_id);

		// verification code
		let (_, m) = T::MemberMgmt::user_memberships(&who, Some(id))
			.next()
			.ok_or::<frame_support::pallet_prelude::DispatchError>(Error::<T>::NotAMember.into())?;
		let rank = T::MemberMgmt::rank_of(&id, &m).expect("has rank");

		assert_has_event::<T>(Event::MembershipRankUpdated { membership_id, rank }.into());

		assert_eq!(Communities::<T>::member_rank(&id, &membership_id), rank);

		Ok(())
	}

	#[benchmark]
	fn demote() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin): (CommunityIdOf<T>, OriginFor<T>) = create_community::<T>(RawOrigin::Root.into(), None)?;

		let who: AccountIdOf<T> = frame_benchmarking::account("community_benchmarking", 0, 0);
		let membership_id = MembershipIdOf::<T>::from(0);

		T::BenchmarkHelper::issue_membership(id, membership_id)?;

		Communities::<T>::add_member(origin.clone(), T::Lookup::unlookup(who.clone()))?;

		Communities::<T>::promote(origin.clone(), membership_id)?;

		#[extrinsic_call]
		_(origin.into_caller(), membership_id);

		// verification code
		assert_eq!(Communities::<T>::member_rank(&id, &membership_id), 0.into());

		Ok(())
	}

	#[benchmark]
	fn vote() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin) = create_community::<T>(
			RawOrigin::Root.into(),
			Some(DecisionMethodFor::<T>::CommunityAsset(1u32.into(), 1u64.into())),
		)?;
		let members = setup_members::<T>(origin.clone(), id)?;

		let (who, membership_id) = members
			.first()
			.expect("desired size of community to be equal or greather than 1")
			.clone();

		T::Assets::mint_into(1u32.into(), &who, 4u64.into())?;

		prepare_track_and_prepare_poll::<T>(origin.into_caller(), who.clone())?;

		Communities::<T>::vote(
			RawOrigin::Signed(who.clone()).into(),
			membership_id,
			0u32,
			Vote::AssetBalance(true, 1u32.into(), 1u64.into()),
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Signed(who.clone()),
			membership_id,
			0u32,
			Vote::AssetBalance(true, 1u32.into(), 2u64.into()),
		);

		// verification code
		assert_has_event::<T>(
			Event::VoteCasted {
				who: who.clone(),
				poll_index: 0u32,
				vote: Vote::AssetBalance(true, 1u32.into(), 2u32.into()),
			}
			.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn remove_vote() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin) = create_community::<T>(RawOrigin::Root.into(), None)?;
		let members = setup_members::<T>(origin.clone(), id)?;

		let (who, membership_id) = members
			.first()
			.expect("desired size of community to be equal or greather than 1")
			.clone();

		prepare_track_and_prepare_poll::<T>(origin.into_caller(), who.clone())?;

		Communities::<T>::vote(
			RawOrigin::Signed(who.clone()).into(),
			membership_id,
			0u32,
			Vote::Standard(true),
		)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(who.clone()), membership_id, 0u32);

		// verification code
		assert_has_event::<T>(
			Event::VoteRemoved {
				who: who.clone(),
				poll_index: 0u32,
			}
			.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn unlock() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin) = create_community::<T>(RawOrigin::Root.into(), Some(DecisionMethod::NativeToken))?;
		let members = setup_members::<T>(origin.clone(), id)?;

		let (who, membership_id) = members
			.first()
			.expect("desired size of community to be equal or greather than 1")
			.clone();

		let index = prepare_track_and_prepare_poll::<T>(origin.into_caller(), who.clone())?;

		Communities::<T>::vote(
			RawOrigin::Signed(who.clone()).into(),
			membership_id,
			0u32,
			Vote::NativeBalance(true, 1u32.into()),
		)?;

		assert_eq!(
			T::Balances::balance_frozen(&FreezeReason::VoteCasted.into(), &who),
			1u32.into()
		);

		T::BenchmarkHelper::finish_poll(index)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(who.clone()), 0u32);

		// verification code
		assert_eq!(
			T::Balances::balance_frozen(&FreezeReason::VoteCasted.into(), &who),
			0u32.into()
		);

		Ok(())
	}

	#[benchmark]
	fn dispatch_as_account() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin) = create_community::<T>(RawOrigin::Root.into(), Some(DecisionMethod::NativeToken))?;
		let remark = b"Hello, world".to_vec();

		#[extrinsic_call]
		_(
			origin.into_caller(),
			Box::new(frame_system::Call::<T>::remark_with_event { remark: remark.clone() }.into()),
		);

		// verification code
		let sender = Communities::<T>::community_account(&id);
		let hash = <T as frame_system::Config>::Hashing::hash(&remark);

		assert_has_event::<T>(frame_system::Event::<T>::Remarked { sender, hash }.into());

		Ok(())
	}

	impl_benchmark_test_suite!(
		Communities,
		sp_io::TestExternalities::new(Default::default()),
		crate::mock::Test
	);
}
