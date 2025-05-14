use super::*;

use crate::Communities;
use core::marker::PhantomData;
use frame_support::{traits::OriginTrait, PalletId};
use pallet_communities::AccountIdOf;
use sp_runtime::{
	traits::{AccountIdConversion, TryConvert},
	SaturatedConversion,
};
use xcm::latest::{BodyId, Junction, Junction::Plurality, Location};
use xcm_executor::traits::ConvertLocation;

pub struct PluralityConvertsToCommunityAccountId;
impl ConvertLocation<AccountId> for PluralityConvertsToCommunityAccountId {
	fn convert_location(location: &Location) -> Option<AccountId> {
		match location.unpack() {
			(
				0,
				[Plurality {
					id: BodyId::Index(id), ..
				}],
			) => Some(Communities::community_account(&(*id).saturated_into())),
			_ => None,
		}
	}
}

pub struct HereConvertsTo<Account>(PhantomData<Account>);
impl<Account: Get<AccountId>, AccountId: From<[u8; 32]>> ConvertLocation<AccountId> for HereConvertsTo<Account> {
	fn convert_location(location: &Location) -> Option<AccountId> {
		if location.is_here() {
			Some(Account::get().into())
		} else {
			None
		}
	}
}

pub struct AccountId32FromRelay<Network, AccountId>(PhantomData<(Network, AccountId)>);
impl<Network: Get<Option<NetworkId>>, AccountId: From<[u8; 32]> + Into<[u8; 32]> + Clone> ConvertLocation<AccountId>
	for AccountId32FromRelay<Network, AccountId>
{
	fn convert_location(location: &Location) -> Option<AccountId> {
		let id = match location.unpack() {
			(1, [AccountId32 { id, network: None }]) => id,
			(1, [AccountId32 { id, network }]) if *network == Network::get() => id,
			_ => return None,
		};

		Some((*id).into())
	}
}

pub struct SignedByCommunityToPlurality<T>(PhantomData<T>);
impl<T, OuterOrigin> TryConvert<OuterOrigin, Location> for SignedByCommunityToPlurality<T>
where
	OuterOrigin: OriginTrait<AccountId = AccountIdOf<T>> + Clone,
	T: pallet_communities::Config,
	Junction: TryFrom<pallet_communities::Origin<T>>,
{
	fn try_convert(o: OuterOrigin) -> Result<Location, OuterOrigin> {
		let Some(account_id) = o.clone().into_signer() else {
			return Err(o.clone());
		};
		let Some((_, community_id)) = PalletId::try_from_sub_account(&account_id) else {
			return Err(o.clone());
		};
		let origin = pallet_communities::Origin::<T>::new(community_id);
		let j = Junction::try_from(origin).map_err(|_| o)?;
		Ok(j.into())
	}
}
