use super::*;
use crate::AssetHubWestendParaSender;
use emulated_integration_tests_common::xcm_emulator::assert_expected_events;
use parity_scale_codec::Encode;

/// An account on Asset Hub that reaches Kreivo acts as the account with the same key here (and
/// not as a hashed one), so users keep one address across both chains.
#[test]
fn asset_hub_accounts_map_one_to_one() {
	let alice = AssetHubWestendParaSender::get();
	let fees: Asset = (ksm(), kreivo_runtime::EXISTENTIAL_DEPOSIT * 100).into();
	let remark = kreivo_runtime::RuntimeCall::System(frame_system::Call::remark_with_event {
		remark: b"one account, two chains".to_vec(),
	});
	let message = Xcm::<()>::builder_unsafe()
		.withdraw_asset(fees.clone())
		.buy_execution(fees, WeightLimit::Unlimited)
		.transact(OriginKind::SovereignAccount, None, remark.encode())
		.refund_surplus()
		.deposit_asset(AllCounted(1), account_location(&alice))
		.build();

	let dest = AssetHubWestend::sibling_location_of(Kreivo::para_id());
	AssetHubWestend::execute_with(|| {
		assert_ok!(<AssetHubWestend as AssetHubWestendParaPallet>::PolkadotXcm::send(
			asset_hub_westend_runtime::RuntimeOrigin::signed(alice.clone()),
			bx!(dest.into()),
			bx!(VersionedXcm::from(message)),
		));
	});

	Kreivo::execute_with(|| {
		type RuntimeEvent = kreivo_runtime::RuntimeEvent;
		assert_expected_events!(
			Kreivo,
			vec![
				RuntimeEvent::System(frame_system::Event::Remarked { sender, .. }) => {
					sender: *sender == alice,
				},
				RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true, .. }) => {},
			]
		);
	});
}
