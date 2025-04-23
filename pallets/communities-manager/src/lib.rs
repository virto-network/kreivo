#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

use frame_contrib_traits::{gas_tank::MakeTank, tracks::MutateTracks};
use frame_support::{
	pallet_prelude::*,
	traits::{
		nonfungibles_v2::Mutate as ItemMutate,
		nonfungibles_v2::{Create as CollectionCreate, Trading},
		Incrementable, OriginTrait, RankedMembers,
	},
};
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
use pallet_communities::{
	types::{AccountIdOf, CommunityIdOf, DecisionMethodFor, NativeBalanceOf, PalletsOriginOf, RuntimeOriginFor},
	Origin as CommunityOrigin,
};
use pallet_nfts::CollectionConfig;
use pallet_referenda::{TrackInfo, TracksInfo};
use parity_scale_codec::Decode;
use sp_runtime::{
	str_array,
	traits::{Get, StaticLookup},
};

type TrackInfoOf<T> = TrackInfo<NativeBalanceOf<T>, BlockNumberFor<T>>;

#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct TankConfig<Weight, BlockNumber> {
	capacity: Option<Weight>,
	periodicity: Option<BlockNumber>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use parity_scale_codec::HasCompact;

	type CommunityName = BoundedVec<u8, ConstU32<25>>;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_communities::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type CreateCollection: CollectionCreate<
			AccountIdOf<Self>,
			CollectionConfig<NativeBalanceOf<Self>, BlockNumberFor<Self>, CommunityIdOf<Self>>,
			CollectionId = CommunityIdOf<Self>,
		>;

		type MakeTank: MakeTank<
			Gas = Weight,
			TankId = (CommunityIdOf<Self>, <Self as Config>::MembershipId),
			BlockNumber = BlockNumberFor<Self>,
		>;

		type Tracks: TracksInfo<NativeBalanceOf<Self>, BlockNumberFor<Self>>
			+ MutateTracks<
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

		type CreateMembershipsOrigin: EnsureOrigin<OriginFor<Self>>;

		type MembershipId: Parameter + Decode + Incrementable + HasCompact;

		type MembershipsManagerCollectionId: Get<CommunityIdOf<Self>>;

		type MembershipsManagerOwner: Get<AccountIdOf<Self>>;

		type CreateMemberships: ItemMutate<
				AccountIdOf<Self>,
				Self::ItemConfig,
				CollectionId = CommunityIdOf<Self>,
				ItemId = <Self as Config>::MembershipId,
			> + Trading<
				AccountIdOf<Self>,
				NativeBalanceOf<Self>,
				CollectionId = CommunityIdOf<Self>,
				ItemId = <Self as Config>::MembershipId,
			>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The community with [`CommunityId`](pallet_communities::CommunityId)
		/// has been created.
		CommunityRegistered { id: T::CommunityId },
		/// The
		MembershipsCreated {
			starting_at: <T as Config>::MembershipId,
			amount: u32,
		},
	}

	// Errors inform users that something worked or went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Community name didn't contain valid utf8 characters
		InvalidCommunityName,
		/// It was not possible to register the community
		CannotRegister,
		/// The amount of memberships to create exceeds the limit of 1024
		CreatingTooManyMemberships,
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
			let maybe_deposit = T::RegisterOrigin::ensure_origin(origin)?;

			let community_name = core::str::from_utf8(&name).map_err(|_| Error::<T>::InvalidCommunityName)?;

			let first_admin_account_id = T::Lookup::lookup(first_admin)?;
			let admin_origin = frame_system::Origin::<T>::Signed(first_admin_account_id);

			// Register first to check if community exists
			pallet_communities::Pallet::<T>::register(&admin_origin.clone().into(), &community_id, maybe_deposit)?;

			if let Some(decision_method) = maybe_decision_method {
				pallet_communities::Pallet::<T>::set_decision_method(
					admin_origin.into(),
					community_id,
					decision_method,
				)?;
			}

			let community_account = pallet_communities::Pallet::<T>::community_account(&community_id);

			// Create memberships collection for community
			T::CreateCollection::create_collection_with_id(
				community_id,
				&community_account,
				&community_account,
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

			Self::deposit_event(Event::<T>::CommunityRegistered { id: community_id });
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::create_memberships((*amount).into()))]
		#[pallet::call_index(1)]
		pub fn create_memberships(
			origin: OriginFor<T>,
			amount: u16,
			starting_at: <T as Config>::MembershipId,
			#[pallet::compact] price: NativeBalanceOf<T>,
			tank_config: TankConfig<Weight, BlockNumberFor<T>>,
			maybe_expiration: Option<BlockNumberFor<T>>,
		) -> DispatchResult {
			ensure!(amount <= 1024u16, Error::<T>::CreatingTooManyMemberships);
			T::CreateMembershipsOrigin::ensure_origin(origin.clone())?;

			let collection_id = &T::MembershipsManagerCollectionId::get();
			let mut id = starting_at.clone();
			let mut minted = 0u32;
			for _ in 0..amount {
				T::CreateMemberships::mint_into(
					collection_id,
					&id,
					&T::MembershipsManagerOwner::get(),
					&Default::default(),
					true,
				)?;

				Self::do_set_gas_tank(&(*collection_id, id.clone()), &tank_config)?;

				if let Some(expiration) = maybe_expiration {
					T::CreateMemberships::set_typed_attribute(
						collection_id,
						&id,
						&b"membership_expiration",
						&expiration,
					)?;
				}

				T::CreateMemberships::set_price(
					&T::MembershipsManagerCollectionId::get(),
					&id,
					&T::MembershipsManagerOwner::get(),
					Some(price),
					None,
				)?;
				if let Some(next_id) = id.increment() {
					id = next_id;
					minted += 1;
				} else {
					break;
				}
			}

			Self::deposit_event(Event::<T>::MembershipsCreated {
				starting_at,
				amount: minted,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		pub fn set_gas_tank(
			origin: OriginFor<T>,
			community_id: CommunityIdOf<T>,
			membership_id: <T as Config>::MembershipId,
			config: TankConfig<Weight, BlockNumberFor<T>>,
		) -> DispatchResult {
			T::CreateMembershipsOrigin::ensure_origin(origin)?;
			Self::do_set_gas_tank(&(community_id, membership_id), &config)
		}
	}

	impl<T: Config> Pallet<T> {
		#[inline]
		pub(crate) fn do_set_gas_tank(
			tank_id: &(CommunityIdOf<T>, <T as Config>::MembershipId),
			config: &TankConfig<Weight, BlockNumberFor<T>>,
		) -> DispatchResult {
			let TankConfig { capacity, periodicity } = config;
			T::MakeTank::make_tank(tank_id, *capacity, *periodicity)?;

			Ok(())
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
}
