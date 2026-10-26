//! `ReferendaTrackIds`, which re-keys the community referenda's storage from `u16` track ids to
//! `u32` ones (`combine(old, 0)`), and the registration of the tracks migration.

use super::*;

use crate::{
	config::communities::governance::{CommunityReferendaInstance as I, CommunityTracksInstance},
	config::system::MigrationsMaxServiceWeight,
	migrations::referenda_rekey::*,
	AccountId, BlockNumber,
};
use frame_support::StorageHasher;
use frame_support::{
	migrations::{SteppedMigration, SteppedMigrationError},
	storage::unhashed,
	traits::{schedule::DispatchTime, Bounded},
	weights::{Weight, WeightMeter},
	BoundedVec, Twox64Concat,
};
use pallet_referenda::{
	DecidingCount, DecidingStatus, Deposit, ReferendumInfo, ReferendumInfoFor, ReferendumStatus, TrackQueue,
};
use pallet_referenda_tracks::SplitId;
use parity_scale_codec::{Decode, Encode};

type Migration = ReferendaTrackIds;

fn status<Track: Eq + core::fmt::Debug + Encode + Decode + scale_info::TypeInfo + Clone>(
	track: Track,
	submitted: BlockNumber,
) -> ReferendumStatus<
	Track,
	pallet_referenda::PalletsOriginOf<Runtime>,
	BlockNumber,
	pallet_referenda::BoundedCallOf<Runtime, I>,
	pallet_referenda::BalanceOf<Runtime, I>,
	pallet_referenda::TallyOf<Runtime, I>,
	AccountId,
	pallet_referenda::ScheduleAddressOf<Runtime, I>,
> {
	ReferendumStatus {
		track,
		origin: frame_system::RawOrigin::Root.into(),
		proposal: Bounded::Inline(BoundedVec::truncate_from(b"proposal".to_vec())),
		enactment: DispatchTime::After(10),
		submitted,
		submission_deposit: Deposit {
			who: AccountId::new([1; 32]),
			amount: 5,
		},
		decision_deposit: None,
		deciding: Some(DecidingStatus {
			since: submitted + 1,
			confirming: None,
		}),
		tally: Default::default(),
		in_queue: false,
		alarm: Some((submitted + 20, (submitted + 20, 0))),
	}
}

fn put_referendum(index: u32, info: &Info<u16>) {
	unhashed::put(&ReferendumInfoFor::<Runtime, I>::hashed_key_for(index), info);
}

fn old_key(prefix_of: &[u8], track: u16) -> Vec<u8> {
	[prefix_of, &Twox64Concat::hash(&track.encode())[..]].concat()
}

fn prefix(name: &[u8]) -> Vec<u8> {
	frame_support::storage::storage_prefix(b"CommunityReferenda", name).to_vec()
}

fn queue() -> Vec<(u32, u32)> {
	vec![(3, 100), (4, 50)]
}

/// Community referenda as they were before the upgrade.
fn populate() {
	put_referendum(0, &ReferendumInfo::Ongoing(status(7u16, 100)));
	put_referendum(1, &ReferendumInfo::Ongoing(status(9u16, 110)));
	put_referendum(2, &ReferendumInfo::Approved(120, None, None));
	for (track, count) in [(7u16, 3u32), (9, 1)] {
		unhashed::put(&old_key(&prefix(b"DecidingCount"), track), &count);
		unhashed::put(&old_key(&prefix(b"TrackQueue"), track), &queue());
	}
}

fn run(meter_limit: Weight, from: Option<Cursor>) -> Option<Cursor> {
	let mut meter = WeightMeter::with_limit(meter_limit);
	Migration::step(from, &mut meter).expect("it does not fail")
}

fn migrate() {
	let mut cursor = None;
	for _ in 0..1_000 {
		cursor = run(MigrationsMaxServiceWeight::get(), cursor);
		if cursor.is_none() {
			return;
		}
	}
	panic!("it did not finish");
}

fn root() -> Vec<u8> {
	sp_io::storage::root(sp_runtime::StateVersion::V1)
}

// An ongoing community referendum keeps everything but its track, which becomes `combine(old, 0)`.
#[test]
fn an_ongoing_referendum_gets_the_new_track_and_keeps_the_rest() {
	TestExternalities::default().execute_with(|| {
		populate();

		migrate();

		let Some(ReferendumInfo::Ongoing(migrated)) = ReferendumInfoFor::<Runtime, I>::get(0) else {
			panic!("the referendum is ongoing");
		};
		let expected = status(<u32 as SplitId>::combine(7, 0), 100);
		assert_eq!(migrated, expected);
		let Some(ReferendumInfo::Ongoing(second)) = ReferendumInfoFor::<Runtime, I>::get(1) else {
			panic!("the referendum is ongoing");
		};
		assert_eq!(second.track, <u32 as SplitId>::combine(9, 0));
	})
}

// A finished referendum carries no track, and is left as it was.
#[test]
fn a_finished_referendum_is_untouched() {
	TestExternalities::default().execute_with(|| {
		populate();
		let key = ReferendumInfoFor::<Runtime, I>::hashed_key_for(2);
		let before = unhashed::get_raw(&key);

		migrate();

		assert_eq!(unhashed::get_raw(&key), before);
	})
}

// `DecidingCount` and `TrackQueue` move to the key of the new track, and the old keys are gone.
#[test]
fn the_deciding_counts_and_the_queues_move_to_the_new_keys() {
	TestExternalities::default().execute_with(|| {
		populate();

		migrate();

		for (track, count) in [(7u16, 3u32), (9, 1)] {
			let new = <u32 as SplitId>::combine(track, 0);
			assert_eq!(DecidingCount::<Runtime, I>::get(new), count);
			assert_eq!(TrackQueue::<Runtime, I>::get(new).into_inner(), queue());
			assert!(!unhashed::exists(&old_key(&prefix(b"DecidingCount"), track)));
			assert!(!unhashed::exists(&old_key(&prefix(b"TrackQueue"), track)));
		}
	})
}

// A second run changes nothing.
#[test]
fn a_second_run_changes_nothing() {
	TestExternalities::default().execute_with(|| {
		populate();
		migrate();
		let after_the_first = root();

		migrate();

		assert_eq!(root(), after_the_first);
	})
}

// With a meter for two units it takes many blocks, and ends in the state a single run does.
#[test]
fn it_yields_and_resumes_to_the_same_state() {
	let in_one_go = TestExternalities::default().execute_with(|| {
		populate();
		migrate();
		root()
	});

	TestExternalities::default().execute_with(|| {
		populate();
		let limit = heaviest_unit().saturating_mul(2);
		let (mut cursor, mut steps) = (None, 0);
		loop {
			cursor = run(limit, cursor);
			steps += 1;
			if cursor.is_none() {
				break;
			}
			assert!(steps < 1_000);
		}
		assert!(steps > 3, "it yielded {steps} times");
		assert_eq!(root(), in_one_go);
	});
}

// What does not decode is skipped, never an error, and stays where it was.
#[test]
fn what_does_not_decode_is_skipped() {
	TestExternalities::default().execute_with(|| {
		populate();
		let counter = old_key(&prefix(b"DecidingCount"), 11);
		unhashed::put_raw(&counter, &[1]);
		let queue_key = old_key(&prefix(b"TrackQueue"), 11);
		unhashed::put_raw(&queue_key, &[0xff, 0xff]);
		let odd_key = [prefix(b"DecidingCount"), vec![1, 2, 3]].concat();
		unhashed::put_raw(&odd_key, &0u32.encode());
		let garbage = ReferendumInfoFor::<Runtime, I>::hashed_key_for(5);
		unhashed::put_raw(&garbage, &[9, 9, 9]);

		migrate();

		assert_eq!(unhashed::get_raw(&counter), Some(vec![1]));
		assert_eq!(unhashed::get_raw(&queue_key), Some(vec![0xff, 0xff]));
		assert!(unhashed::exists(&odd_key));
		assert_eq!(unhashed::get_raw(&garbage), Some(vec![9, 9, 9]));
		// The rest was migrated.
		assert_eq!(DecidingCount::<Runtime, I>::get(<u32 as SplitId>::combine(7, 0)), 3);
	})
}

// A tiny empty meter can never do one unit: that is the only error. A meter that has run out yields.
#[test]
fn it_fails_only_when_not_even_one_unit_fits_an_empty_meter() {
	TestExternalities::default().execute_with(|| {
		populate();

		let mut meter = WeightMeter::with_limit(Weight::from_parts(1, 1));
		assert!(matches!(
			Migration::step(None, &mut meter),
			Err(SteppedMigrationError::InsufficientWeight { .. })
		));

		let mut spent = WeightMeter::with_limit(MigrationsMaxServiceWeight::get());
		assert!(spent.try_consume(MigrationsMaxServiceWeight::get()).is_ok());
		assert!(matches!(Migration::step(None, &mut spent), Ok(Some(_))));
	})
}

// frame-contrib's `integrity_test` of the tracks migration asks the largest possible origin to fit
// the cursor key (32 + 16 + the origin). Kreivo's `PalletsOrigin` is 604 bytes at most, and the cursor
// key holds 1 024 since frame-contrib #160, so it holds (it did not at the 256 bytes of a86ebf0).
#[test]
fn the_tracks_migration_integrity_test_holds() {
	pallet_referenda_tracks::migration::MigrateV0ToV1::<
		Runtime,
		CommunityTracksInstance,
		u16,
		pallet_referenda_tracks::migration::TryIntoGroup,
	>::integrity_test();
}

#[test]
fn the_origins_the_tracks_migration_walks_fit_its_cursor() {
	use frame_support::traits::OriginTrait;
	use parity_scale_codec::MaxEncodedLen;

	// Key of `OriginToTrackId`: prefix, `Blake2_128Concat` hash, the encoded origin.
	let key_len = |origin_len: usize| 32 + 16 + origin_len;
	let community: pallet_referenda::PalletsOriginOf<Runtime> =
		pallet_communities::Origin::<Runtime>::new(u16::MAX).into();
	assert!(key_len(community.encoded_size()) <= pallet_referenda_tracks::migration::MAX_KEY_LEN as usize);

	// The worst case, which `integrity_test` checks.
	let largest = <<RuntimeOrigin as OriginTrait>::PalletsOrigin as MaxEncodedLen>::max_encoded_len();
	println!(
		"largest PalletsOrigin: {largest} bytes, key {} of {}",
		key_len(largest),
		pallet_referenda_tracks::migration::MAX_KEY_LEN
	);
}
