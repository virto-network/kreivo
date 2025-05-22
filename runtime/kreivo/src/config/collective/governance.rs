use super::*;

use frame_system::EnsureSigned;
use pallet_ranked_collective::{TallyOf, Votes};
use sp_core::ConstU128;

pub type KreivoReferendaInstance = pallet_referenda::Instance1;

impl pallet_referenda::Config<KreivoReferendaInstance> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_referenda::WeightInfo<Self>;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = EnsureSigned<AccountId>;
	type CancelOrigin = ReferendumCanceller;
	type KillOrigin = ReferendumKiller;
	type Slash = Treasury;
	type Votes = Votes;
	type Tally = TallyOf<Runtime, KreivoCollectiveInstance>;
	type SubmissionDeposit = ConstU128<{ UNITS }>;
	type MaxQueued = ConstU32<10>;
	type UndecidingTimeout = ConstU32<{ 2 * DAYS }>;
	type AlarmInterval = ConstU32<1>;
	type Tracks = tracks::TracksInfo;
	type Preimages = Preimage;
	type BlockNumberProvider = RelaychainData;
}
