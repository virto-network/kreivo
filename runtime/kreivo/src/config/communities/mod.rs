use super::*;

use frame_support::traits::TryMapSuccess;
#[cfg(not(feature = "runtime-benchmarks"))]
use frame_system::EnsureNever;
use frame_system::{EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::origin::{EnsureCommunity, EnsureSignedPays};
use sp_runtime::{morph_types, traits::AccountIdConversion};
use virto_common::{CommunityId, MembershipId};

use fc_traits_memberships::{NonFungiblesMemberships, WithHooks};
pub mod governance;
mod kreivo_memberships;
pub mod memberships;

#[cfg(feature = "runtime-benchmarks")]
use self::{
	governance::{CommunityReferendaInstance, CommunityTracksInstance},
	memberships::CommunityMembershipsInstance,
};
use pallet_custom_origins::CreateMemberships;

#[cfg(feature = "runtime-benchmarks")]
use {
	frame_benchmarking::BenchmarkError,
	frame_support::traits::{schedule::DispatchTime, tokens::nonfungible_v2::Mutate},
	frame_system::pallet_prelude::{OriginFor, RuntimeCallFor},
	pallet_communities::{
		types::{CommunityIdOf, MembershipIdOf, PalletsOriginOf, PollIndexOf},
		BenchmarkHelper,
	},
	pallet_nfts::Pallet as Nfts,
	pallet_referenda::{BoundedCallOf, Curve, Pallet as Referenda, TrackInfo},
	pallet_referenda_tracks::Pallet as Tracks,
	sp_core::Encode,
	sp_runtime::Perbill,
};

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
	#[cfg(not(feature = "runtime-benchmarks"))]
	type CreateOrigin = EnsureNever<CreationPayment>;
	#[cfg(feature = "runtime-benchmarks")]
	type CreateOrigin = RootCreatesCommunitiesForFree;
	type AdminOrigin = EitherOf<EnsureCommunity<Self>, EnsureCommunityAccount>;
	type MemberMgmtOrigin = EitherOf<EnsureCommunity<Self>, EnsureCommunityAccount>;
	type MemberMgmt =
		WithHooks<NonFungiblesMemberships<CommunityMemberships>, kreivo_memberships::CopySystemAttributesOnAssign>;
	type MembershipId = MembershipId;

	type Polls = CommunityReferenda;

	type Assets = Assets;
	type AssetsFreezer = AssetsFreezer;
	type Balances = Balances;

	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = crate::weights::pallet_communities::WeightInfo<Runtime>;

	type PalletId = CommunityPalletId;

	type ItemConfig = pallet_nfts::ItemConfig;
	type RuntimeFreezeReason = RuntimeFreezeReason;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

impl pallet_communities_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CreateCollection = CommunityMemberships;
	type Tracks = CommunityTracks;
	type RankedCollective = KreivoCollective;
	type RegisterOrigin = EitherOf<RootCreatesCommunitiesForFree, AnyoneElsePays>;

	type CreateMembershipsOrigin = EitherOf<EnsureRoot<AccountId>, CreateMemberships>;
	type MembershipId = MembershipId;
	type MembershipsManagerOwner = TreasuryAccount;
	type MembershipsManagerCollectionId = MembershipsCollectionId;
	type CreateMemberships = CommunityMemberships;
	type MakeTank = MembershipsGasTank;

	type WeightInfo = crate::weights::pallet_communities_manager::WeightInfo<Self>;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct CommunityBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
type MembershipsManagementCollection =
	frame_support::traits::nonfungible_v2::ItemOf<CommunityMemberships, MembershipsCollectionId, AccountId>;

#[cfg(feature = "runtime-benchmarks")]
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
		let collection = MembershipsCollectionId::get();
		Nfts::<Runtime, CommunityMembershipsInstance>::do_create_collection(
			collection,
			TreasuryAccount::get(),
			TreasuryAccount::get(),
			Default::default(),
			0,
			pallet_nfts::Event::ForceCreated {
				collection,
				owner: TreasuryAccount::get(),
			},
		)?;

		let community_id = Self::community_id();
		let community_account = pallet_communities::Pallet::<Runtime>::community_account(&community_id);

		Nfts::<Runtime, CommunityMembershipsInstance>::do_create_collection(
			community_id,
			community_account.clone(),
			community_account.clone(),
			Default::default(),
			0,
			pallet_nfts::Event::ForceCreated {
				collection: community_id,
				owner: community_account,
			},
		)?;

		Ok(())
	}

	fn issue_membership(
		community_id: CommunityIdOf<Runtime>,
		membership_id: MembershipIdOf<Runtime>,
	) -> Result<(), BenchmarkError> {
		let community_account = pallet_communities::Pallet::<Runtime>::community_account(&community_id);

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
