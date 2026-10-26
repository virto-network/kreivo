use super::*;

use frame_support::traits::TryMapSuccess;
use frame_system::{EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::origin::{EnsureCommunity, EnsureSignedPays};
use sp_runtime::{morph_types, traits::AccountIdConversion};
use virto_common::{CommunityId, MembershipId};

pub use memberships::{Memberships, MembershipsManagerAccount, RetiringMembershipsAccount};

pub mod governance;
pub mod memberships;

/// The entries a registered community writes, and their bytes: the key and value of each at their
/// largest encoding (`MaxEncodedLen`). `tests::deposit_values::community_footprint` measures them.
///
/// 13 entries: `System::Account`, `Communities::{CommunityIdFor, Info}`,
/// `CommunityMemberships::{Collection, CollectionRoleOf, CollectionConfigOf, CollectionAccount}`,
/// `CommunityTracks::{Tracks, OriginToTrackId}` and `KreivoCollective::{Members, MemberCount,
/// IdToIndex, IndexToId}`. 1 746 bytes: their keys and values, and the 2 bytes the community adds
/// to the shared `CommunityTracks::TracksIds`.
pub const COMMUNITY_ITEMS: u32 = 13;
pub const COMMUNITY_BYTES: u32 = 1_746;

/// The storage part: the footprint priced at Kreivo's deposit rate.
///
/// Paseo's unit is a hundredth of Kusama's but its deposit rate is not scaled with it, so there
/// the footprint costs more than the whole registration payment: that testnet keeps a tenth of it.
const fn storage_part() -> Balance {
	if cfg!(feature = "paseo") {
		UNITS / 10
	} else {
		deposit(COMMUNITY_ITEMS, COMMUNITY_BYTES)
	}
}

type CreationPayment = Option<(Balance, AccountId, AccountId)>;

parameter_types! {
	pub const CommunityPalletId: PalletId = PalletId(*b"kv/cmtys");
	pub const MembershipsCollectionId: CommunityId = 0;
	pub const MembershipNftAttr: &'static [u8; 10] = b"membership";
	pub const CommunityDepositAmount: Balance = UNITS / 2;
	pub const NoPay: CreationPayment = None;
	/// What each issued membership holds on its community's account (`DEC-19`): the existential
	/// deposit, provisional until the economic study (0007-A1; `deployment/kreivo-0.18.md` §1).
	///
	/// A membership's measured footprint is 6 entries and 1 286 bytes, which `deposit(6, 1 286)`
	/// prices at 40 428 666 198 planck (about 0.0404 KSM): well above this value. It is the owner's ruling
	/// to bind the existential deposit until the study, so the footprint is recorded, not bound
	/// (`tests::deposit_values`).
	pub const MembershipDepositAmount: Balance = EXISTENTIAL_DEPOSIT;
	/// How long a retiring membership waits: 10 days of relay chain blocks, 144 000
	/// (m3-retirement-delay). At least Kusama's unbonding period, 7 days = 100 800, plus 3 days for
	/// the XCM round trip. Both this and the deposit change only by runtime upgrade.
	pub const RetirementDelayInBlocks: BlockNumber = 10 * DAYS;
	/// The most memberships a call issues or retires. Lowered from 1 024, which `issue_memberships`
	/// would weigh at about 0.8 s of `ref_time` and 3.4 MB of proof (measured weights, #515) against
	/// a normal-class block of 0.5 s and 2.5 MiB: it could never be included.
	///
	/// Sized from the measured weights at Kreivo's `DbWeight` (25 µs a read, 100 µs a write), against
	/// half a normal-class block (0.25 s, 1.25 MiB). Per membership, `retire_memberships` is the
	/// larger: 80.97 µs + 4 reads + 7 writes = 880.97 µs of `ref_time`, and 3 334 B of proof;
	/// `issue_memberships` takes 93.33 µs + 4 reads + 6 writes = 793.33 µs, and the same proof.
	/// At 256, retiring weighs 0.226 s (90 % of the half block) and 0.857 MB of proof (65 %);
	/// issuing weighs 0.204 s (82 %) and 0.857 MB. 320, the next multiple of 64, would take 113 %
	/// of the `ref_time`, and 512 would take 180 %. `tests::deposit_values` checks both bounds.
	pub const MaxIssuePerCall: u32 = 256;
	/// The part of a registration that the community's account holds (0010-Q2, `DEP-21`): a
	/// community's measured storage footprint at Kreivo's deposit rate, `deposit(13, 1 746)`
	/// = 13 × 6 666 666 660 + 1 746 × 333 333 = 87 248 665 998 planck, about 0.0872 KSM.
	pub const CommunityStoragePartAmount: Balance = storage_part();
	/// The rest of the registration payment, paid to the Treasury: 0.5 KSM less the storage part,
	/// 412 751 334 002 planck, about 0.4128 KSM (0010-Q1: two amounts whose sum is the payment).
	pub const CommunityConsumptionPartAmount: Balance = UNITS / 2 - storage_part();
}

morph_types! {
	pub type AccountToCommunityId: TryMorph = |a: AccountId| -> Result<CommunityId, ()> {
		PalletId::try_from_sub_account(&a).map(|(_, id)| id).ok_or(())
	};
}
pub(crate) type EnsureCommunityAccount = TryMapSuccess<EnsureSigned<AccountId>, AccountToCommunityId>;

type RootCreatesCommunitiesForFree = EnsureRootWithSuccess<AccountId, NoPay>;
type AnyoneElsePays = EnsureSignedPays<Runtime, CommunityDepositAmount, TreasuryAccount>;

impl pallet_communities::Config for Runtime {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = weights::pallet_communities::WeightInfo<Runtime>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type CreateOrigin = frame_system::EnsureNever<CreationPayment>;

	#[cfg(feature = "runtime-benchmarks")]
	type CreateOrigin = RootCreatesCommunitiesForFree;

	type AdminOrigin = EitherOf<EnsureCommunity<Self>, EnsureCommunityAccount>;
	type MemberMgmtOrigin = EitherOf<EnsureCommunity<Self>, EnsureCommunityAccount>;
	// A member acts on a membership they hold (`DEC-31`).
	type MemberOrigin = EnsureSigned<AccountId>;
	type CommunityId = CommunityId;

	type MembershipId = MembershipId;
	type MemberMgmt = Memberships;
	type Polls = CommunityReferenda;
	type CommunityOfClass = governance::CommunityOfTrack;
	type Assets = Assets;

	type AssetsFreezer = AssetsFreezer;

	type Balances = Balances;
	type BlockNumberProvider = RelaychainData;

	type PalletId = CommunityPalletId;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

impl pallet_communities_manager::Config for Runtime {
	type CreateCollection = CommunityMemberships;
	type Tracks = governance::CommunityTracksWrapper;
	type RankedCollective = KreivoCollective;
	type WeightInfo = weights::pallet_communities_manager::WeightInfo<Self>;
	type RegisterOrigin = EitherOf<RootCreatesCommunitiesForFree, AnyoneElsePays>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type GroupAdminOrigin = EitherOf<EnsureCommunity<Runtime>, EnsureCommunityAccount>;
	type Issuer = Memberships;
	type ItemAttributes =
		pallet_communities_manager::ClearNftsAttributes<Runtime, memberships::CommunityMembershipsInstance>;
	type MembershipsManagerAccount = MembershipsManagerAccount;
	type CommunityStoragePart = CommunityStoragePartAmount;
	type CommunityConsumptionPart = CommunityConsumptionPartAmount;
	type RetirementHolder = RetiringMembershipsAccount;
	type MembershipDeposit = MembershipDepositAmount;
	type RetirementDelay = RetirementDelayInBlocks;
	type MaxIssuePerCall = MaxIssuePerCall;
}

#[cfg(feature = "runtime-benchmarks")]
pub use benchmarks::CommunityBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;

	use alloc::boxed::Box;
	use governance::{CommunityReferendaInstance, CommunityTracksInstance};

	use frame_benchmarking::BenchmarkError;
	use frame_contrib_traits::memberships::Issue;
	use frame_support::traits::{nonfungibles_v2::Create, schedule::DispatchTime};
	use frame_system::pallet_prelude::{OriginFor, RuntimeCallFor};
	use pallet_communities::{
		types::{CommunityIdOf, MembershipIdOf, PalletsOriginOf, PollIndexOf},
		BenchmarkHelper,
	};
	use pallet_referenda::{BoundedCallOf, Curve, Pallet as Referenda, TrackInfo};
	use pallet_referenda_tracks::Pallet as Tracks;
	use parity_scale_codec::Encode;
	use sp_runtime::Perbill;

	/// Moves time to block `n`. Referenda (and the scheduler) count relay chain blocks, so the
	/// relay chain block number moves along with the parachain's.
	fn advance_to(n: BlockNumber) {
		System::set_block_number(n);
		<RelaychainData as sp_runtime::traits::BlockNumberProvider>::set_block_number(n);
	}

	pub struct CommunityBenchmarkHelper;

	impl BenchmarkHelper<Runtime> for CommunityBenchmarkHelper {
		fn community_id() -> CommunityIdOf<Runtime> {
			1
		}
		fn community_asset_id() -> AssetIdOf<Runtime> {
			1u32.into()
		}
		fn community_desired_size() -> u32 {
			u8::MAX.into()
		}

		fn initialize_memberships_collection() -> Result<(), BenchmarkError> {
			let community_id = Self::community_id();
			let community_account = pallet_communities::Pallet::<Runtime>::community_account(&community_id);

			CommunityMemberships::create_collection_with_id(
				MembershipsCollectionId::get(),
				&community_account,
				&community_account,
				&Default::default(),
			)?;

			// A community's collection is owned and administered by the manager account (`DEC-36`).
			let manager = MembershipsManagerAccount::get();
			CommunityMemberships::create_collection_with_id(
				Self::community_id(),
				&manager,
				&manager,
				&Default::default(),
			)?;

			Ok(())
		}

		fn issue_membership(
			community_id: CommunityIdOf<Runtime>,
			membership_id: MembershipIdOf<Runtime>,
		) -> Result<(), BenchmarkError> {
			<Memberships as Issue<AccountId>>::issue(&community_id, &membership_id)?;

			Ok(())
		}

		fn prepare_track(pallet_origin: PalletsOriginOf<Runtime>) -> Result<(), BenchmarkError> {
			let id = Self::community_id();
			let info = TrackInfo {
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
			};

			// The community's first track, as `pallet_communities_manager` creates it.
			Tracks::<Runtime, CommunityTracksInstance>::do_insert(
				<governance::CommunityTrackId as pallet_referenda_tracks::SplitId>::combine(id, 0),
				info,
				pallet_origin,
			)?;

			Ok(())
		}

		fn prepare_poll(
			origin: OriginFor<Runtime>,
			proposal_origin: PalletsOriginOf<Runtime>,
			proposal_call: RuntimeCallFor<Runtime>,
		) -> Result<PollIndexOf<Runtime>, BenchmarkError> {
			let bounded_call = BoundedVec::truncate_from(proposal_call.encode());
			let proposal_origin = Box::new(proposal_origin);
			let proposal = BoundedCallOf::<Runtime, CommunityReferendaInstance>::Inline(bounded_call);
			let enactment_moment = DispatchTime::After(1);

			let index = 0u32;
			Referenda::<Runtime, CommunityReferendaInstance>::submit(
				origin.clone(),
				proposal_origin,
				proposal,
				enactment_moment,
			)?;
			Referenda::<Runtime, CommunityReferendaInstance>::place_decision_deposit(origin, index)?;

			advance_to(2);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), 0)?;

			Ok(0)
		}

		fn finish_poll(index: PollIndexOf<Runtime>) -> Result<(), BenchmarkError> {
			advance_to(8);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), index)?;

			frame_support::assert_ok!(Referenda::<Runtime, CommunityReferendaInstance>::ensure_ongoing(index));

			advance_to(9);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), index)?;

			frame_support::assert_err!(
				Referenda::<Runtime, CommunityReferendaInstance>::ensure_ongoing(index),
				pallet_referenda::Error::<Runtime, CommunityReferendaInstance>::NotOngoing
			);

			Ok(())
		}
	}
}
