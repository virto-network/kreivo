use super::*;

use frame_system::{EnsureNever, EnsureRootWithSuccess};
use pallet_ranked_collective::Rank;
use sp_core::ConstU16;
use sp_runtime::traits::Convert;

pub use pallet_custom_origins::*;
pub mod governance;
pub mod tracks;

pub struct AtLeastRank<const R: Rank>;
impl<const R: Rank, T> Convert<T, Rank> for AtLeastRank<R> {
	fn convert(_track: T) -> Rank {
		R
	}
}

pub type KreivoCollectiveInstance = pallet_ranked_collective::Instance1;
impl pallet_ranked_collective::Config<KreivoCollectiveInstance> for Runtime {
	type WeightInfo = pallet_ranked_collective::weights::SubstrateWeight<Self>;
	type RuntimeEvent = RuntimeEvent;

	type AddOrigin = EnsureNever<()>;

	type RemoveOrigin = Self::DemoteOrigin;

	// Initially, members of kreivo collective are promoted via governance action
	// In the future, it's expected to have an auxiliary pallet to observe the
	// criteria for ranking
	type PromoteOrigin = EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>;

	// Initially, members of kreivo collective are demoted via governance action
	// In the future, it's expected to have an auxiliary pallet to observe the
	// criteria for ranking
	type DemoteOrigin = EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>;

	type ExchangeOrigin = EnsureRoot<AccountId>;
	type Polls = KreivoReferenda;

	type MinRankOfClass = AtLeastRank<1>;
	type MemberSwappedHandler = ();
	type VoteWeight = pallet_ranked_collective::Linear;
	type MaxMemberCount = ();

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkSetup = CollectiveBenchmarkSetup;
}

#[cfg(feature = "runtime-benchmarks")]
use frame_support::traits::RankedMembers;
#[cfg(feature = "runtime-benchmarks")]
pub struct CollectiveBenchmarkSetup;
#[cfg(feature = "runtime-benchmarks")]
impl pallet_ranked_collective::BenchmarkSetup<AccountId> for CollectiveBenchmarkSetup {
	/// Ensure that this member is registered correctly.
	fn ensure_member(acc: &AccountId) {
		<pallet_ranked_collective::Pallet<Runtime, KreivoCollectiveInstance> as RankedMembers>::induct(acc)
			.expect("Account exists");
	}
}
