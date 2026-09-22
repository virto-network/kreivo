use super::*;
use crate::{
	chains::{
		asset_hub_westend::genesis::{AssetHubWestendAssetOwner, ED as ASSET_HUB_ED},
		kreivo::genesis::usdt,
	},
	AssetHubWestendParaReceiver, AssetHubWestendParaSender, KreivoParaReceiver, KreivoParaSender,
};
use emulated_integration_tests_common::{ASSETS_PALLET_ID, USDT_ID};
use frame_support::assert_noop;

#[test]
fn ksm_arrives_from_asset_hub() {
	let receiver = KreivoParaReceiver::get();
	let amount = ASSET_HUB_ED * 1_000;
	let before = kreivo_ksm_of(&receiver);

	send_ksm_from_asset_hub(AssetHubWestendParaSender::get(), &receiver, amount);

	Kreivo::execute_with(|| Kreivo::assert_xcmp_queue_success(None));
	let received = kreivo_ksm_of(&receiver) - before;
	// Everything but the execution fees on Kreivo.
	assert!(received > 0 && received < amount, "received {received} of {amount}");
}

#[test]
fn ksm_goes_back_to_asset_hub() {
	let kreivo_account = KreivoParaSender::get();
	let receiver = AssetHubWestendParaReceiver::get();
	let amount = ASSET_HUB_ED * 1_000;
	// Kreivo's sovereign account on Asset Hub holds the KSM that Kreivo's accounts own.
	send_ksm_from_asset_hub(AssetHubWestendParaSender::get(), &kreivo_account, amount * 2);
	Kreivo::execute_with(|| Kreivo::assert_xcmp_queue_success(None));

	let before = asset_hub_ksm_of(&receiver);
	let dest = Kreivo::sibling_location_of(AssetHubWestend::para_id());
	Kreivo::execute_with(|| {
		assert_ok!(
			<Kreivo as KreivoParaPallet>::PolkadotXcm::transfer_assets_using_type_and_then(
				kreivo_runtime::RuntimeOrigin::signed(kreivo_account),
				bx!(dest.into()),
				bx!(Assets::from(vec![(ksm(), amount).into()]).into()),
				bx!(TransferType::DestinationReserve),
				bx!(AssetId(ksm()).into()),
				bx!(TransferType::DestinationReserve),
				deposit_to(&receiver),
				WeightLimit::Unlimited,
			)
		);
	});

	AssetHubWestend::execute_with(|| AssetHubWestend::assert_xcmp_queue_success(None));
	let received = asset_hub_ksm_of(&receiver) - before;
	assert!(received > 0 && received < amount, "received {received} of {amount}");
}

#[test]
fn usdt_arrives_from_asset_hub() {
	let sender = AssetHubWestendParaSender::get();
	let receiver = KreivoParaReceiver::get();
	let amount = 5_000_000;
	AssetHubWestend::mint_asset(
		asset_hub_westend_runtime::RuntimeOrigin::signed(AssetHubWestendAssetOwner::get()),
		USDT_ID,
		sender.clone(),
		amount * 2,
	);

	let usdt_on_asset_hub = Location::new(0, [PalletInstance(ASSETS_PALLET_ID), GeneralIndex(USDT_ID.into())]);
	let dest = AssetHubWestend::sibling_location_of(Kreivo::para_id());
	AssetHubWestend::execute_with(|| {
		assert_ok!(
			<AssetHubWestend as AssetHubWestendParaPallet>::PolkadotXcm::transfer_assets_using_type_and_then(
				asset_hub_westend_runtime::RuntimeOrigin::signed(sender),
				bx!(dest.into()),
				bx!(Assets::from(vec![
					(ksm(), ASSET_HUB_ED * 100).into(),
					(usdt_on_asset_hub, amount).into(),
				])
				.into()),
				bx!(TransferType::LocalReserve),
				bx!(AssetId(ksm()).into()),
				bx!(TransferType::LocalReserve),
				bx!(VersionedXcm::from(
					Xcm::<()>::builder_unsafe()
						.deposit_asset(AllCounted(2), account_location(&receiver))
						.build()
				)),
				WeightLimit::Unlimited,
			)
		);
	});

	Kreivo::execute_with(|| Kreivo::assert_xcmp_queue_success(None));
	Kreivo::execute_with(|| {
		assert_eq!(<Kreivo as KreivoParaPallet>::Assets::balance(usdt(), &receiver), amount);
	});
}

/// USDT on Kreivo is bridge-backed (`BridgeBackedAssets`), so it can't leave through Asset Hub
/// by reserve transfer, even the USDT that arrived that way.
#[test]
fn bridge_backed_usdt_cannot_be_reserve_transferred() {
	let usdt_on_kreivo = Location::new(
		1,
		[
			Parachain(AssetHubWestend::para_id().into()),
			PalletInstance(ASSETS_PALLET_ID),
			GeneralIndex(USDT_ID.into()),
		],
	);
	let dest = Kreivo::sibling_location_of(AssetHubWestend::para_id());
	Kreivo::execute_with(|| {
		assert_noop!(
			<Kreivo as KreivoParaPallet>::PolkadotXcm::transfer_assets(
				kreivo_runtime::RuntimeOrigin::signed(KreivoParaSender::get()),
				bx!(dest.into()),
				bx!(account_location(&AssetHubWestendParaReceiver::get()).into()),
				bx!(Assets::from(vec![(usdt_on_kreivo, 1_000_000u128).into()]).into()),
				0,
				WeightLimit::Unlimited,
			),
			pallet_xcm::Error::<kreivo_runtime::Runtime>::Filtered
		);
	});
}
