use super::*;
use emulated_integration_tests_common::xcm_emulator::assert_expected_events;
use parity_scale_codec::Encode;
use sp_runtime::MultiAddress;
use xcm::latest::AssetTransferFilter;

/// 1 KSM (and 1 WND, playing KSM here): both have 12 decimals.
const KSM: Balance = 1_000_000_000_000;

/// Someone with KSM on Asset Hub, and nothing on Kreivo, creates a community in one message:
/// they send KSM keeping their origin (`InitiateTransfer { preserve_origin: true }`), and the
/// same message `Transact`s `CommunitiesManager::register` as their account on Kreivo, which
/// pays the community deposit with the KSM that just arrived.
#[test]
fn a_community_can_be_created_from_asset_hub() {
	let founder = AccountId::new([7u8; 32]);
	let community_id = 42;
	AssetHubWestend::fund_accounts(vec![(founder.clone(), 10 * KSM)]);
	assert_eq!(kreivo_ksm_of(&founder), 0, "the founder has nothing on Kreivo yet");

	let register = kreivo_runtime::RuntimeCall::CommunitiesManager(pallet_communities_manager::Call::register {
		community_id,
		name: b"Founded from Asset Hub".to_vec().try_into().expect("short name; qed"),
		first_admin: MultiAddress::Id(founder.clone()),
		maybe_decision_method: None,
		maybe_track_info: None,
	});
	let fees: Asset = (ksm(), KSM / 10).into();
	let on_kreivo = Xcm::<()>::builder_unsafe()
		// The KSM that arrives goes to the founder's account (the same key on Kreivo)…
		.deposit_asset(AllCounted(1), account_location(&founder))
		// …which then registers the community, paying its deposit.
		.transact(OriginKind::SovereignAccount, None, register.encode())
		.refund_surplus()
		.deposit_asset(AllCounted(1), account_location(&founder))
		.build();
	let message = Xcm::<asset_hub_westend_runtime::RuntimeCall>(vec![
		WithdrawAsset((ksm(), 3 * KSM).into()),
		// Asset Hub's own execution and delivery fees for sending the message.
		PayFees {
			asset: (ksm(), KSM / 10).into(),
		},
		InitiateTransfer {
			destination: AssetHubWestend::sibling_location_of(Kreivo::para_id()),
			remote_fees: Some(AssetTransferFilter::ReserveDeposit(Definite(fees.into()))),
			preserve_origin: true,
			assets: vec![AssetTransferFilter::ReserveDeposit(Wild(AllCounted(1)))]
				.try_into()
				.expect("one asset; qed"),
			remote_xcm: on_kreivo,
		},
	]);

	AssetHubWestend::execute_with(|| {
		assert_ok!(<AssetHubWestend as AssetHubWestendParaPallet>::PolkadotXcm::execute(
			asset_hub_westend_runtime::RuntimeOrigin::signed(founder.clone()),
			bx!(VersionedXcm::from(message)),
			Weight::MAX,
		));
	});

	Kreivo::execute_with(|| {
		type RuntimeEvent = kreivo_runtime::RuntimeEvent;
		assert_expected_events!(
			Kreivo,
			vec![
				RuntimeEvent::CommunitiesManager(pallet_communities_manager::Event::CommunityRegistered { id }) => {
					id: *id == community_id,
				},
				RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true, .. }) => {},
			]
		);
		assert!(kreivo_runtime::Communities::community_exists(&community_id));
	});

	// The founder keeps what the deposit and fees didn't use.
	assert!(kreivo_ksm_of(&founder) > 0);
}
