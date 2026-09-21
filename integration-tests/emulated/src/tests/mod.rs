mod accounts;
mod reserve_transfers;
mod runtime_apis;

use crate::{
	chains::{asset_hub_westend::AssetHubWestendParaPallet, kreivo::KreivoParaPallet},
	AssetHubWestendPara as AssetHubWestend, KreivoPara as Kreivo,
};
use emulated_integration_tests_common::xcm_emulator::{bx, Parachain, TestExt};
use frame_support::assert_ok;
use parachains_common::{AccountId, Balance};
use xcm::{latest::prelude::*, VersionedXcm};
use xcm_executor::traits::TransferType;

/// KSM: the relay chain's token, reserved on Asset Hub.
fn ksm() -> Location {
	Location::parent()
}

fn account_location(who: &AccountId) -> Location {
	Location::new(
		0,
		[AccountId32 {
			network: None,
			id: who.clone().into(),
		}],
	)
}

fn deposit_to(who: &AccountId) -> Box<VersionedXcm<()>> {
	bx!(VersionedXcm::from(
		Xcm::<()>::builder_unsafe()
			.deposit_asset(AllCounted(1), account_location(who))
			.build()
	))
}

fn kreivo_ksm_of(who: &AccountId) -> Balance {
	Kreivo::execute_with(|| <Kreivo as KreivoParaPallet>::Balances::free_balance(who))
}

fn asset_hub_ksm_of(who: &AccountId) -> Balance {
	AssetHubWestend::execute_with(|| <AssetHubWestend as AssetHubWestendParaPallet>::Balances::free_balance(who))
}

/// Sends `amount` KSM from `from` on Asset Hub to `to` on Kreivo, as Asset Hub does since the
/// Kusama Asset Hub migration: Asset Hub is the reserve.
fn send_ksm_from_asset_hub(from: AccountId, to: &AccountId, amount: Balance) {
	let dest = AssetHubWestend::sibling_location_of(Kreivo::para_id());
	AssetHubWestend::execute_with(|| {
		assert_ok!(
			<AssetHubWestend as AssetHubWestendParaPallet>::PolkadotXcm::transfer_assets_using_type_and_then(
				asset_hub_westend_runtime::RuntimeOrigin::signed(from),
				bx!(dest.into()),
				bx!(Assets::from(vec![(ksm(), amount).into()]).into()),
				bx!(TransferType::LocalReserve),
				bx!(AssetId(ksm()).into()),
				bx!(TransferType::LocalReserve),
				deposit_to(to),
				WeightLimit::Unlimited,
			)
		);
	});
}
