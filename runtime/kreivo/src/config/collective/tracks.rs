use super::*;

use alloc::borrow::Cow;
use pallet_referenda::Track;
use sp_runtime::{str_array as s, FixedI64};

pub type TrackId = u16;

const fn percent(x: i32) -> FixedI64 {
	FixedI64::from_rational(x as u128, 100)
}

#[cfg(feature = "paseo")]
mod period {
	use super::*;

	pub const PREPARE: BlockNumber = 5 * MINUTES;
	pub const DECISION: BlockNumber = 4 * DAYS;
	pub const CONFIRM: BlockNumber = 15 * MINUTES;
}

#[cfg(not(feature = "paseo"))]
mod period {
	use super::*;

	pub const PREPARE: BlockNumber = 2 * MINUTES;
	pub const DECISION: BlockNumber = 1 * DAYS;
	pub const CONFIRM: BlockNumber = 2 * MINUTES;
}

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = TrackId;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;

	fn tracks() -> impl Iterator<Item = Cow<'static, Track<TrackId, Balance, BlockNumber>>> {
		const DATA: [Track<TrackId, Balance, BlockNumber>; 5] = [
			Track {
				id: 0,
				info: pallet_referenda::TrackInfo {
					name: s("Root"),
					max_deciding: 1,
					decision_deposit: 10 * UNITS,
					prepare_period: period::PREPARE,
					decision_period: period::DECISION,
					confirm_period: period::CONFIRM,
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
					prepare_period: period::PREPARE,
					decision_period: period::DECISION,
					confirm_period: period::CONFIRM,
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
					prepare_period: period::PREPARE,
					decision_period: period::DECISION,
					confirm_period: period::CONFIRM,
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
					prepare_period: period::PREPARE,
					decision_period: period::DECISION,
					confirm_period: period::CONFIRM,
					min_enactment_period: 1,
					min_approval: pallet_referenda::Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(50),
						ceil: Perbill::from_percent(100),
					},
					min_support: pallet_referenda::Curve::make_linear(28, 28, percent(50), percent(100)),
				},
			},
			Track {
				id: 4,
				info: pallet_referenda::TrackInfo {
					name: s("Black Hole Event Horizon"),
					max_deciding: 1,
					decision_deposit: UNITS,
					prepare_period: 5 * MINUTES,
					decision_period: 1 * DAYS,
					confirm_period: 5 * MINUTES,
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
		DATA.iter().map(Borrowed)
	}

	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else if let Ok(custom_origin) = pallet_custom_origins::Origin::try_from(id.clone()) {
			match custom_origin {
				Origin::ReferendumCanceller => Ok(1),
				Origin::ReferendumKiller => Ok(2),
				Origin::CreateMemberships => Ok(3),
				Origin::BlackHoleEventHorizon => Ok(4),
			}
		} else {
			Err(())
		}
	}
}
