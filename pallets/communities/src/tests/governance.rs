use super::*;

use frame_support::{parameter_types, traits::OriginTrait};
use pallet_referenda::{BoundedCallOf, Curve, TrackInfoOf};
use parity_scale_codec::Encode;
use sp_runtime::{str_array as s, BoundedVec, TokenError};

use crate::{
	types::{Tally, Vote},
	Call, DecisionMethod,
};
use frame_support::assert_noop;
use pallet_referenda::TrackInfo;
use sp_runtime::Perbill;
use virto_common::MembershipId;

const COMMUNITY_A: CommunityId = 1;
const COMMUNITY_B: CommunityId = 2;
const COMMUNITY_C: CommunityId = 3;
const COMMUNITY_D: CommunityId = 4;

const MS_X_COMMUNITY: usize = 3;
const MEMBERSHIPS: [[MembershipId; MS_X_COMMUNITY]; 4] = {
	let mut m = [[0; MS_X_COMMUNITY]; 4];
	let mut i = 0;
	while i < MS_X_COMMUNITY * 4 {
		m[i / MS_X_COMMUNITY][i % MS_X_COMMUNITY] = (i + 1) as MembershipId;
		i += 1;
	}
	m
};
const fn memberships_of(community: CommunityId) -> &'static [MembershipId] {
	&MEMBERSHIPS[community as usize - 1]
}
const fn membership(community: CommunityId, m: usize) -> MembershipId {
	MEMBERSHIPS[community as usize - 1][m - 1]
}

const COMMUNITY_B_ASSET_ID: AssetId = 2;

const ALICE: AccountId = AccountId::new([1; 32]);
const BOB: AccountId = AccountId::new([2; 32]);
const CHARLIE: AccountId = AccountId::new([3; 32]);

parameter_types! {
	pub OriginForCommunityA: Box<OriginCaller> =
		Box::new(TestEnvBuilder::create_community_origin(&COMMUNITY_A).caller().clone());
	pub OriginForCommunityB: Box<OriginCaller> =
		Box::new(TestEnvBuilder::create_community_origin(&COMMUNITY_B).caller().clone());
	pub OriginForCommunityC: Box<OriginCaller> =
		Box::new(TestEnvBuilder::create_community_origin(&COMMUNITY_C).caller().clone());
	pub OriginForCommunityD: Box<OriginCaller> =
		Box::new(TestEnvBuilder::create_community_origin(&COMMUNITY_D).caller().clone());

	pub CommunityTrack: TrackInfoOf<Test> = TrackInfo {
		name: s("Community"),
		max_deciding: 1,
		decision_deposit: 5,
		prepare_period: 1,
		decision_period: 5,
		confirm_period: 1,
		min_enactment_period: 1,
		min_approval: Curve::LinearDecreasing {
			length: Perbill::from_percent(100),
			floor: Perbill::from_percent(50),
			ceil: Perbill::from_percent(100),
		},
		min_support: Curve::LinearDecreasing {
			length: Perbill::from_percent(100),
			floor: Perbill::from_percent(0),
			ceil: Perbill::from_percent(100),
		},
	};

	pub ProposalCallAddBob: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::add_member { who: BOB }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};

	pub ProposalCallAddAlice: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::add_member { who: ALICE }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};

	pub ProposalCallRemoveCharlieFromC: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::remove_member { who: CHARLIE, membership_id: membership(COMMUNITY_C, 3) }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};

	pub ProposalCallPromoteCharlie: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::promote { membership_id: membership(COMMUNITY_D, 3) }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};
}

fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = TestEnvBuilder::new()
		.with_balances(&[(ALICE, 15), (BOB, 15), (CHARLIE, 15)])
		// Membership-based
		.add_community(
			COMMUNITY_A,
			DecisionMethod::Membership,
			&[ALICE],
			memberships_of(COMMUNITY_A),
			Some(CommunityTrack::get()),
		)
		// Community-asset based
		.add_community(
			COMMUNITY_B,
			DecisionMethod::CommunityAsset(COMMUNITY_B_ASSET_ID, 10),
			&[BOB, CHARLIE],
			memberships_of(COMMUNITY_B),
			Some(CommunityTrack::get()),
		)
		.add_asset(
			&COMMUNITY_B_ASSET_ID,
			&Communities::community_account(&COMMUNITY_B),
			true,
			1,
			None,
			Some(vec![(BOB, 50), (CHARLIE, 50)]),
		)
		// Native-asset based
		.add_community(
			COMMUNITY_C,
			DecisionMethod::NativeToken,
			&[ALICE, BOB, CHARLIE],
			memberships_of(COMMUNITY_C),
			Some(CommunityTrack::get()),
		)
		// Rank-based
		.add_community(
			COMMUNITY_D,
			DecisionMethod::Rank,
			&[ALICE, BOB, CHARLIE],
			memberships_of(COMMUNITY_D),
			Some(CommunityTrack::get()),
		)
		.build();

	t.execute_with(|| {
		assert_ok!(Referenda::submit(
			RuntimeOrigin::signed(ALICE),
			OriginForCommunityA::get(),
			ProposalCallAddBob::get(),
			frame_support::traits::schedule::DispatchTime::After(1),
		));
		assert_ok!(Referenda::submit(
			RuntimeOrigin::signed(BOB),
			OriginForCommunityB::get(),
			ProposalCallAddAlice::get(),
			frame_support::traits::schedule::DispatchTime::After(1),
		));
		assert_ok!(Referenda::submit(
			RuntimeOrigin::signed(BOB),
			OriginForCommunityC::get(),
			ProposalCallRemoveCharlieFromC::get(),
			frame_support::traits::schedule::DispatchTime::After(1),
		));

		System::assert_has_event(
			pallet_referenda::Event::<Test>::Submitted {
				index: 0,
				proposal: ProposalCallAddBob::get(),
				track: COMMUNITY_A,
			}
			.into(),
		);
		System::assert_has_event(
			pallet_referenda::Event::<Test>::Submitted {
				index: 1,
				proposal: ProposalCallAddAlice::get(),
				track: COMMUNITY_B,
			}
			.into(),
		);
		System::assert_has_event(
			pallet_referenda::Event::<Test>::Submitted {
				index: 2,
				proposal: ProposalCallRemoveCharlieFromC::get(),
				track: COMMUNITY_C,
			}
			.into(),
		);

		assert_ok!(Referenda::place_decision_deposit(RuntimeOrigin::signed(ALICE), 0));
		assert_ok!(Referenda::place_decision_deposit(RuntimeOrigin::signed(BOB), 1));
		assert_ok!(Referenda::place_decision_deposit(RuntimeOrigin::signed(BOB), 2));

		tick_block();
	});

	t
}

mod vote {
	use super::*;

	mod common {
		use super::*;

		#[test]
		fn fails_if_vote_weight_is_zero() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						membership(COMMUNITY_B, 1),
						1,
						Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 0)
					),
					Error::VoteBelowMinimum
				);
			});
		}

		#[test]
		fn fails_if_not_a_member() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						membership(COMMUNITY_A, 2),
						0,
						Vote::Standard(true)
					),
					Error::NotAMember
				);
			});
		}

		#[test]
		fn fails_if_poll_is_not_ongoing() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(ALICE),
						membership(COMMUNITY_A, 1),
						256,
						Vote::Standard(true)
					),
					Error::NotOngoing
				);
			});
		}

		#[test]
		fn fails_if_voting_on_invalid_track() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						membership(COMMUNITY_B, 1),
						0,
						Vote::Standard(true)
					),
					Error::InvalidTrack
				);
			});
		}

		#[test]
		fn transferring_memberships_does_not_lead_to_double_voting() {
			new_test_ext().execute_with(|| {
				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					membership(COMMUNITY_A, 1),
					0,
					Vote::Standard(true)
				));

				System::assert_last_event(
					crate::Event::VoteCasted {
						who: ALICE,
						poll_index: 0,
						vote: Vote::Standard(true),
					}
					.into(),
				);

				assert_ok!(Nfts::transfer(
					RuntimeOrigin::signed(ALICE),
					COMMUNITY_A,
					membership(COMMUNITY_A, 1),
					BOB
				));

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_A, 1),
					0,
					Vote::Standard(true)
				));

				System::assert_last_event(
					crate::Event::VoteCasted {
						who: BOB,
						poll_index: 0,
						vote: Vote::Standard(true),
					}
					.into(),
				);

				use frame_support::traits::Polling;
				assert_eq!(
					Referenda::as_ongoing(0),
					Some((
						Tally {
							ayes: 1,
							bare_ayes: 1,
							nays: 0,
							..Default::default()
						},
						COMMUNITY_A
					))
				);
			});
		}
	}

	mod membership {
		use frame_contrib_traits::memberships::Inspect;

		use super::*;

		#[test]
		fn passing_poll_executes() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 0,
						track: COMMUNITY_A,
						proposal: ProposalCallAddBob::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					membership(COMMUNITY_A, 1),
					0,
					Vote::Standard(true)
				));

				tick_block();

				// After voting, the poll starts confirmation
				System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 0 }.into());

				tick_block();

				// After confirmation, vote should be completed and approved
				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 0,
						tally: Tally {
							ayes: 1,
							nays: 0,
							bare_ayes: 1,
							..Default::default()
						},
					}
					.into(),
				);

				let community_account = Communities::community_account(&COMMUNITY_A);
				let (_, membership_id) = MembershipsManager::user_memberships(&community_account, None)
					.next()
					.expect("CommunityA should still have memberships");

				tick_block();

				// Proposal is enacted and exeuted
				System::assert_has_event(
					crate::Event::<Test>::MemberAdded {
						who: BOB,
						membership_id,
					}
					.into(),
				);
			});
		}

		#[test]
		fn poll_rejects_on_single_nay() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 0,
						track: COMMUNITY_A,
						proposal: ProposalCallAddBob::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					membership(COMMUNITY_A, 1),
					0,
					Vote::Standard(false)
				));

				tick_blocks(5);

				// After voting, the poll should be completed and approved
				System::assert_has_event(
					pallet_referenda::Event::<Test>::Rejected {
						index: 0,
						tally: Tally {
							ayes: 0,
							nays: 1,
							bare_ayes: 0,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}

		#[test]
		fn tie_breaking_works() {
			fn run_referenda() -> sp_io::TestExternalities {
				let mut ext = new_test_ext();

				ext.execute_with(|| {
					// For now, this community will vote membership-based
					assert_ok!(Communities::set_decision_method(
						TestEnvBuilder::create_community_origin(&COMMUNITY_C),
						COMMUNITY_C,
						DecisionMethod::Membership
					));

					// Before voting, the poll is ongoing
					System::assert_has_event(
						pallet_referenda::Event::<Test>::DecisionStarted {
							index: 2,
							track: COMMUNITY_C,
							proposal: ProposalCallRemoveCharlieFromC::get(),
							tally: Tally::default(),
						}
						.into(),
					);

					assert_ok!(Communities::vote(
						RuntimeOrigin::signed(ALICE),
						membership(COMMUNITY_C, 1),
						2,
						Vote::Standard(true)
					));

					tick_block();

					assert_ok!(Communities::vote(
						RuntimeOrigin::signed(CHARLIE),
						membership(COMMUNITY_C, 3),
						2,
						Vote::Standard(false)
					));

					tick_blocks(4);

					// After voting, the poll starts confirmation
					System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 2 }.into());
				});

				ext
			}

			run_referenda().execute_with(|| {
				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_C, 2),
					2,
					Vote::Standard(true)
				));

				tick_block();

				// After confirmation, vote should be completed and approved
				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 2,
						tally: Tally {
							ayes: 2,
							nays: 1,
							bare_ayes: 2,
							..Default::default()
						},
					}
					.into(),
				);
			});

			run_referenda().execute_with(|| {
				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_C, 2),
					2,
					Vote::Standard(false)
				));

				tick_block();

				// After voting, the poll starts confirmation
				System::assert_has_event(
					pallet_referenda::Event::<Test>::Rejected {
						index: 2,
						tally: Tally {
							ayes: 1,
							nays: 2,
							bare_ayes: 1,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}
	}

	mod asset_balance {
		use super::*;

		#[test]
		fn fails_if_not_enough_balance() {
			new_test_ext().execute_with(|| {
				// Cannot keep free funds lower than min. balance
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(CHARLIE),
						membership(COMMUNITY_B, 2),
						1,
						Vote::AssetBalance(false, COMMUNITY_B_ASSET_ID, 51)
					),
					TokenError::FundsUnavailable
				);
			});
		}

		#[test]
		fn fails_if_asset_vote_weight_is_under_minimum() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						membership(COMMUNITY_B, 1),
						1,
						Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 9)
					),
					Error::VoteBelowMinimum
				);
				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_B, 1),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 10)
				));
			});
		}

		#[test]
		fn passes_with_approval_and_support() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 1,
						track: COMMUNITY_B,
						proposal: ProposalCallAddAlice::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				// We're going to vote high enough to pass to confirmation immediately:
				// 66% approval / 10% of support

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_B, 1),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 30)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_B, 2),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 30)
				));

				tick_block();

				System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 1 }.into());

				tick_block();

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 1,
						tally: Tally {
							ayes: 60,
							nays: 0,
							bare_ayes: 60,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}

		#[test]
		fn passes_with_approval_but_not_support() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 1,
						track: COMMUNITY_B,
						proposal: ProposalCallAddAlice::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				// We're going to vote high enough to have a pass in approval, but not enough to
				// pass in support until decision period ends: 66% approval / 10% of support

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_B, 1),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 12)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_B, 2),
					1,
					Vote::AssetBalance(false, COMMUNITY_B_ASSET_ID, 11)
				));

				tick_blocks(4);

				System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 1 }.into());

				tick_block();

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 1,
						tally: Tally {
							ayes: 12,
							nays: 11,
							bare_ayes: 12,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}

		#[test]
		fn voter_can_change_decision_over_time() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 1,
						track: COMMUNITY_B,
						proposal: ProposalCallAddAlice::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				tick_block();

				// We're going to vote high in a series of three votes, each one attempting to
				// turn the poll over.

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_B, 2),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 11)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_B, 1),
					1,
					Vote::AssetBalance(false, COMMUNITY_B_ASSET_ID, 12)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_B, 2),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 13)
				));

				tick_blocks(3);

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 1,
						tally: Tally {
							ayes: 13,
							nays: 12,
							bare_ayes: 13,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}
	}

	mod native_balance {
		use super::*;

		#[test]
		fn fails_if_not_enough_balance() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						membership(COMMUNITY_C, 2),
						2,
						Vote::NativeBalance(true, 16)
					),
					TokenError::FundsUnavailable
				);
			});
		}

		#[test]
		fn rejects_on_most_nays() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 2,
						track: COMMUNITY_C,
						proposal: ProposalCallRemoveCharlieFromC::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				tick_block();

				// BOB won't be able to vote, since they don't have enough funds to do so
				// ALICE has a limited support they can put, so CHARLIE will be able to
				// cast a majority nay vote.

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_C, 3),
					2,
					Vote::NativeBalance(false, 14)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					membership(COMMUNITY_C, 1),
					2,
					Vote::NativeBalance(true, 7)
				));

				tick_blocks(3);

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Rejected {
						index: 2,
						tally: Tally {
							ayes: 7,
							nays: 14,
							bare_ayes: 7,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}

		#[test]
		fn voter_can_change_decision() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 2,
						track: COMMUNITY_C,
						proposal: ProposalCallRemoveCharlieFromC::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				tick_block();

				// We're going to vote high in a series of three votes, each one attempting to
				// turn the poll over.

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_C, 3),
					2,
					Vote::NativeBalance(false, 6)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					membership(COMMUNITY_C, 1),
					2,
					Vote::NativeBalance(true, 7)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_C, 3),
					2,
					Vote::NativeBalance(false, 8)
				));

				tick_blocks(2);

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Rejected {
						index: 2,
						tally: Tally {
							ayes: 7,
							nays: 8,
							bare_ayes: 7,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}
	}

	mod rank {
		use frame_support::traits::Polling;

		use super::*;

		fn new_test_ext() -> sp_io::TestExternalities {
			let mut ext = super::new_test_ext();

			ext.execute_with(|| {
				assert_ok!(Communities::promote(
					Into::<RuntimeOrigin>::into(*OriginForCommunityD::get()),
					membership(COMMUNITY_D, 1)
				));
				assert_ok!(Communities::promote(
					Into::<RuntimeOrigin>::into(*OriginForCommunityD::get()),
					membership(COMMUNITY_D, 2)
				));
				assert_ok!(Communities::promote(
					Into::<RuntimeOrigin>::into(*OriginForCommunityD::get()),
					membership(COMMUNITY_D, 3)
				));

				assert_ok!(Referenda::submit(
					RuntimeOrigin::signed(CHARLIE),
					OriginForCommunityD::get(),
					ProposalCallPromoteCharlie::get(),
					frame_support::traits::schedule::DispatchTime::After(1),
				));

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Submitted {
						index: 3,
						proposal: ProposalCallPromoteCharlie::get(),
						track: COMMUNITY_D,
					}
					.into(),
				);

				assert_ok!(Referenda::place_decision_deposit(RuntimeOrigin::signed(CHARLIE), 3));

				tick_block();
			});

			ext
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					membership(COMMUNITY_D, 1),
					3,
					Vote::Standard(true)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_D, 2),
					3,
					Vote::Standard(false)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_D, 3),
					3,
					Vote::Standard(true)
				));

				tick_blocks(2);

				System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 3 }.into());
			});
		}

		#[test]
		fn it_works_with_different_ranks() {
			new_test_ext().execute_with(|| {
				assert_ok!(Communities::promote(
					Into::<RuntimeOrigin>::into(*OriginForCommunityD::get()),
					membership(COMMUNITY_D, 1)
				));

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					membership(COMMUNITY_D, 1),
					3,
					Vote::Standard(false)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					membership(COMMUNITY_D, 2),
					3,
					Vote::Standard(true)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					membership(COMMUNITY_D, 3),
					3,
					Vote::Standard(true)
				));

				assert_eq!(
					Referenda::as_ongoing(3).expect("the poll was initiated; qed").0,
					Tally {
						ayes: 2,
						nays: 2,
						bare_ayes: 2,
						..Default::default()
					}
				)
			});
		}
	}
}

mod remove_vote {
	use frame_support::traits::{fungible::Inspect, Polling};

	use super::*;

	#[test]
	fn fails_if_not_a_member() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Communities::remove_vote(RuntimeOrigin::signed(BOB), membership(COMMUNITY_A, 2), 0,),
				Error::NotAMember
			);
		});
	}

	#[test]
	fn fails_if_trying_to_remove_vote_from_invalid_track() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Communities::remove_vote(RuntimeOrigin::signed(ALICE), membership(COMMUNITY_A, 1), 1),
				Error::InvalidTrack
			);
		});
	}

	#[test]
	fn fails_if_poll_is_no_vote_casted() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Communities::remove_vote(RuntimeOrigin::signed(ALICE), membership(COMMUNITY_A, 1), 0),
				Error::NoVoteCasted
			);
		});
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(Communities::vote(
				RuntimeOrigin::signed(ALICE),
				membership(COMMUNITY_A, 1),
				0,
				Vote::Standard(true)
			));

			tick_block();

			assert_ok!(Communities::remove_vote(
				RuntimeOrigin::signed(ALICE),
				membership(COMMUNITY_A, 1),
				0
			));

			System::assert_has_event(
				crate::Event::<Test>::VoteRemoved {
					who: ALICE,
					poll_index: 0,
				}
				.into(),
			);

			assert_eq!(
				Referenda::as_ongoing(0).expect("we already created poll 0; qed").0,
				Tally::default()
			);
		});

		new_test_ext().execute_with(|| {
			assert_ok!(Communities::vote(
				RuntimeOrigin::signed(ALICE),
				membership(COMMUNITY_C, 1),
				2,
				Vote::NativeBalance(true, 15)
			));

			assert_eq!(
				Balances::reducible_balance(
					&ALICE,
					frame_support::traits::tokens::Preservation::Expendable,
					frame_support::traits::tokens::Fortitude::Polite
				),
				0
			);

			tick_block();

			assert_ok!(Communities::remove_vote(
				RuntimeOrigin::signed(ALICE),
				membership(COMMUNITY_C, 1),
				2
			));

			System::assert_has_event(
				crate::Event::<Test>::VoteRemoved {
					who: ALICE,
					poll_index: 2,
				}
				.into(),
			);

			assert_eq!(
				Referenda::as_ongoing(2).expect("we already created poll 2; qed").0,
				Tally::default()
			);

			assert_eq!(
				Balances::reducible_balance(
					&ALICE,
					frame_support::traits::tokens::Preservation::Expendable,
					frame_support::traits::tokens::Fortitude::Polite
				),
				7
			);
		});
	}
}

mod unlock {
	use super::*;

	#[test]
	fn fails_if_trying_to_unlock_on_an_ongoing_poll() {
		new_test_ext().execute_with(|| {
			// Since BOB never casted a vote, a lock wasn't put in place
			assert_noop!(
				Communities::unlock(RuntimeOrigin::signed(BOB), 1),
				Error::AlreadyOngoing
			);
		});
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(Communities::vote(
				RuntimeOrigin::signed(BOB),
				membership(COMMUNITY_B, 1),
				1,
				Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 15)
			));

			assert_ok!(Communities::vote(
				RuntimeOrigin::signed(CHARLIE),
				membership(COMMUNITY_C, 3),
				2,
				Vote::NativeBalance(true, 15)
			));

			tick_blocks(6);

			assert_ok!(Communities::unlock(RuntimeOrigin::signed(BOB), 1));

			assert_ok!(Communities::unlock(RuntimeOrigin::signed(CHARLIE), 2));
		});
	}
}
