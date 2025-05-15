use super::Runtime;

macro_rules! assert_call_size {
	($pallet: ident) => {
		println!(
			"size_of<{}::Call>: {}",
			stringify!($pallet),
			&core::mem::size_of::<$pallet::Call<Runtime>>(),
		);
		assert!(core::mem::size_of::<$pallet::Call<Runtime>>() as u32 <= 1024);
	};
	($pallet: ident, $instance: path) => {
		println!(
			"size_of<$pallet::Call>: {}",
			&core::mem::size_of::<$pallet::Call<Runtime, $instance>>(),
		);
		assert!(core::mem::size_of::<$pallet::Call<Runtime, $instance>>() as u32 <= 1024);
	};
}

#[test]
fn runtime_sanity_call_does_not_exceed_1kb() {
	// System: frame_system = 0
	assert_call_size!(frame_system);
	// ParachainSystem: cumulus_pallet_parachain_system = 1
	assert_call_size!(cumulus_pallet_parachain_system);
	// Timestamp: pallet_timestamp = 2
	assert_call_size!(pallet_timestamp);
	// ParachainInfo: parachain_info = 3
	assert_call_size!(parachain_info);
	// Balances: pallet_balances = 10
	assert_call_size!(pallet_balances);
	// TransactionPayment: pallet_transaction_payment = 11
	assert_call_size!(pallet_transaction_payment);
	// Burner: pallet_burner = 12
	// assert_call_size!(pallet_burner);
	// Assets: pallet_assets::<Instance1> = 13
	assert_call_size!(pallet_assets, pallet_assets::Instance1);
	// AssetTxPayment: pallet_asset_tx_payment::{Pallet, Storage, Event<T>} = 14
	assert_call_size!(pallet_asset_tx_payment);
	// Authorship: pallet_authorship = 20
	assert_call_size!(pallet_authorship);
	// CollatorSelection: pallet_collator_selection = 21
	assert_call_size!(pallet_collator_selection);
	// Session: pallet_session = 22
	assert_call_size!(pallet_session);
	// Aura: pallet_aura = 23
	assert_call_size!(pallet_aura);
	// AuraExt: cumulus_pallet_aura_ext = 24
	assert_call_size!(cumulus_pallet_aura_ext);
	// XcmpQueue: cumulus_pallet_xcmp_queue = 30
	assert_call_size!(cumulus_pallet_xcmp_queue);
	// PolkadotXcm: pallet_xcm = 31
	assert_call_size!(pallet_xcm);
	// CumulusXcm: cumulus_pallet_xcm = 32
	assert_call_size!(cumulus_pallet_xcm);
	// MessageQueue: pallet_message_queue = 33
	assert_call_size!(pallet_message_queue);
	// // AssetRegistry: pallet_asset_registry = 34
	// assert_call_size!(pallet_asset_registry);
	// Sudo: pallet_sudo = 40
	// assert_call_size!(pallet_sudo);
	// Multisig: pallet_multisig = 42
	assert_call_size!(pallet_multisig);
	// Utility: pallet_utility = 43
	assert_call_size!(pallet_utility);
	// Proxy: pallet_proxy = 44
	assert_call_size!(pallet_proxy);
	// Scheduler: pallet_scheduler = 45
	assert_call_size!(pallet_scheduler);
	// Preimage: pallet_preimage = 46
	assert_call_size!(pallet_preimage);
	// Treasury: pallet_treasury = 50
	assert_call_size!(pallet_treasury);
	// Payments: pallet_payments = 60
	assert_call_size!(pallet_payments);
}
