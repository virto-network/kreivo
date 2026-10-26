use crate::{
	mock::*,
	weights::{SubstrateWeight, WeightInfo},
	CommunityDeposits, Deposits, Error, Event, HoldReason, NextMembershipId, Retiring,
};
use frame_contrib_traits::memberships::{Inspect as _, InspectEnumerable as _};
use frame_support::{
	assert_noop, assert_ok,
	traits::fungible::{hold::Inspect as _, Inspect as _, Mutate as _},
	weights::Weight,
	BoundedVec,
};
use pallet_nfts::AttributeNamespace;

const ALICE: AccountId = AccountId::new([1; 32]);
const BOB: AccountId = AccountId::new([2; 32]);
const CAROL: AccountId = AccountId::new([3; 32]);

fn community_account(id: CommunityId) -> AccountId {
	Communities::community_account(&id)
}

/// Registers a community administered by `admin` (Root registers for free), and funds its account.
fn register_community(id: CommunityId, admin: AccountId, funds: Balance) {
	assert_ok!(CommunitiesManager::register(
		RuntimeOrigin::root(),
		id,
		BoundedVec::try_from(b"A Community".to_vec()).expect("meets max length; qed"),
		admin,
		None,
		None,
	));
	if funds > 0 {
		assert_ok!(Balances::mint_into(&community_account(id), funds));
	}
}

fn issue(admin: AccountId, amount: u32) {
	assert_ok!(CommunitiesManager::issue_memberships(
		RuntimeOrigin::signed(admin),
		amount
	));
}

fn retire(admin: AccountId, ids: Vec<MembershipId>) {
	assert_ok!(CommunitiesManager::retire_memberships(
		RuntimeOrigin::signed(admin),
		BoundedVec::try_from(ids).expect("meets the limit; qed"),
	));
}

fn stock(community: CommunityId) -> Vec<MembershipId> {
	let mut stock: Vec<_> = MembershipsManager::group_available_memberships(&community).collect();
	stock.sort();
	stock
}

fn held_deposits(community: CommunityId) -> Balance {
	Balances::balance_on_hold(&HoldReason::MembershipDeposit.into(), &community_account(community))
}

fn at(block: BlockNumber) {
	System::set_block_number(block);
}

fn register_as(payer: AccountId, id: CommunityId, admin: AccountId) -> frame_support::dispatch::DispatchResult {
	CommunitiesManager::register(
		RuntimeOrigin::signed(payer),
		id,
		BoundedVec::try_from(b"A Community".to_vec()).expect("meets max length; qed"),
		admin,
		None,
		None,
	)
}

fn held_community_deposit(community: CommunityId) -> Balance {
	Balances::balance_on_hold(&HoldReason::CommunityDeposit.into(), &community_account(community))
}

// DEC-38, 0010-Q1, 0010-Q3
#[test]
fn registering_pays_the_consumption_part_to_the_payee_and_holds_the_storage_part() {
	new_test_ext().execute_with(|| {
		assert_ok!(Balances::mint_into(&BOB, 1_000));

		assert_ok!(register_as(BOB, 1, ALICE));

		// The total paid is the registration payment.
		assert_eq!(Balances::balance(&BOB), 900);
		assert_eq!(Balances::balance(&Treasury::get()), 70);
		assert_eq!(Balances::total_balance(&community_account(1)), 30);
		assert_eq!(held_community_deposit(1), 30);
		assert_eq!(CommunityDeposits::<Test>::get(1), Some(30));
	})
}

// m3-root-registration
#[test]
fn a_root_registration_moves_nothing_holds_nothing_and_records_nothing() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 0);

		assert_eq!(Balances::balance(&Treasury::get()), 0);
		assert_eq!(Balances::total_balance(&community_account(1)), 0);
		assert_eq!(held_community_deposit(1), 0);
		assert_eq!(CommunityDeposits::<Test>::get(1), None);
	})
}

#[test]
fn registering_a_community_creates_its_first_track_in_its_own_group() {
	use pallet_referenda_tracks::SplitId;

	new_test_ext().execute_with(|| {
		register_community(7, ALICE, 0);

		let track = <TrackId as SplitId>::combine(7, 0);
		assert!(pallet_referenda_tracks::TracksIds::<Test>::get().contains(&track));
		assert_eq!(track.split(), (7, 0));
	})
}

// 0010-Q3, REQ-MI-12
#[test]
fn a_later_change_of_the_parts_does_not_change_an_existing_hold() {
	new_test_ext().execute_with(|| {
		assert_ok!(Balances::mint_into(&BOB, 1_000));
		assert_ok!(Balances::mint_into(&CAROL, 1_000));
		assert_ok!(register_as(BOB, 1, ALICE));

		StoragePart::set(50);
		ConsumptionPart::set(50);
		assert_ok!(register_as(CAROL, 2, BOB));

		assert_eq!(CommunityDeposits::<Test>::get(1), Some(30));
		assert_eq!(held_community_deposit(1), 30);
		assert_eq!(CommunityDeposits::<Test>::get(2), Some(50));
		assert_eq!(held_community_deposit(2), 50);
	})
}

// 0010-Q1: the payer pays the whole payment or registers nothing.
#[test]
fn a_registration_the_payer_cannot_afford_registers_nothing() {
	new_test_ext().execute_with(|| {
		// Enough for the consumption part, but not for both.
		assert_ok!(Balances::mint_into(&BOB, 80));

		assert!(register_as(BOB, 1, ALICE).is_err());

		assert!(!Communities::community_exists(&1));
		assert_eq!(Balances::balance(&BOB), 80);
		assert_eq!(Balances::balance(&Treasury::get()), 0);
		assert_eq!(CommunityDeposits::<Test>::get(1), None);
	})
}

// DEC-36
#[test]
fn a_community_collection_is_owned_and_administered_by_the_memberships_manager() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 0);

		let manager = MembershipsManagerAccountId::get();
		assert_eq!(Memberships::collection_owner(1), Some(manager.clone()));
		let roles = pallet_nfts::CollectionRoleOf::<Test>::get(1, &manager).expect("the manager has roles");
		assert!(roles.has_role(pallet_nfts::CollectionRole::Admin));
		assert!(pallet_nfts::CollectionRoleOf::<Test>::get(1, community_account(1)).is_none());
	})
}

// AC-F1.1, REQ-MI-1, REQ-MI-2, REQ-MI-D1
#[test]
fn issuing_places_memberships_in_the_stock_and_holds_their_deposits() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 3);

		assert_eq!(stock(1), vec![0, 1, 2]);
		for id in 0..3 {
			assert_eq!(Memberships::owner(1, id), Some(community_account(1)));
			assert_eq!(Deposits::<Test>::get((1, id)), Some(10));
		}
		assert_eq!(held_deposits(1), 30);
		assert_eq!(NextMembershipId::<Test>::get(), 3);
		System::assert_last_event(
			Event::<Test>::MembershipsIssued {
				community: 1,
				first: 0,
				amount: 3,
			}
			.into(),
		);
	})
}

// AC-F1.3, REQ-MI-14
#[test]
fn identifiers_are_never_reused_across_communities() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		register_community(2, BOB, 1_000);

		issue(ALICE, 2);
		issue(BOB, 2);
		issue(ALICE, 1);

		assert_eq!(stock(1), vec![0, 1, 4]);
		assert_eq!(stock(2), vec![2, 3]);
		assert_eq!(NextMembershipId::<Test>::get(), 5);
	})
}

// AC-F2.1, REQ-MI-D1
#[test]
fn an_issuance_the_account_cannot_cover_is_refused_and_issues_nothing() {
	new_test_ext().execute_with(|| {
		// Enough for two deposits, not for three.
		register_community(1, ALICE, 25);

		assert_noop!(
			CommunitiesManager::issue_memberships(RuntimeOrigin::signed(ALICE), 3),
			Error::<Test>::MembershipLimit
		);

		assert!(stock(1).is_empty());
		assert_eq!(held_deposits(1), 0);
		assert_eq!(NextMembershipId::<Test>::get(), 0);
		assert_eq!(Deposits::<Test>::iter().count(), 0);
	})
}

// ERR-TooManyMemberships
#[test]
fn an_issuance_beyond_the_limit_is_refused() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);

		assert_noop!(
			CommunitiesManager::issue_memberships(RuntimeOrigin::signed(ALICE), 9),
			Error::<Test>::TooManyMemberships
		);
		issue(ALICE, 8);
	})
}

// DEC-31
#[test]
fn only_a_community_administrator_can_issue_or_retire() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 1);

		for origin in [RuntimeOrigin::signed(BOB), RuntimeOrigin::root(), RuntimeOrigin::none()] {
			assert!(CommunitiesManager::issue_memberships(origin.clone(), 1).is_err());
			assert!(CommunitiesManager::retire_memberships(origin, BoundedVec::try_from(vec![0]).unwrap()).is_err());
		}
		assert_eq!(stock(1), vec![0]);
	})
}

// REQ-MI-12
#[test]
fn a_membership_keeps_the_deposit_it_was_issued_with() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 1);

		MembershipDepositAmount::set(25);
		issue(ALICE, 1);

		assert_eq!(Deposits::<Test>::get((1, 0)), Some(10));
		assert_eq!(Deposits::<Test>::get((1, 1)), Some(25));
		assert_eq!(held_deposits(1), 35);

		retire(ALICE, vec![0]);
		at(1 + 5);
		assert_ok!(CommunitiesManager::complete_retirement(
			RuntimeOrigin::signed(BOB),
			1,
			0,
			attributes_of(0)
		));
		assert_eq!(held_deposits(1), 25);
	})
}

// AC-F5.1, REQ-MI-D2
#[test]
fn retiring_takes_memberships_out_of_the_stock_at_once() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 2);

		retire(ALICE, vec![0, 1]);

		assert!(stock(1).is_empty());
		assert_eq!(Memberships::owner(1, 0), Some(RetiringMembershipsAccount::get()));
		assert_eq!(Retiring::<Test>::get((1, 0)), Some(1 + 5));
		assert_eq!(Retiring::<Test>::get((1, 1)), Some(1 + 5));
		// Nothing is left to assign, and the deposits are still held.
		assert!(Communities::add_member(RuntimeOrigin::signed(ALICE), BOB).is_err());
		assert_eq!(held_deposits(1), 20);
		System::assert_last_event(
			Event::<Test>::MembershipsRetiring {
				community: 1,
				amount: 2,
				completable_at: 6,
			}
			.into(),
		);
	})
}

// AC-F5.3, ERR-NotRetirable
#[test]
fn an_assigned_membership_cannot_be_retired() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 2);
		assert_ok!(Communities::add_member(RuntimeOrigin::signed(ALICE), BOB));
		let (_, assigned) = MembershipsManager::user_memberships(&BOB, Some(1))
			.next()
			.expect("BOB holds a membership");

		assert_noop!(
			CommunitiesManager::retire_memberships(
				RuntimeOrigin::signed(ALICE),
				BoundedVec::try_from(vec![assigned]).unwrap()
			),
			Error::<Test>::NotRetirable
		);
		assert_eq!(Memberships::owner(1, assigned), Some(BOB));
	})
}

// ERR-NotRetirable
#[test]
fn a_community_cannot_retire_the_memberships_of_another() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		register_community(2, BOB, 1_000);
		issue(BOB, 1);

		assert_noop!(
			CommunitiesManager::retire_memberships(
				RuntimeOrigin::signed(ALICE),
				BoundedVec::try_from(vec![0]).unwrap()
			),
			Error::<Test>::NotRetirable
		);
		assert_eq!(stock(2), vec![0]);
	})
}

// AC-F5.2, ERR-RetirementNotElapsed
#[test]
fn a_retirement_cannot_complete_before_its_delay_has_passed() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 1);
		retire(ALICE, vec![0]);

		at(1 + 4);
		assert_noop!(
			CommunitiesManager::complete_retirement(RuntimeOrigin::signed(BOB), 1, 0, attributes_of(0)),
			Error::<Test>::RetirementNotElapsed
		);

		at(1 + 5);
		assert_ok!(CommunitiesManager::complete_retirement(
			RuntimeOrigin::signed(BOB),
			1,
			0,
			attributes_of(0)
		));
	})
}

// ERR-RetirementNotElapsed
#[test]
fn only_a_retiring_membership_can_be_completed() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 1);

		assert_noop!(
			CommunitiesManager::complete_retirement(RuntimeOrigin::signed(BOB), 1, 0, attributes_of(0)),
			Error::<Test>::NotRetiring
		);
		assert_noop!(
			CommunitiesManager::complete_retirement(RuntimeOrigin::none(), 1, 0, 0),
			sp_runtime::DispatchError::BadOrigin
		);
	})
}

// AC-F5.2, AC-F3.2, REQ-MI-4, REQ-MI-D2, 0009-A2
#[test]
fn completing_a_retirement_burns_the_item_clears_every_attribute_and_releases_the_deposit() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 2);
		let free_before = Balances::balance(&community_account(1));
		retire(ALICE, vec![0]);

		// Whatever others wrote, in each namespace of the collection system.
		let attribute = |namespace: AttributeNamespace<AccountId>, key: &[u8]| {
			assert_ok!(Memberships::force_set_attribute(
				RuntimeOrigin::root(),
				None,
				1,
				Some(0),
				namespace,
				BoundedVec::try_from(key.to_vec()).unwrap(),
				BoundedVec::try_from(b"value".to_vec()).unwrap(),
			));
		};
		attribute(AttributeNamespace::Pallet, b"in the pallet namespace");
		attribute(AttributeNamespace::ItemOwner, b"by the owner");
		attribute(AttributeNamespace::Account(CAROL), b"by an account");
		attribute(AttributeNamespace::CollectionOwner, b"by the collection owner");
		// The neighbour keeps its own.
		assert_ok!(Memberships::force_set_attribute(
			RuntimeOrigin::root(),
			None,
			1,
			Some(1),
			AttributeNamespace::ItemOwner,
			BoundedVec::try_from(b"theirs".to_vec()).unwrap(),
			BoundedVec::try_from(b"value".to_vec()).unwrap(),
		));

		at(1 + 5);
		assert_ok!(CommunitiesManager::complete_retirement(
			RuntimeOrigin::signed(CAROL),
			1,
			0,
			attributes_of(0)
		));

		assert_eq!(Memberships::owner(1, 0), None);
		assert_eq!(pallet_nfts::Attribute::<Test>::iter_prefix((1, Some(0))).count(), 0);
		assert!(pallet_nfts::Attribute::<Test>::contains_key((
			1,
			Some(1),
			AttributeNamespace::<AccountId>::ItemOwner,
			BoundedVec::<u8, _>::try_from(b"theirs".to_vec()).unwrap(),
		)));
		assert_eq!(Deposits::<Test>::get((1, 0)), None);
		assert_eq!(Retiring::<Test>::get((1, 0)), None);
		assert_eq!(held_deposits(1), 10);
		assert_eq!(Balances::balance(&community_account(1)), free_before + 10);
		assert_eq!(stock(1), vec![1]);
		System::assert_last_event(
			Event::<Test>::RetirementCompleted {
				community: 1,
				membership_id: 0,
			}
			.into(),
		);
	})
}

#[test]
fn weights() {
	let max_total_extrinsics = MAX_BLOCK_WEIGHT * NORMAL_DISPATCH_RATIO;
	let max_extrinsic_weight = max_total_extrinsics
		.saturating_sub(MAX_BLOCK_WEIGHT * AVERAGE_ON_INITIALIZE_RATIO)
		.saturating_sub(BASE_EXTRINSIC);

	assert_eq!(max_extrinsic_weight, Weight::from_parts(349_875_000_000, 3_670_016));

	println!("max block weight: {MAX_BLOCK_WEIGHT}");
	println!("max total extrinsics weight: {max_total_extrinsics}");
	println!("max extrinsic weight: {max_extrinsic_weight}\n");

	let mut total = Weight::zero();

	let calls = vec![("register", SubstrateWeight::<Test>::register())];

	for (function, weight) in calls {
		println!("{function}: {weight:?}",);
		println!(
			"\tpercentage of max extrinsic weight: {:.2}% (ref_time), {:.2}% (proof_size)",
			(weight.ref_time() as f64 / max_extrinsic_weight.ref_time() as f64) * 100.0,
			(weight.proof_size() as f64 / max_extrinsic_weight.proof_size() as f64) * 100.0,
		);
		println!(
			"\tmax tx per block: {} (ref_time), {} (proof_size)",
			max_extrinsic_weight.ref_time() / weight.ref_time(),
			max_extrinsic_weight.proof_size() / weight.proof_size().max(1)
		);

		assert!(weight.all_lt(max_extrinsic_weight));

		total += weight;
	}

	// output total weight, useful for evaluating net weight changes when optimising
	println!("\ntotal weight: {total:?}");
}

/// How many attributes an item carries, in every namespace.
fn attributes_of(item: MembershipId) -> u32 {
	pallet_nfts::Attribute::<Test>::iter_prefix((1, Some(item))).count() as u32
}

fn set_attributes(item: MembershipId, amount: u32) {
	for i in 0..amount {
		assert_ok!(Memberships::force_set_attribute(
			RuntimeOrigin::root(),
			None,
			1,
			Some(item),
			AttributeNamespace::Pallet,
			BoundedVec::try_from(format!("extra {i}").into_bytes()).unwrap(),
			BoundedVec::try_from(b"value".to_vec()).unwrap(),
		));
	}
}

// M-1 of the review of #515
#[test]
fn completing_a_retirement_is_refused_when_the_item_carries_more_attributes_than_the_witness() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 1);
		retire(ALICE, vec![0]);
		set_attributes(0, 3);
		let carried = attributes_of(0);

		at(1 + 5);
		assert_noop!(
			CommunitiesManager::complete_retirement(RuntimeOrigin::signed(BOB), 1, 0, carried - 1),
			Error::<Test>::TooManyAttributes
		);
		assert_eq!(attributes_of(0), carried);
	})
}

#[test]
fn completing_a_retirement_with_the_exact_witness_passes() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 1);
		retire(ALICE, vec![0]);
		set_attributes(0, 3);

		at(1 + 5);
		assert_ok!(CommunitiesManager::complete_retirement(
			RuntimeOrigin::signed(BOB),
			1,
			0,
			attributes_of(0)
		));
		assert_eq!(attributes_of(0), 0);
	})
}

#[test]
fn completing_a_retirement_returns_the_weight_of_the_attributes_it_cleared() {
	new_test_ext().execute_with(|| {
		register_community(1, ALICE, 1_000);
		issue(ALICE, 2);
		retire(ALICE, vec![0, 1]);
		set_attributes(0, 5);
		set_attributes(1, 1);
		let (many, few) = (attributes_of(0), attributes_of(1));
		assert!(many > few);

		at(1 + 5);
		// The same witness for both: the item with fewer attributes weighs less.
		let heavy = CommunitiesManager::complete_retirement(RuntimeOrigin::signed(BOB), 1, 0, many).unwrap();
		let light = CommunitiesManager::complete_retirement(RuntimeOrigin::signed(BOB), 1, 1, many).unwrap();

		let weight = |a| <Test as crate::Config>::WeightInfo::complete_retirement(a);
		assert_eq!(heavy.actual_weight, Some(weight(many)));
		assert_eq!(light.actual_weight, Some(weight(few)));
		assert!(light.actual_weight.unwrap().ref_time() < heavy.actual_weight.unwrap().ref_time());
	})
}
