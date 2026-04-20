use super::*;
use core::fmt::Debug;
use frame_support::traits::{fungibles, tokens::imbalance::ImbalanceAccounting};
use xcm_builder::AssetChecking;
use xcm_executor::traits::{ConvertLocation, MatchesFungibles, TransactAsset};
use xcm_executor::AssetsInHolding;

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
		Assets: fungibles::Inspect<AccountId, AssetId: 'static, Balance: 'static>
			+ fungibles::Mutate<AccountId>
			+ fungibles::Balanced<AccountId, OnDropCredit: 'static, OnDropDebt: 'static>
			+ fungibles::Create<AccountId>
			+ 'static,
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
where
	fungibles::Imbalance<
		<Assets as fungibles::Inspect<AccountId>>::AssetId,
		<Assets as fungibles::Inspect<AccountId>>::Balance,
		<Assets as fungibles::Balanced<AccountId>>::OnDropCredit,
		<Assets as fungibles::Balanced<AccountId>>::OnDropDebt,
	>: ImbalanceAccounting<u128>,
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

	fn mint_asset(what: &Asset, context: &XcmContext) -> Result<AssetsInHolding, XcmError> {
		// Create the asset if it doesn't exist yet before minting.
		if let Ok((asset_id, _amount)) = Matcher::matches_fungibles(what) {
			if !Assets::asset_exists(asset_id.clone()) {
				Assets::create(asset_id, NewAssetsOwner::get(), false, 1u32.into())
					.map_err(|_| XcmError::AssetNotFound)?;
			}
		}
		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::mint_asset(
			what, context,
		)
	}

	fn deposit_asset(
		what: AssetsInHolding,
		who: &Location,
		context: Option<&XcmContext>,
	) -> Result<(), (AssetsInHolding, XcmError)> {
		// Create any external assets that don't exist yet before depositing.
		// Note: mint_asset (called by ReserveAssetDeposited) already handles creation,
		// but deposit_asset may also be called directly via DepositAsset instruction.
		for asset in what.fungible_assets_iter() {
			if let Ok((asset_id, _amount)) = Matcher::matches_fungibles(&asset) {
				if !Assets::asset_exists(asset_id.clone()) {
					let _ = Assets::create(asset_id, NewAssetsOwner::get(), false, 1u32.into());
				}
			}
		}

		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::deposit_asset(
			what, who, context,
		)
	}

	fn withdraw_asset(
		what: &Asset,
		who: &Location,
		maybe_context: Option<&XcmContext>,
	) -> Result<AssetsInHolding, XcmError> {
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
	) -> Result<Asset, XcmError> {
		FungiblesAdapter::<Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount>::internal_transfer_asset(
			what, from, to, context
		)
	}
}
