use super::*;

use alloc::borrow::Cow;
use pallet_referenda::{Curve, Track};
use sp_runtime::{str_array as s, Perbill};

pub type TrackId = u16;

pub type CollectiveReferendaInstance = pallet_referenda::Instance1;
impl pallet_referenda::Config<CollectiveReferendaInstance> for Test {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_referenda::weights::SubstrateWeight<Self>;
	type Scheduler = Scheduler;
	type Currency = Balances;
	// Communities can submit proposals.
	type SubmitOrigin = AsEnsureOriginWithArg<pallet_ranked_collective::EnsureMember<Test, CollectiveInstance, 1>>;
	type CancelOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Slash = ();
	type Votes = pallet_ranked_collective::Votes;
	type Tally = pallet_ranked_collective::TallyOf<Test, CollectiveInstance>;
	type SubmissionDeposit = ConstU64<2>;
	type MaxQueued = ConstU32<3>;
	type UndecidingTimeout = ConstU64<20>;
	type AlarmInterval = AlarmInterval;
	type Tracks = Tracks;
	type Preimages = ();
	type BlockNumberProvider = System;
}

pub type CollectiveInstance = pallet_ranked_collective::Instance1;
impl pallet_ranked_collective::Config<CollectiveInstance> for Test {
	type WeightInfo = pallet_ranked_collective::weights::SubstrateWeight<Self>;
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureNever<()>;
	type RemoveOrigin = Self::DemoteOrigin;
	type PromoteOrigin = EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>;
	type DemoteOrigin = EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>;
	type ExchangeOrigin = EnsureNever<()>;
	type Polls = CollectiveReferenda;
	type MinRankOfClass = ();
	type MemberSwappedHandler = ();
	type VoteWeight = pallet_ranked_collective::Linear;
	type MaxMemberCount = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkSetup = ();
}
pub struct Tracks;
impl TracksInfo<Balance, BlockNumberFor<Test>> for Tracks {
	type Id = TrackId;
	type RuntimeOrigin = <RuntimeOrigin as OriginTrait>::PalletsOrigin;

	fn tracks() -> impl Iterator<Item = Cow<'static, Track<TrackId, Balance, BlockNumberFor<Test>>>> {
		const DATA: [Track<TrackId, Balance, BlockNumberFor<Test>>; 1] = [Track {
			id: 0,
			info: TrackInfo {
				name: s("Root"),
				max_deciding: 1,
				decision_deposit: 0,
				prepare_period: 1,
				decision_period: 4,
				confirm_period: 1,
				min_enactment_period: 1,
				min_approval: Curve::LinearDecreasing {
					length: Perbill::from_percent(100),
					floor: Perbill::from_percent(90),
					ceil: Perbill::from_percent(100),
				},
				min_support: Curve::LinearDecreasing {
					length: Perbill::from_percent(100),
					floor: Perbill::from_percent(0),
					ceil: Perbill::from_percent(100),
				},
			},
		}];
		DATA.iter().map(Cow::Borrowed)
	}

	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}
