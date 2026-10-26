//! The calls that the benchmarks of `pallet_referenda_tracks` make on the community tracks, with the
//! origin they make them with: root. The `CreateOrigin` and `GroupManagerCreateOrigin` of Kreivo are
//! `EnsureNever`, except in the benchmark build, where root is allowed to create groups and
//! sub-tracks (frame-contrib's own benchmarks need it).

use super::*;

use crate::{config::communities::governance::CommunityTracksInstance, CommunityTracks};
use frame_support::assert_ok;
use pallet_referenda::{Curve, TrackInfo};
use pallet_referenda_tracks::{Pallet as Tracks, SplitId, TracksIds};
use sp_runtime::Perbill;

type Instance = CommunityTracksInstance;

fn info() -> TrackInfo<crate::Balance, crate::BlockNumber> {
	TrackInfo {
		name: sp_runtime::str_array("Community"),
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
	}
}

fn origin(who: u8) -> pallet_referenda::PalletsOriginOf<Runtime> {
	frame_system::RawOrigin::Signed(AccountId32::new([who; 32])).into()
}

// `new_group_with_track` benchmark: root creates a group with its first track.
#[test]
fn root_creates_a_group_with_its_first_track() {
	TestExternalities::default().execute_with(|| {
		assert_ok!(Tracks::<Runtime, Instance>::new_group_with_track(
			RuntimeOrigin::root(),
			origin(1),
			info()
		));
		assert_eq!(TracksIds::<Runtime, Instance>::get().len(), 1);
	})
}

// `add_sub_track` benchmark: root adds a sub-track.
#[test]
fn root_adds_a_sub_track() {
	TestExternalities::default().execute_with(|| {
		assert_ok!(Tracks::<Runtime, Instance>::add_sub_track(
			RuntimeOrigin::root(),
			origin(1),
			info()
		));
		assert_eq!(TracksIds::<Runtime, Instance>::get().len(), 1);
	})
}

// `remove_group` benchmark: a group is built (its first track, then sub-tracks) and root removes it.
#[test]
fn root_removes_a_group_of_several_tracks() {
	TestExternalities::default().execute_with(|| {
		assert_ok!(Tracks::<Runtime, Instance>::new_group_with_track(
			RuntimeOrigin::root(),
			origin(1),
			info()
		));
		let (group, _) = TracksIds::<Runtime, Instance>::get()
			.into_iter()
			.next()
			.expect("one track")
			.split();
		let n = 5u16;
		for sub in 1..n {
			assert_ok!(CommunityTracks::do_insert(
				<u32 as SplitId>::combine(group, sub),
				info(),
				origin(10 + sub as u8)
			));
		}
		assert_eq!(TracksIds::<Runtime, Instance>::get().len(), n as usize);

		assert_ok!(Tracks::<Runtime, Instance>::remove_group(
			RuntimeOrigin::root(),
			group,
			n.into()
		));
		assert!(TracksIds::<Runtime, Instance>::get().is_empty());
	})
}
