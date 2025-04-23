#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
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
	AccountIdOf, AssetIdOf, CommunityIdOf, DecisionMethodFor, NativeBalanceOf, Origin as CommunityOrigin,
	PalletsOriginOf, RuntimeOriginFor,
};
use pallet_nfts::CollectionConfig;
use pallet_referenda::{TrackInfo, TracksInfo};
use parity_scale_codec::Decode;
use sp_runtime::{
	str_array,
	traits::{Get, StaticLookup},
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

pub mod weights;
pub use weights::*;

type TrackInfoOf<T> = TrackInfo<NativeBalanceOf<T>, BlockNumberFor<T>>;

#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct TankConfig<Weight, BlockNumber> {
	capacity: Option<Weight>,
	periodicity: Option<BlockNumber>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::DefaultNoBound;
	use parity_scale_codec::HasCompact;

	type CommunityName = BoundedVec<u8, ConstU32<25>>;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_communities::Config
	where
		AssetIdOf<Self>: MaybeSerializeDeserialize,
	{
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

		type MembershipId: Parameter + Decode + Incrementable + HasCompact + MaybeSerializeDeserialize;

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

	/// A genesis community info.
	pub type GenesisCommunityOf<T> = (
		// community_id,im
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

	/// A genesis membership info.
	pub type GenesisMembershipOf<T> = (
		// starting_at
		<T as Config>::MembershipId,
		// amount
		u16,
		// price
		NativeBalanceOf<T>,
		// tank_config,
		(Option<Weight>, Option<BlockNumberFor<T>>),
		// maybe_expiration
		Option<BlockNumberFor<T>>,
	);

	#[pallet::genesis_config]
	#[derive(DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		/// A list of initial communities, with the most basic settings.
		pub communities: Vec<GenesisCommunityOf<T>>,
		pub memberships: Vec<GenesisMembershipOf<T>>,
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

			for (starting_at, amount, price, tank_config, maybe_expiration) in &self.memberships {
				let (capacity, periodicity) = *tank_config;
				Pallet::<T>::try_create_memberships(
					starting_at.clone(),
					*amount,
					*price,
					TankConfig { capacity, periodicity },
					*maybe_expiration,
				)
				.unwrap()
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

			Self::try_create_memberships(starting_at.clone(), amount, price, tank_config, maybe_expiration)
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
}

impl<T: Config> Pallet<T> {
	pub(crate) fn try_register(
		maybe_deposit: Option<(NativeBalanceOf<T>, T::AccountId, T::AccountId)>,
		community_id: CommunityIdOf<T>,
		community_name: &str,
		admin: &T::AccountId,
		maybe_decision_method: Option<DecisionMethodFor<T>>,
		maybe_track_info: Option<TrackInfoOf<T>>,
		maybe_rank: Option<u16>,
	) -> DispatchResult {
		let admin_origin = frame_system::Origin::<T>::Signed(admin.clone());

		// Register first to check if community exists
		pallet_communities::Pallet::<T>::register(&admin_origin.clone().into(), &community_id, maybe_deposit)?;

		if let Some(decision_method) = maybe_decision_method {
			pallet_communities::Pallet::<T>::set_decision_method(admin_origin.into(), community_id, decision_method)?;
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
		if let Some(rank) = maybe_rank {
			for _ in 0..rank {
				T::RankedCollective::promote(&community_account)?;
			}
		}

		Self::deposit_event(Event::<T>::CommunityRegistered { id: community_id });
		Ok(())
	}

	pub(crate) fn try_create_memberships(
		starting_at: <T as Config>::MembershipId,
		amount: u16,
		price: NativeBalanceOf<T>,
		tank_config: TankConfig<Weight, BlockNumberFor<T>>,
		maybe_expiration: Option<BlockNumberFor<T>>,
	) -> DispatchResult {
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
				T::CreateMemberships::set_typed_attribute(collection_id, &id, &b"membership_expiration", &expiration)?;
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
