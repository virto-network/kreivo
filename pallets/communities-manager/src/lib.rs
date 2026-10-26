#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use frame_contrib_traits::memberships::Issue;
use frame_support::{
	pallet_prelude::*,
	traits::{
		fungible::{hold::Mutate as MutateHold, Mutate},
		nonfungibles_v2::Create as CollectionCreate,
		tokens::{Precision, Preservation},
		Incrementable, OriginTrait, RankedMembers,
	},
};
use frame_system::{
	ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
use pallet_communities::{
	AccountIdOf, AssetIdOf, CommunityIdOf, DecisionMethodFor, NativeBalanceOf, Origin as CommunityOrigin,
	PalletsOriginOf, RuntimeOriginFor,
};
use pallet_nfts::{AttributeNamespace, CollectionConfig};
use pallet_referenda::TrackInfo;
use sp_runtime::{
	str_array,
	traits::{BlockNumberProvider, Saturating, StaticLookup, Zero},
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(all(feature = "runtime-benchmarks", feature = "std"))]
pub use benchmarking::test_group_benchmark;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

pub mod weights;
pub use weights::*;

/// What the pallet needs of the tracks pallet: to create the governance track of a community.
///
/// `Id` is the community's id, not the track's: the implementation decides which track (or tracks)
/// belong to the community, e.g. `combine(community_id, 0)` over a split track id.
pub trait MutateTracks<Balance, Moment> {
	/// The id of the community that the track belongs to.
	type Id;
	/// The origin that the track is associated with.
	type RuntimeOrigin;

	/// Inserts the first track of the community `id`, associated with `origin`.
	fn insert(id: Self::Id, info: TrackInfo<Balance, Moment>, origin: Self::RuntimeOrigin) -> DispatchResult;
}

type TrackInfoOf<T> = TrackInfo<NativeBalanceOf<T>, BlockNumberFor<T>>;

/// A moment of the clock communities (and so retirements) are timed with.
pub type MomentOf<T> = <<T as pallet_communities::Config>::BlockNumberProvider as BlockNumberProvider>::BlockNumber;

/// Clears the attributes of a membership item, in every namespace of its collection system.
///
/// Whoever sets an attribute on an item (its holder, a delegate, the collection's team) writes it in
/// a namespace the memberships manager does not own, and a burn leaves it behind. A retirement
/// clears them all, with the root origin of the collection system.
pub trait ClearItemAttributes<CollectionId, ItemId> {
	/// How many attributes the item carries, in every namespace.
	fn count(collection: &CollectionId, item: &ItemId) -> u32;

	/// Clears every attribute of the item, whatever its namespace.
	fn clear_all(collection: &CollectionId, item: &ItemId) -> DispatchResult;

	/// Gives the item `amount` attributes, for the benchmark of the clearing.
	#[cfg(feature = "runtime-benchmarks")]
	fn set_benchmark_attributes(collection: &CollectionId, item: &ItemId, amount: u32);
}

/// Clears the attributes of a `pallet_nfts` item through its force origin, which must accept root.
pub struct ClearNftsAttributes<T, I = ()>(core::marker::PhantomData<(T, I)>);

impl<T: pallet_nfts::Config<I>, I: 'static> ClearItemAttributes<T::CollectionId, T::ItemId>
	for ClearNftsAttributes<T, I>
{
	fn count(collection: &T::CollectionId, item: &T::ItemId) -> u32 {
		pallet_nfts::Attribute::<T, I>::iter_prefix((collection, Some(item))).count() as u32
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn set_benchmark_attributes(collection: &T::CollectionId, item: &T::ItemId, amount: u32) {
		for i in 0..amount {
			let key = i.to_le_bytes().to_vec();
			pallet_nfts::Pallet::<T, I>::force_set_attribute(
				frame_system::RawOrigin::Root.into(),
				None,
				*collection,
				Some(*item),
				AttributeNamespace::Pallet,
				key.try_into().expect("the key limit holds a u32"),
				Default::default(),
			)
			.expect("the benchmark sets its attributes");
		}
	}

	fn clear_all(collection: &T::CollectionId, item: &T::ItemId) -> DispatchResult {
		let attributes: Vec<_> = pallet_nfts::Attribute::<T, I>::iter_prefix((collection, Some(item)))
			.map(|((namespace, key), _)| (namespace, key))
			.collect();

		for (namespace, key) in attributes {
			pallet_nfts::Pallet::<T, I>::clear_attribute(
				frame_system::RawOrigin::Root.into(),
				*collection,
				Some(*item),
				namespace as AttributeNamespace<T::AccountId>,
				key,
			)?;
		}
		Ok(())
	}
}

// The pallet macros clone every call argument, `Copy` ones included.
#[allow(clippy::clone_on_copy)]
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::DefaultNoBound;
	use parity_scale_codec::HasCompact;

	type CommunityName = BoundedVec<u8, ConstU32<25>>;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config<RuntimeEvent: From<Event<Self>>>
		+ pallet_communities::Config<
			MembershipId: Incrementable + HasCompact + MaybeSerializeDeserialize + Default,
			Balances: MutateHold<
				<Self as frame_system::Config>::AccountId,
				Reason = <Self as Config>::RuntimeHoldReason,
			>,
		>
	where
		AssetIdOf<Self>: MaybeSerializeDeserialize,
	{
		/// The overarching hold reason.
		type RuntimeHoldReason: From<HoldReason>;

		type CreateCollection: CollectionCreate<
			AccountIdOf<Self>,
			CollectionConfig<NativeBalanceOf<Self>, BlockNumberFor<Self>, CommunityIdOf<Self>>,
			CollectionId = CommunityIdOf<Self>,
		>;

		/// Creates the governance track of a community.
		type Tracks: MutateTracks<
			NativeBalanceOf<Self>,
			BlockNumberFor<Self>,
			Id = CommunityIdOf<Self>,
			RuntimeOrigin = PalletsOriginOf<Self>,
		>;

		type RankedCollective: RankedMembers<AccountId = AccountIdOf<Self>>;

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		type RegisterOrigin: EnsureOrigin<
			OriginFor<Self>,
			Success = Option<(NativeBalanceOf<Self>, AccountIdOf<Self>, AccountIdOf<Self>)>,
		>;

		/// The origin of a community's administration. It resolves to the community.
		type GroupAdminOrigin: EnsureOrigin<OriginFor<Self>, Success = CommunityIdOf<Self>>;

		/// The memberships manager: the part of issuing and retiring a membership that touches the item.
		type Issuer: Issue<AccountIdOf<Self>, Group = CommunityIdOf<Self>, Membership = Self::MembershipId>;

		/// Clears what holders and delegates wrote on an item, when its retirement completes.
		type ItemAttributes: ClearItemAttributes<CommunityIdOf<Self>, Self::MembershipId>;

		/// Owner and administrator of every community's collection of memberships.
		type MembershipsManagerAccount: Get<AccountIdOf<Self>>;

		/// The part of a community's registration that stays with the community: transferred to its
		/// account and held there as [`HoldReason::CommunityDeposit`]. A change applies to new
		/// communities only: each keeps the amount it was registered with.
		#[pallet::constant]
		type CommunityStoragePart: Get<NativeBalanceOf<Self>>;

		/// The part of a community's registration that is paid to the payee, for good.
		#[pallet::constant]
		type CommunityConsumptionPart: Get<NativeBalanceOf<Self>>;

		/// The account that holds a membership while it retires. It is the one the memberships manager
		/// is configured with.
		type RetirementHolder: Get<AccountIdOf<Self>>;

		/// What each issued membership holds on its community's account. A change applies to new
		/// issuances only: each membership keeps the amount it was issued with.
		#[pallet::constant]
		type MembershipDeposit: Get<NativeBalanceOf<Self>>;

		/// How long a retiring membership waits before its retirement can be completed, in the clock of
		/// [`pallet_communities::Config::BlockNumberProvider`].
		#[pallet::constant]
		type RetirementDelay: Get<MomentOf<Self>>;

		/// The most memberships a call issues or retires.
		#[pallet::constant]
		type MaxIssuePerCall: Get<u32>;
	}

	/// The in-code storage version: 1 since the memberships live in their groups' collections
	/// (`NextMembershipId`, `Deposits`, `Retiring` and `CommunityDeposits` are new). The runtime's
	/// stepped migration moves the data and sets the on-chain version.
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// The reasons this pallet holds funds on a community's account.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// The deposit of an issued membership.
		#[codec(index = 0)]
		MembershipDeposit,
		/// The storage part of a community's registration.
		#[codec(index = 1)]
		CommunityDeposit,
	}

	/// The identifier the next issued membership takes. Memberships of every community draw from it,
	/// so an identifier is never reused, in any community.
	#[pallet::storage]
	pub type NextMembershipId<T: Config> = StorageValue<_, T::MembershipId, ValueQuery>;

	/// The storage part of its registration that a community holds on its account, by community.
	/// A community registered for free has none.
	#[pallet::storage]
	pub type CommunityDeposits<T: Config> = StorageMap<_, Blake2_128Concat, CommunityIdOf<T>, NativeBalanceOf<T>>;

	/// What a membership holds on its community's account, by community and membership.
	#[pallet::storage]
	pub type Deposits<T: Config> =
		StorageMap<_, Blake2_128Concat, (CommunityIdOf<T>, T::MembershipId), NativeBalanceOf<T>>;

	/// When a retiring membership's retirement can be completed, by community and membership.
	#[pallet::storage]
	pub type Retiring<T: Config> = StorageMap<_, Blake2_128Concat, (CommunityIdOf<T>, T::MembershipId), MomentOf<T>>;

	/// A genesis community info.
	pub type GenesisCommunityOf<T> = (
		// community_id
		CommunityIdOf<T>,
		// name
		String,
		// admin
		AccountIdOf<T>,
		// maybe_decision_method
		Option<DecisionMethodFor<T>>,
		// maybe_rank
		Option<u16>,
	);

	#[pallet::genesis_config]
	#[derive(DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		/// A list of initial communities, with the most basic settings.
		pub communities: Vec<GenesisCommunityOf<T>>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			for (community_id, name, admin, maybe_decision_method, maybe_rank) in &self.communities {
				Pallet::<T>::try_register(
					None, // genesis communities are given for free :)
					*community_id,
					name.as_str(),
					admin,
					maybe_decision_method.clone(),
					None, // use the default track info. Let the community set the track later.
					*maybe_rank,
				)
				.unwrap();
			}
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The community with [`CommunityId`](pallet_communities::CommunityId)
		/// has been created.
		CommunityRegistered { id: T::CommunityId },
		/// A community issued memberships into its stock, with fresh identifiers from `first` on.
		MembershipsIssued {
			community: T::CommunityId,
			first: T::MembershipId,
			amount: u32,
		},
		/// A community retired memberships from its stock. Their retirements can be completed from
		/// `completable_at` on.
		MembershipsRetiring {
			community: T::CommunityId,
			amount: u32,
			completable_at: MomentOf<T>,
		},
		/// A retirement was completed: the membership was burnt and its deposit released.
		RetirementCompleted {
			community: T::CommunityId,
			membership_id: T::MembershipId,
		},
	}

	// Errors inform users that something worked or went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Community name didn't contain valid utf8 characters
		InvalidCommunityName,
		/// It was not possible to register the community
		CannotRegister,
		/// The community's account cannot cover the deposits of the issuance.
		MembershipLimit,
		/// The call issues more memberships than the limit.
		TooManyMemberships,
		/// No identifier is left for a new membership.
		NoMembershipIdLeft,
		/// The membership is not in the community's stock, or has no deposit.
		NotRetirable,
		/// The membership is not retiring.
		NotRetiring,
		/// The delay of the retirement has not passed.
		RetirementNotElapsed,
		/// The membership carries more attributes than the witness declares.
		TooManyAttributes,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke
	// state changes. These functions materialize as "extrinsics", which are often
	// compared to transactions. Dispatchable functions must be annotated with a
	// weight and must return a DispatchResult.
	#[pallet::call(weight(<T as Config>::WeightInfo))]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		pub fn register(
			origin: OriginFor<T>,
			community_id: CommunityIdOf<T>,
			name: CommunityName,
			first_admin: pallet_communities::AccountIdLookupOf<T>,
			maybe_decision_method: Option<DecisionMethodFor<T>>,
			maybe_track_info: Option<TrackInfoOf<T>>,
		) -> DispatchResult {
			Self::try_register(
				T::RegisterOrigin::ensure_origin(origin)?,
				community_id,
				core::str::from_utf8(&name).map_err(|_| Error::<T>::InvalidCommunityName)?,
				&T::Lookup::lookup(first_admin)?,
				maybe_decision_method,
				maybe_track_info,
				None,
			)
		}

		/// Issues `amount` memberships into the stock of the community, each with a fresh identifier,
		/// held by the community's account, locked, and backed by a deposit held on that account.
		///
		/// Refused, and nothing is issued, if the community's account cannot cover every deposit.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::issue_memberships((*amount).min(T::MaxIssuePerCall::get())))]
		pub fn issue_memberships(origin: OriginFor<T>, amount: u32) -> DispatchResult {
			let community = T::GroupAdminOrigin::ensure_origin(origin)?;
			ensure!(amount <= T::MaxIssuePerCall::get(), Error::<T>::TooManyMemberships);
			Self::do_issue_memberships(&community, amount)
		}

		/// Retires memberships of the community's stock. Each leaves the stock at once, and its
		/// retirement can be completed after the delay.
		///
		/// An assigned membership is released first.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::retire_memberships(ids.len() as u32))]
		pub fn retire_memberships(
			origin: OriginFor<T>,
			ids: BoundedVec<T::MembershipId, T::MaxIssuePerCall>,
		) -> DispatchResult {
			let community = T::GroupAdminOrigin::ensure_origin(origin)?;
			Self::do_retire_memberships(&community, &ids)
		}

		/// Completes a retirement, once its delay has passed: the membership is burnt, every
		/// attribute it carries is cleared, and its deposit is released to the community's account.
		///
		/// `attributes` is a witness: the number of attributes the membership carries, in every
		/// namespace. The call is refused if there are more, and the weight of the attributes that
		/// were not there is refunded.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::complete_retirement(*attributes))]
		pub fn complete_retirement(
			origin: OriginFor<T>,
			community_id: CommunityIdOf<T>,
			membership_id: T::MembershipId,
			attributes: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let cleared = Self::do_complete_retirement(&community_id, &membership_id, attributes)?;
			Ok(Some(<T as Config>::WeightInfo::complete_retirement(cleared)).into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Registers a community. `maybe_payment` names who pays for it and to whom (`None`, as for Root
	/// and genesis, pays nothing); the amounts are the pallet's own two parts.
	pub(crate) fn try_register(
		maybe_payment: Option<(NativeBalanceOf<T>, AccountIdOf<T>, AccountIdOf<T>)>,
		community_id: CommunityIdOf<T>,
		community_name: &str,
		admin: &AccountIdOf<T>,
		maybe_decision_method: Option<DecisionMethodFor<T>>,
		maybe_track_info: Option<TrackInfoOf<T>>,
		maybe_rank: Option<u16>,
	) -> DispatchResult {
		let admin_origin = frame_system::Origin::<T>::Signed(admin.clone());

		// Register first to check if community exists. The registration payment is not forwarded: it
		// is split between the payee and the community below.
		pallet_communities::Pallet::<T>::register(&admin_origin.clone().into(), &community_id, None)?;

		if let Some(decision_method) = maybe_decision_method {
			pallet_communities::Pallet::<T>::set_decision_method(admin_origin.into(), community_id, decision_method)?;
		}

		let community_account = pallet_communities::Pallet::<T>::community_account(&community_id);

		if let Some((_, payer, payee)) = maybe_payment {
			Self::pay_for_registration(&community_id, &community_account, &payer, &payee)?;
		}

		// Create the community's collection of memberships, owned and administered by the manager.
		let manager = T::MembershipsManagerAccount::get();
		T::CreateCollection::create_collection_with_id(
			community_id,
			&manager,
			&manager,
			&CollectionConfig {
				settings: Default::default(),
				max_supply: None,
				mint_settings: Default::default(),
			},
		)?;

		// Create governance track for community
		let community_origin: RuntimeOriginFor<T> = CommunityOrigin::<T>::new(community_id).into();
		T::Tracks::insert(
			community_id,
			maybe_track_info.unwrap_or_else(|| Self::default_tack(community_name)),
			community_origin.into_caller(),
		)?;
		// Induct community at Kreivo Governance with rank 0
		T::RankedCollective::induct(&community_account)?;
		if let Some(rank) = maybe_rank {
			for _ in 0..rank {
				T::RankedCollective::promote(&community_account)?;
			}
		}

		Self::deposit_event(Event::<T>::CommunityRegistered { id: community_id });
		Ok(())
	}

	/// Splits the registration payment: the consumption part goes to the payee, and the storage part to
	/// the community's account, where it is held and recorded.
	fn pay_for_registration(
		community_id: &CommunityIdOf<T>,
		community_account: &AccountIdOf<T>,
		payer: &AccountIdOf<T>,
		payee: &AccountIdOf<T>,
	) -> DispatchResult {
		let consumption = T::CommunityConsumptionPart::get();
		if !consumption.is_zero() {
			T::Balances::transfer(payer, payee, consumption, Preservation::Preserve)?;
		}

		let storage = T::CommunityStoragePart::get();
		if !storage.is_zero() {
			T::Balances::transfer(payer, community_account, storage, Preservation::Preserve)?;
			let reason: T::RuntimeHoldReason = HoldReason::CommunityDeposit.into();
			T::Balances::hold(&reason, community_account, storage)?;
			CommunityDeposits::<T>::insert(community_id, storage);
		}
		Ok(())
	}

	pub(crate) fn do_issue_memberships(community: &CommunityIdOf<T>, amount: u32) -> DispatchResult {
		ensure!(
			pallet_communities::Pallet::<T>::community_exists(community),
			pallet_communities::Error::<T>::CommunityDoesNotExist
		);

		let community_account = pallet_communities::Pallet::<T>::community_account(community);
		let deposit = T::MembershipDeposit::get();
		let reason: T::RuntimeHoldReason = HoldReason::MembershipDeposit.into();

		let first = NextMembershipId::<T>::get();
		let mut id = first;
		for _ in 0..amount {
			T::Issuer::issue(community, &id)?;
			T::Balances::hold(&reason, &community_account, deposit).map_err(|_| Error::<T>::MembershipLimit)?;
			Deposits::<T>::insert((community, id), deposit);
			id = id.increment().ok_or(Error::<T>::NoMembershipIdLeft)?;
		}
		NextMembershipId::<T>::put(id);

		Self::deposit_event(Event::<T>::MembershipsIssued {
			community: *community,
			first,
			amount,
		});
		Ok(())
	}

	pub(crate) fn do_retire_memberships(community: &CommunityIdOf<T>, ids: &[T::MembershipId]) -> DispatchResult {
		let holder = T::RetirementHolder::get();
		let completable_at = T::BlockNumberProvider::current_block_number().saturating_add(T::RetirementDelay::get());

		for id in ids {
			ensure!(Deposits::<T>::contains_key((community, id)), Error::<T>::NotRetirable);
			T::Issuer::retire(community, id, &holder).map_err(|_| Error::<T>::NotRetirable)?;
			Retiring::<T>::insert((community, id), completable_at);
		}

		Self::deposit_event(Event::<T>::MembershipsRetiring {
			community: *community,
			amount: ids.len() as u32,
			completable_at,
		});
		Ok(())
	}

	/// Completes a retirement, and returns how many attributes the item carried.
	pub(crate) fn do_complete_retirement(
		community: &CommunityIdOf<T>,
		id: &T::MembershipId,
		witness: u32,
	) -> Result<u32, DispatchError> {
		let completable_at = Retiring::<T>::get((community, id)).ok_or(Error::<T>::NotRetiring)?;
		ensure!(
			T::BlockNumberProvider::current_block_number() >= completable_at,
			Error::<T>::RetirementNotElapsed
		);
		let attributes = T::ItemAttributes::count(community, id);
		ensure!(attributes <= witness, Error::<T>::TooManyAttributes);

		// Burn it (the manager unlocks it, and clears what it wrote), then clear whatever others
		// wrote in their own namespaces, which the burn leaves behind.
		T::Issuer::burn(community, id)?;
		T::ItemAttributes::clear_all(community, id)?;

		Retiring::<T>::remove((community, id));
		if let Some(deposit) = Deposits::<T>::take((community, id)) {
			let community_account = pallet_communities::Pallet::<T>::community_account(community);
			let reason: T::RuntimeHoldReason = HoldReason::MembershipDeposit.into();
			T::Balances::release(&reason, &community_account, deposit, Precision::Exact)?;
		}

		Self::deposit_event(Event::<T>::RetirementCompleted {
			community: *community,
			membership_id: *id,
		});
		Ok(attributes)
	}

	fn default_tack(name: &str) -> TrackInfoOf<T> {
		use sp_runtime::Perbill;
		TrackInfo {
			name: str_array(name),
			max_deciding: 1,
			decision_deposit: 0u8.into(),
			prepare_period: 1u8.into(),
			decision_period: u8::MAX.into(),
			confirm_period: 1u8.into(),
			min_enactment_period: 1u8.into(),
			min_approval: pallet_referenda::Curve::LinearDecreasing {
				length: Perbill::from_percent(100),
				floor: Perbill::from_percent(50),
				ceil: Perbill::from_percent(100),
			},
			min_support: pallet_referenda::Curve::LinearDecreasing {
				length: Perbill::from_percent(100),
				floor: Perbill::from_percent(0),
				ceil: Perbill::from_percent(50),
			},
		}
	}
}
