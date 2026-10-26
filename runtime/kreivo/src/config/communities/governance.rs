use super::*;

use core::marker::PhantomData;
use frame_support::{dispatch::DispatchResult, traits::AsEnsureOriginWithArg};
#[cfg(not(feature = "runtime-benchmarks"))]
use frame_system::EnsureNever;
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRootWithSuccess};
use pallet_communities::RuntimeOriginFor;
use sp_runtime::traits::Convert;

use pallet_referenda::{BalanceOf, PalletsOriginOf, TrackIdOf, TracksInfo};
use pallet_referenda_tracks::SplitId;

pub type CommunityTracksInstance = pallet_referenda_tracks::Instance2;
pub type CommunityReferendaInstance = pallet_referenda::Instance2;

/// The id of a community track: `combine(community_id: u16, sub: u16)`, see
/// [`pallet_referenda_tracks::SplitId`]. A community's first track is `combine(community_id, 0)`.
pub type CommunityTrackId = u32;

/// The community that a track (a poll class) belongs to: the group half of its id.
pub struct CommunityOfTrack;
impl Convert<CommunityTrackId, Option<CommunityId>> for CommunityOfTrack {
	fn convert(track: CommunityTrackId) -> Option<CommunityId> {
		Some(track.split().0)
	}
}

/// Creates the governance track of a community, as `combine(community_id, 0)`.
pub struct CommunityTracksWrapper;
impl pallet_communities_manager::MutateTracks<Balance, BlockNumber> for CommunityTracksWrapper {
	type Id = CommunityId;
	type RuntimeOrigin = PalletsOriginOf<Runtime>;

	fn insert(
		id: Self::Id,
		info: pallet_referenda::TrackInfo<Balance, BlockNumber>,
		origin: Self::RuntimeOrigin,
	) -> DispatchResult {
		CommunityTracks::do_insert(CommunityTrackId::combine(id, 0), info, origin)
	}
}

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
		Ok(pallet_communities::Origin::<Runtime>::new(id.split().0).into())
	}
}

/// Root creates groups (and sub-tracks) on the first group, as frame-contrib's kitchensink does.
/// Only in the benchmark build: it is what the benchmarks of `pallet_referenda_tracks` call with.
#[cfg(feature = "runtime-benchmarks")]
pub struct EnsureRootReturnsFirstGroup;
#[cfg(feature = "runtime-benchmarks")]
impl EnsureOriginWithArg<RuntimeOrigin, PalletsOriginOf<Runtime>> for EnsureRootReturnsFirstGroup {
	type Success = <CommunityTrackId as SplitId>::Half;

	fn try_origin(o: RuntimeOrigin, _: &PalletsOriginOf<Runtime>) -> Result<Self::Success, RuntimeOrigin> {
		<EnsureRoot<AccountId> as frame_support::traits::EnsureOrigin<RuntimeOrigin>>::try_origin(o)
			.map(|_| Default::default())
	}

	fn try_successful_origin(_: &PalletsOriginOf<Runtime>) -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::root())
	}
}

impl pallet_referenda_tracks::Config<CommunityTracksInstance> for Runtime {
	// Community tracks are created by `pallet_communities_manager` (`CommunityTracksWrapper`), never by
	// a call, and no call adds sub-tracks to a community yet.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type GroupManagerCreateOrigin = AsEnsureOriginWithArg<EnsureNever<<CommunityTrackId as SplitId>::Half>>;
	// The benchmarks of `pallet_referenda_tracks` create groups and sub-tracks as root.
	#[cfg(feature = "runtime-benchmarks")]
	type CreateOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
	#[cfg(feature = "runtime-benchmarks")]
	type GroupManagerCreateOrigin = EnsureRootReturnsFirstGroup;
	// A community configures its own track (deposit, periods, curves, deciding limit), as root does
	// (the former `UpdateOrigin`). It cannot remove it: only root does.
	type GroupManagerOrigin = EitherOf<AsEnsureOriginWithArg<EnsureRoot<AccountId>>, EnsureOriginToTrack>;
	type RemoveOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
	type RemoveGroupOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
	type TrackId = CommunityTrackId;
	type MaxTracks = ConstU32<65536>;
	type WeightInfo = weights::pallet_referenda_tracks::WeightInfo<Self>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityTracksBenchmarkHelper;
}

pub struct EnsureCommunityMember<T, I: 'static = ()>(PhantomData<T>, PhantomData<I>);

impl<T, I> EnsureOriginWithArg<RuntimeOriginFor<T>, PalletsOriginOf<T>> for EnsureCommunityMember<T, I>
where
	T: pallet_communities::Config<CommunityId = CommunityId> + pallet_referenda::Config<I>,
	T::Tracks:
		TracksInfo<BalanceOf<T, I>, BlockNumberFor<T>, RuntimeOrigin = PalletsOriginOf<T>, Id = CommunityTrackId>,
{
	type Success = T::AccountId;

	fn try_origin(
		o: RuntimeOriginFor<T>,
		track_origin: &PalletsOriginOf<T>,
	) -> Result<Self::Success, RuntimeOriginFor<T>> {
		use frame_contrib_traits::memberships::Inspect;
		use frame_system::RawOrigin::Signed;
		let community_id = T::Tracks::track_for(track_origin).map_err(|_| o.clone())?.split().0;

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
	type BlockNumberProvider = RelaychainData;
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
