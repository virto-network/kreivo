use super::*;

use frame_support::traits::TryMapSuccess;
use frame_system::{EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::origin::{EnsureCommunity, EnsureSignedPays};
use sp_runtime::{morph_types, traits::AccountIdConversion};
use virto_common::{CommunityId, MembershipId};

use frame_contrib_traits::memberships::{NonFungiblesMemberships, WithHooks};
pub mod governance;
pub mod memberships;

use pallet_custom_origins::CreateMemberships;

type CreationPayment = Option<(Balance, AccountId, AccountId)>;

parameter_types! {
	pub const CommunityPalletId: PalletId = PalletId(*b"kv/cmtys");
	pub const MembershipsCollectionId: CommunityId = 0;
	pub const MembershipNftAttr: &'static [u8; 10] = b"membership";
	pub const CommunityDepositAmount: Balance = UNITS / 2;
	pub const NoPay: CreationPayment = None;
}

morph_types! {
	pub type AccountToCommunityId: TryMorph = |a: AccountId| -> Result<CommunityId, ()> {
		PalletId::try_from_sub_account(&a).map(|(_, id)| id).ok_or(())
	};
}
type EnsureCommunityAccount = TryMapSuccess<EnsureSigned<AccountId>, AccountToCommunityId>;

type RootCreatesCommunitiesForFree = EnsureRootWithSuccess<AccountId, NoPay>;
type AnyoneElsePays = EnsureSignedPays<Runtime, CommunityDepositAmount, TreasuryAccount>;

impl pallet_communities::Config for Runtime {
	type CommunityId = CommunityId;
	type MembershipId = MembershipId;
	type ItemConfig = pallet_nfts::ItemConfig;
	type MemberMgmt =
		WithHooks<NonFungiblesMemberships<CommunityMemberships>, memberships::CopySystemAttributesOnAssign>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type CreateOrigin = frame_system::EnsureNever<CreationPayment>;
	#[cfg(feature = "runtime-benchmarks")]
	type CreateOrigin = RootCreatesCommunitiesForFree;
	type AdminOrigin = EitherOf<EnsureCommunity<Self>, EnsureCommunityAccount>;

	type MemberMgmtOrigin = EitherOf<EnsureCommunity<Self>, EnsureCommunityAccount>;

	type Polls = CommunityReferenda;
	type Assets = Assets;
	type AssetsFreezer = AssetsFreezer;

	type Balances = Balances;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeFreezeReason = RuntimeFreezeReason;

	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = weights::pallet_communities::WeightInfo<Runtime>;
	type PalletId = CommunityPalletId;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

impl pallet_communities_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CreateCollection = CommunityMemberships;
	type MakeTank = MembershipsGasTank;
	type Tracks = CommunityTracks;
	type RankedCollective = KreivoCollective;

	type WeightInfo = weights::pallet_communities_manager::WeightInfo<Self>;
	type RegisterOrigin = EitherOf<RootCreatesCommunitiesForFree, AnyoneElsePays>;
	type CreateMembershipsOrigin = EitherOf<EnsureRoot<AccountId>, CreateMemberships>;
	type MembershipId = MembershipId;
	type MembershipsManagerCollectionId = MembershipsCollectionId;
	type MembershipsManagerOwner = TreasuryAccount;

	type CreateMemberships = CommunityMemberships;
}

#[cfg(feature = "runtime-benchmarks")]
pub use benchmarks::CommunityBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;

	use governance::{CommunityReferendaInstance, CommunityTracksInstance};

	use frame_benchmarking::BenchmarkError;
	use frame_support::traits::{
		nonfungible_v2::{ItemOf, Mutate},
		nonfungibles_v2::Create,
		schedule::DispatchTime,
	};
	use frame_system::pallet_prelude::{OriginFor, RuntimeCallFor};
	use pallet_communities::{
		types::{CommunityIdOf, MembershipIdOf, PalletsOriginOf, PollIndexOf},
		BenchmarkHelper,
	};
	use pallet_referenda::{BoundedCallOf, Curve, Pallet as Referenda, TrackInfo};
	use pallet_referenda_tracks::Pallet as Tracks;
	use parity_scale_codec::Encode;
	use sp_runtime::Perbill;

	type MembershipsManagementCollection = ItemOf<CommunityMemberships, MembershipsCollectionId, AccountId>;

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

			CommunityMemberships::create_collection_with_id(
				Self::community_id(),
				&community_account,
				&community_account,
				&Default::default(),
			)?;

			Ok(())
		}

		fn issue_membership(
			community_id: CommunityIdOf<Runtime>,
			membership_id: MembershipIdOf<Runtime>,
		) -> Result<(), BenchmarkError> {
			let community_account = Communities::community_account(&community_id);

			MembershipsManagementCollection::mint_into(&membership_id, &community_account, &Default::default(), true)?;

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

			Tracks::<Runtime, CommunityTracksInstance>::insert(RuntimeOrigin::root(), id, info, pallet_origin)?;

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

			System::set_block_number(2);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), 0)?;

			Ok(0)
		}

		fn finish_poll(index: PollIndexOf<Runtime>) -> Result<(), BenchmarkError> {
			System::set_block_number(8);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), index)?;

			frame_support::assert_ok!(Referenda::<Runtime, CommunityReferendaInstance>::ensure_ongoing(index));

			System::set_block_number(9);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), index)?;

			frame_support::assert_err!(
				Referenda::<Runtime, CommunityReferendaInstance>::ensure_ongoing(index),
				pallet_referenda::Error::<Runtime, CommunityReferendaInstance>::NotOngoing
			);

			Ok(())
		}
	}
}
