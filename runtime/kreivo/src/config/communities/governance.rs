use super::*;

use frame_system::{pallet_prelude::BlockNumberFor, EnsureRootWithSuccess};
use pallet_communities::RuntimeOriginFor;
use sp_std::marker::PhantomData;

use pallet_referenda::{BalanceOf, PalletsOriginOf, TrackIdOf, TracksInfo};

pub type CommunityTracksInstance = pallet_referenda_tracks::Instance2;
pub type CommunityReferendaInstance = pallet_referenda::Instance2;

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = 0;
	pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
}

pub struct EnsureOriginToTrack;
impl EnsureOriginWithArg<RuntimeOrigin, TrackIdOf<Runtime, CommunityTracksInstance>> for EnsureOriginToTrack {
	type Success = ();

	fn try_origin(
		o: RuntimeOrigin,
		id: &TrackIdOf<Runtime, CommunityTracksInstance>,
	) -> Result<Self::Success, RuntimeOrigin> {
		let track_id_for_origin: TrackIdOf<Runtime, CommunityTracksInstance> =
			CommunityTracks::track_for(&o.clone().caller).map_err(|_| o.clone())?;
		ensure!(&track_id_for_origin == id, o);

		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(id: &TrackIdOf<Runtime, CommunityTracksInstance>) -> Result<RuntimeOrigin, ()> {
		Ok(pallet_communities::Origin::<Runtime>::new(id.clone()).into())
	}
}

impl pallet_referenda_tracks::Config<CommunityTracksInstance> for Runtime {
	type AdminOrigin = EnsureRoot<AccountId>;
	type UpdateOrigin = EnsureOriginToTrack;
	type RuntimeEvent = RuntimeEvent;
	type TrackId = CommunityId;
	type MaxTracks = ConstU32<65536>;
	type WeightInfo = weights::pallet_referenda_tracks::WeightInfo<Self>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityTracksBenchmarkHelper;
}

pub struct EnsureCommunityMember<T, I: 'static = ()>(PhantomData<T>, PhantomData<I>);

impl<T, I> EnsureOriginWithArg<RuntimeOriginFor<T>, PalletsOriginOf<T>> for EnsureCommunityMember<T, I>
where
	T: pallet_communities::Config + pallet_referenda::Config<I>,
	T::Tracks: TracksInfo<
		BalanceOf<T, I>,
		BlockNumberFor<T>,
		RuntimeOrigin = PalletsOriginOf<T>,
		Id = <T as pallet_communities::Config>::CommunityId,
	>,
{
	type Success = T::AccountId;

	fn try_origin(
		o: RuntimeOriginFor<T>,
		track_origin: &PalletsOriginOf<T>,
	) -> Result<Self::Success, RuntimeOriginFor<T>> {
		use fc_traits_memberships::Inspect;
		use frame_system::RawOrigin::Signed;
		let community_id = T::Tracks::track_for(track_origin).map_err(|_| o.clone())?;

		match o.clone().into() {
			Ok(Signed(who)) => {
				if T::MemberMgmt::is_member_of(&community_id, &who) {
					Ok(who)
				} else {
					Err(o.clone())
				}
			}
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_track_origin: &PalletsOriginOf<T>) -> Result<RuntimeOriginFor<T>, ()> {
		todo!()
	}
}

// Paritally from https://github.com/polkadot-fellows/runtimes/blob/b5ba0e91d5dd3c4020e848b27be5f2b47e16f281/relay/kusama/src/governance/mod.rs#L75
impl pallet_referenda::Config<CommunityReferendaInstance> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_referenda::WeightInfo<Self>;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = EitherOf<
		EnsureRootWithSuccess<AccountId, TreasuryAccount>,
		EnsureCommunityMember<Self, CommunityReferendaInstance>,
	>;
	type CancelOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Slash = Treasury;
	type Votes = pallet_communities::types::VoteWeight;
	type Tally = pallet_communities::types::Tally<Runtime>;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = CommunityTracks;
	type Preimages = Preimage;
}

#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::SaturatedConversion;

#[cfg(feature = "runtime-benchmarks")]
pub struct CommunityTracksBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_referenda_tracks::BenchmarkHelper<Runtime, CommunityTracksInstance> for CommunityTracksBenchmarkHelper {
	fn track_id(id: u32) -> TrackIdOf<Runtime, CommunityTracksInstance> {
		id.saturated_into()
	}
}
