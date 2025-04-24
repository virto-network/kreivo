use crate::{
	types::{CommunityIdOf, CommunityState::Active, MembershipIdOf, RuntimeOriginFor},
	AccountIdOf, CommunityIdFor, Config, Info, Pallet,
};
use core::marker::PhantomData;
use frame_contrib_traits::memberships::{GenericRank, Inspect};
use frame_support::{
	pallet_prelude::*,
	traits::{EnsureOriginWithArg, MapSuccess, OriginTrait},
};
use frame_system::EnsureSigned;
#[cfg(feature = "xcm")]
use sp_runtime::traits::TryConvert;
use sp_runtime::{morph_types, Permill};

pub struct EnsureCommunity<T>(PhantomData<T>);

impl<T> EnsureOrigin<RuntimeOriginFor<T>> for EnsureCommunity<T>
where
	RuntimeOriginFor<T>: OriginTrait + Into<Result<RawOrigin<T>, RuntimeOriginFor<T>>> + From<RawOrigin<T>>,
	T: Config,
{
	type Success = T::CommunityId;

	fn try_origin(o: RuntimeOriginFor<T>) -> Result<Self::Success, RuntimeOriginFor<T>> {
		use frame_system::RawOrigin::{None, Root};
		if matches!(o.as_system_ref(), Some(Root) | Some(None)) {
			return Err(o);
		}
		let id = match o.clone().into() {
			Ok(RawOrigin { community_id, .. }) => community_id,
			Err(_) => {
				let origin = o.clone().into_caller();
				CommunityIdFor::<T>::get(origin).ok_or_else(|| o.clone())?
			}
		};
		Info::<T>::get(id)
			.and_then(|c| c.state.eq(&Active).then_some(id))
			.ok_or(o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOriginFor<T>, ()> {
		use crate::BenchmarkHelper;
		Ok(RawOrigin::new(T::BenchmarkHelper::community_id()).into())
	}
}

morph_types! {
	pub type PaymentForCreate<
		AccountId,
		GetAmount: TypedGet,
		GetReceiver: TypedGet<Type = AccountId>
	>: Morph = |sender: AccountId| -> Option<(GetAmount::Type, AccountId, GetReceiver::Type)> {
		Some((GetAmount::get(), sender, GetReceiver::get()))
	};
}

pub type EnsureSignedPays<T, Amount, Beneficiary> =
	MapSuccess<EnsureSigned<AccountIdOf<T>>, PaymentForCreate<AccountIdOf<T>, Amount, Beneficiary>>;

pub struct EnsureMember<T>(PhantomData<T>);

impl<T> EnsureOriginWithArg<RuntimeOriginFor<T>, CommunityIdOf<T>> for EnsureMember<T>
where
	T: Config,
	RuntimeOriginFor<T>: OriginTrait + From<frame_system::Origin<T>>,
{
	type Success = ();

	fn try_origin(
		o: RuntimeOriginFor<T>,
		community_id: &CommunityIdOf<T>,
	) -> Result<Self::Success, RuntimeOriginFor<T>> {
		use frame_system::RawOrigin::Signed;

		match o.clone().into() {
			Ok(Signed(who)) => {
				if T::MemberMgmt::is_member_of(community_id, &who) {
					Ok(())
				} else {
					Err(o.clone())
				}
			}
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_community_id: &CommunityIdOf<T>) -> Result<RuntimeOriginFor<T>, ()> {
		todo!("Find an account that is a member of this community");
	}
}

/// Origin to represent the voice of a community or a subset of its members
/// as well as the voting preference of said group.
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
pub struct RawOrigin<T: Config> {
	community_id: CommunityIdOf<T>,
	subset: Option<Subset<T>>,
}

impl<T: Config> RawOrigin<T> {
	pub const fn new(community_id: CommunityIdOf<T>) -> Self {
		RawOrigin {
			community_id,
			subset: None,
		}
	}

	pub fn with_subset(&mut self, s: Subset<T>) {
		self.subset = Some(s);
	}

	pub fn id(&self) -> CommunityIdOf<T> {
		self.community_id
	}
}

/// Subsets of the community can also have a voice
#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum Subset<T: Config> {
	Member(MembershipIdOf<T>),
	Members { count: u32 },
	Fraction(Permill),
	AtLeastRank(GenericRank),
}

#[cfg(feature = "xcm")]
impl<T> TryConvert<RuntimeOriginFor<T>, xcm::latest::Location> for RawOrigin<T>
where
	T: Config,
	RuntimeOriginFor<T>: Into<Result<RawOrigin<T>, RuntimeOriginFor<T>>>,
	xcm::latest::Junction: TryFrom<RawOrigin<T>>,
{
	fn try_convert(o: RuntimeOriginFor<T>) -> Result<xcm::latest::Location, RuntimeOriginFor<T>> {
		let Ok(community @ RawOrigin { .. }) = o.clone().into() else {
			return Err(o);
		};
		let j = xcm::latest::Junction::try_from(community).map_err(|_| o)?;
		Ok(j.into())
	}
}

#[cfg(feature = "xcm")]
impl<T> TryFrom<RawOrigin<T>> for xcm::latest::Junction
where
	T: Config,
	u32: From<CommunityIdOf<T>>,
{
	type Error = ();

	fn try_from(o: RawOrigin<T>) -> Result<Self, Self::Error> {
		use xcm::latest::{BodyId, BodyPart, Junction::Plurality};
		let part = match o.subset {
			None => BodyPart::Voice,
			Some(Subset::Member(_)) => BodyPart::Members { count: 1 },
			Some(Subset::Members { count }) => BodyPart::Members { count },
			Some(Subset::Fraction(per)) => BodyPart::Fraction {
				nom: per.deconstruct(),
				denom: <Permill as sp_runtime::PerThing>::ACCURACY,
			},
			Some(Subset::AtLeastRank(_)) => return Err(()),
		};
		Ok(Plurality {
			id: BodyId::Index(o.community_id.into()),
			part,
		})
	}
}

#[cfg(feature = "xcm")]
impl<T: Config> TryFrom<xcm::latest::Junction> for RawOrigin<T>
where
	T: Config,
	T::CommunityId: From<u32> + From<u64>,
{
	type Error = ();

	fn try_from(value: xcm::latest::Junction) -> Result<Self, Self::Error> {
		use xcm::latest::{BodyId::Index, BodyPart::*, Junction::Plurality};
		let Plurality { id: Index(id), part } = value else {
			return Err(());
		};
		let subset = match part {
			Voice => None,
			Members { count } => Some(Subset::Members { count }),
			Fraction { nom, denom } => Some(Subset::Fraction(Permill::from_rational(nom, denom))),
			_ => return Err(()),
		};
		let mut origin = RawOrigin::new(id.into());
		if let Some(s) = subset {
			origin.with_subset(s);
		}
		Ok(origin)
	}
}

/// Ensure the origin is any `Signed` origin.
pub struct AsSignedByCommunity<T>(PhantomData<T>);
impl<T, OuterOrigin> EnsureOrigin<OuterOrigin> for AsSignedByCommunity<T>
where
	OuterOrigin: OriginTrait
		+ From<frame_system::RawOrigin<T::AccountId>>
		+ From<RawOrigin<T>>
		+ Clone
		+ Into<Result<frame_system::RawOrigin<T::AccountId>, OuterOrigin>>
		+ Into<Result<RawOrigin<T>, OuterOrigin>>,
	T: Config,
{
	type Success = T::AccountId;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		match o.clone().into() {
			Ok(RawOrigin { community_id, .. }) => Ok(Pallet::<T>::community_account(&community_id)),
			_ => Err(o.clone()),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<OuterOrigin, ()> {
		use crate::BenchmarkHelper;
		let community_id = T::BenchmarkHelper::community_id();
		Ok(frame_system::RawOrigin::Signed(Pallet::<T>::community_account(&community_id)).into())
	}
}

/// Ensure the origin is any `Signed` origin.
pub struct AsSignedByStaticCommunity<T, C>(PhantomData<(T, C)>);
impl<T, C, OuterOrigin> EnsureOrigin<OuterOrigin> for AsSignedByStaticCommunity<T, C>
where
	OuterOrigin: OriginTrait
		+ From<frame_system::RawOrigin<T::AccountId>>
		+ From<RawOrigin<T>>
		+ Clone
		+ Into<Result<frame_system::RawOrigin<T::AccountId>, OuterOrigin>>
		+ Into<Result<RawOrigin<T>, OuterOrigin>>,
	T: Config,
	C: Get<CommunityIdOf<T>>,
{
	type Success = T::AccountId;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		match o.clone().into() {
			Ok(RawOrigin { ref community_id, .. }) if community_id == &C::get() => {
				Ok(Pallet::<T>::community_account(community_id))
			}
			_ => Err(o.clone()),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<OuterOrigin, ()> {
		use crate::BenchmarkHelper;
		let community_id = T::BenchmarkHelper::community_id();
		Ok(frame_system::RawOrigin::Signed(Pallet::<T>::community_account(&community_id)).into())
	}
}
