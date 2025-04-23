use super::*;

use alloc::borrow::Cow;
use pallet_referenda::Track;
use sp_runtime::{str_array as s, FixedI64};

pub type TrackId = u16;

const fn percent(x: i32) -> FixedI64 {
	FixedI64::from_rational(x as u128, 100)
}

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = TrackId;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;

	fn tracks() -> impl Iterator<Item = Cow<'static, Track<TrackId, Balance, BlockNumber>>> {
		const DATA: [Track<TrackId, Balance, BlockNumber>; 4] = [
			Track {
				id: 0,
				info: pallet_referenda::TrackInfo {
					name: s("Root"),
					max_deciding: 1,
					decision_deposit: 10 * UNITS,
					prepare_period: 15 * MINUTES,
					decision_period: 4 * DAYS,
					confirm_period: 15 * MINUTES,
					min_enactment_period: 1,
					min_approval: pallet_referenda::Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(90),
						ceil: Perbill::from_percent(100),
					},
					min_support: pallet_referenda::Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(0),
						ceil: Perbill::from_percent(100),
					},
				},
			},
			Track {
				id: 1,
				info: pallet_referenda::TrackInfo {
					name: s("Referendum Canceller"),
					max_deciding: 1,
					decision_deposit: UNITS,
					prepare_period: 15 * MINUTES,
					decision_period: 4 * DAYS,
					confirm_period: 15 * MINUTES,
					min_enactment_period: 1,
					min_approval: pallet_referenda::Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(90),
						ceil: Perbill::from_percent(100),
					},
					min_support: pallet_referenda::Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(0),
						ceil: Perbill::from_percent(100),
					},
				},
			},
			Track {
				id: 2,
				info: pallet_referenda::TrackInfo {
					name: s("Referendum Killer"),
					max_deciding: 1,
					decision_deposit: UNITS,
					prepare_period: 15 * MINUTES,
					decision_period: 4 * DAYS,
					confirm_period: 15 * MINUTES,
					min_enactment_period: 1,
					min_approval: pallet_referenda::Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(90),
						ceil: Perbill::from_percent(100),
					},
					min_support: pallet_referenda::Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(0),
						ceil: Perbill::from_percent(100),
					},
				},
			},
			Track {
				id: 3,
				info: pallet_referenda::TrackInfo {
					name: s("Create Memberships"),
					max_deciding: 1,
					decision_deposit: UNITS,
					prepare_period: 15 * MINUTES,
					decision_period: 4 * DAYS,
					confirm_period: 15 * MINUTES,
					min_enactment_period: 1,
					min_approval: pallet_referenda::Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(50),
						ceil: Perbill::from_percent(100),
					},
					min_support: pallet_referenda::Curve::make_linear(28, 28, percent(50), percent(100)),
				},
			},
		];
		DATA.iter().map(Cow::Borrowed)
	}

	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else if let Ok(custom_origin) = pallet_custom_origins::Origin::try_from(id.clone()) {
			match custom_origin {
				pallet_custom_origins::Origin::ReferendumCanceller => Ok(1),
				pallet_custom_origins::Origin::ReferendumKiller => Ok(2),
				pallet_custom_origins::Origin::CreateMemberships => Ok(3),
			}
		} else {
			Err(())
		}
	}
}
