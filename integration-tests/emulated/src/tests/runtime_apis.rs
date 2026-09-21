use super::*;
use crate::{AssetHubWestendParaReceiver, KreivoParaSender};
use parity_scale_codec::{Decode, Encode};
use xcm::{VersionedAssetId, VersionedAssets, VersionedLocation};
use xcm_runtime_apis::{
	dry_run::runtime_decl_for_dry_run_api::DryRunApiV2, fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV2,
};

/// A wallet can dry-run a transfer on Kreivo, then the message it sends on Asset Hub, and price
/// its delivery, before signing anything.
#[test]
fn transfers_to_asset_hub_can_be_dry_run_end_to_end() {
	let sender = KreivoParaSender::get();
	let receiver = AssetHubWestendParaReceiver::get();
	let amount = kreivo_runtime::EXISTENTIAL_DEPOSIT * 1_000;
	let asset_hub = Kreivo::sibling_location_of(AssetHubWestend::para_id());
	// Kreivo's sovereign account on Asset Hub backs the KSM that leaves.
	AssetHubWestend::fund_para_sovereign(Kreivo::para_id(), amount * 10);

	let (forwarded, delivery_fees) = Kreivo::execute_with(|| {
		let call = kreivo_runtime::RuntimeCall::PolkadotXcm(pallet_xcm::Call::transfer_assets_using_type_and_then {
			dest: bx!(asset_hub.clone().into()),
			assets: bx!(Assets::from(vec![(ksm(), amount).into()]).into()),
			assets_transfer_type: bx!(TransferType::DestinationReserve),
			remote_fees_id: bx!(AssetId(ksm()).into()),
			fees_transfer_type: bx!(TransferType::DestinationReserve),
			custom_xcm_on_dest: deposit_to(&receiver),
			weight_limit: WeightLimit::Unlimited,
		});
		let origin = kreivo_runtime::OriginCaller::system(frame_system::RawOrigin::Signed(sender));
		let effects = kreivo_runtime::Runtime::dry_run_call(origin, call, XCM_VERSION).expect("dry run is supported");
		assert_ok!(&effects.execution_result);

		let (destination, messages) = effects
			.forwarded_xcms
			.into_iter()
			.find(|(destination, _)| *destination == VersionedLocation::from(asset_hub.clone()))
			.expect("the transfer sends a message to Asset Hub");
		let message = messages.into_iter().next().expect("one message");

		let fees = kreivo_runtime::Runtime::query_delivery_fees(
			destination,
			message.clone(),
			VersionedAssetId::from(AssetId(ksm())),
		)
		.expect("delivery to Asset Hub can be priced");
		(message, fees)
	});

	// Delivery to a sibling costs something, and it's paid in KSM.
	let VersionedAssets::V5(fees) = delivery_fees else {
		panic!("expected v5 assets")
	};
	let fee = fees.get(0).expect("one fee asset");
	assert_eq!(fee.id, AssetId(ksm()));
	assert!(matches!(fee.fun, Fungible(amount) if amount > 0));

	// Asset Hub receives it as a message it could `Transact` with, hence its own call type.
	let forwarded = VersionedXcm::<asset_hub_westend_runtime::RuntimeCall>::decode(&mut &forwarded.encode()[..])
		.expect("same encoding");
	AssetHubWestend::execute_with(|| {
		let effects = asset_hub_westend_runtime::Runtime::dry_run_xcm(
			AssetHubWestend::sibling_location_of(Kreivo::para_id()).into(),
			forwarded,
		)
		.expect("dry run is supported");
		assert!(
			matches!(effects.execution_result, Outcome::Complete { .. }),
			"{:?}",
			effects.execution_result
		);
	});
}
