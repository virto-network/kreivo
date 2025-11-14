use super::*;
use core::fmt::Debug;
use frame_support::traits::fungibles;
use xcm_builder::AssetChecking;
use xcm_executor::traits::{ConvertLocation, MatchesFungibles, TransactAsset};

pub struct FungiblesAdapterForExternalAssets<
	Assets,
	Matcher,
	AccountIdConverter,
	AccountId,
	CheckAsset,
	CheckingAccount,
	NewAssetsOwner,
>(
	PhantomData<(
		Assets,
		Matcher,
		AccountIdConverter,
		AccountId,
		CheckAsset,
		CheckingAccount,
		NewAssetsOwner,
	)>,
);

impl<
		Assets: fungibles::Mutate<AccountId> + fungibles::Create<AccountId>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		AccountIdConverter: ConvertLocation<AccountId>,
		AccountId: Eq + Clone + Debug, /* can't get away without it since Currency is generic over it. */
		CheckAsset: AssetChecking<Assets::AssetId>,
		CheckingAccount: Get<AccountId>,
		NewAssetsOwner: Get<AccountId>,
	> TransactAsset
	for FungiblesAdapterForExternalAssets<
		Assets,
		Matcher,
		AccountIdConverter,
		AccountId,
		CheckAsset,
		CheckingAccount,
		NewAssetsOwner,
	>
{
	fn can_check_in(origin: &Location, what: &Asset, context: &XcmContext) -> XcmResult {
		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::can_check_in(
			origin, what, context,
		)
	}

	fn check_in(origin: &Location, what: &Asset, context: &XcmContext) {
		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::check_in(
			origin, what, context,
		)
	}

	fn can_check_out(dest: &Location, what: &Asset, context: &XcmContext) -> XcmResult {
		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::can_check_out(
			dest, what, context,
		)
	}

	fn check_out(dest: &Location, what: &Asset, context: &XcmContext) {
		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::check_out(
			dest, what, context,
		)
	}

	fn deposit_asset(what: &Asset, who: &Location, context: Option<&XcmContext>) -> XcmResult {
		let (asset_id, _) = Matcher::matches_fungibles(what)?;

		if !Assets::asset_exists(asset_id.clone()) {
			Assets::create(asset_id, NewAssetsOwner::get(), false, 1u32.into()).map_err(|_| XcmError::AssetNotFound)?;
		}

		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::deposit_asset(
			what, who, context,
		)
	}

	fn withdraw_asset(
		what: &Asset,
		who: &Location,
		maybe_context: Option<&XcmContext>,
	) -> Result<xcm_executor::AssetsInHolding, XcmError> {
		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::withdraw_asset(
			what,
			who,
			maybe_context,
		)
	}

	fn internal_transfer_asset(
		what: &Asset,
		from: &Location,
		to: &Location,
		context: &XcmContext,
	) -> Result<xcm_executor::AssetsInHolding, XcmError> {
		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::internal_transfer_asset(
			what, from, to, context
		)
	}
}
