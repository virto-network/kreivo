use super::{
	config::{communities::memberships::CommunityMembershipsInstance, system::CommunityLookup, TreasuryAccount},
	constants::currency::EXISTENTIAL_DEPOSIT,
	xcm_config::*,
	Balances, Communities, CommunitiesManager, CommunityMemberships, FungibleAssetLocation, Runtime, RuntimeOrigin,
	CENTS, UNITS,
};

use frame_support::{
	assert_ok,
	traits::{fungible::Mutate, nonfungibles_v2::Inspect},
};
use pallet_communities_manager::TankConfig;
use parachains_common::AccountId;
use parity_scale_codec::Encode;
use runtime_constants::time::WEEKS;
use sp_core::crypto::AccountId32;
use sp_io::TestExternalities;
use sp_runtime::{traits::StaticLookup, BoundedVec};
use xcm_executor::{WeighedMessage, XcmExecutor};

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

#[test]
fn ensure_copying_membership_attributes_works() {
	TestExternalities::default().execute_with(|| {
		if cfg!(feature = "runtime-benchmarks") {
			// Note: Need to cover the deposit when the `runtime-benchmarks` feature is set.
			assert_ok!(Balances::mint_into(
				&TreasuryAccount::get(),
				EXISTENTIAL_DEPOSIT + 10 * CENTS
			));
		}

		// Create some memberships.
		assert_ok!(CommunitiesManager::create_memberships(
			RuntimeOrigin::root(),
			10,
			0,
			CENTS,                 // Any price works
			TankConfig::default(), // default means unlimited tank — also, the only publicly exposed constructor ;)
			Some(8 * WEEKS),       // expires in
		));

		const ALICE: AccountId32 = AccountId32::new([1; 32]);
		const BOB: AccountId32 = AccountId32::new([2; 32]);
		assert_ok!(Balances::mint_into(&ALICE, UNITS));
		assert_ok!(Balances::mint_into(&BOB, UNITS));

		assert_ok!(CommunitiesManager::register(
			RuntimeOrigin::root(),
			1,
			BoundedVec::try_from(b"First Community".to_vec()).expect("meets max length; qed"),
			CommunityLookup::unlookup(ALICE),
			// Use default values for decision method and track info
			None,
			None,
		));

		// Let's load some amount to the community, so the community can buy memberships
		// itself.
		assert_ok!(Balances::mint_into(&Communities::community_account(&1), UNITS));

		assert_ok!(Communities::dispatch_as_account(
			RuntimeOrigin::signed(ALICE),
			Box::new(
				pallet_nfts::Call::<Runtime, CommunityMembershipsInstance>::buy_item {
					collection: 0,
					item: 0,
					bid_price: CENTS
				}
				.into()
			)
		));

		assert_ok!(Communities::add_member(
			RuntimeOrigin::signed(ALICE),
			CommunityLookup::unlookup(BOB)
		));

		let key: Vec<u8> = b"membership_gas".to_vec();
		assert!(CommunityMemberships::system_attribute(&1, Some(&0), &key.encode()).is_some());
	})
}

#[test]
fn ensure_asset_creation_when_depositing_nonexisting_assets_works() {
	use frame_support::traits::fungibles::{roles::Inspect as _, Inspect as _};
	use xcm::latest::prelude::*;

	TestExternalities::default().execute_with(|| {
		let asset_id = FungibleAssetLocation::Sibling(virto_common::Para {
			id: 1000,
			pallet: 50,
			index: 42,
		});

		assert!(!super::Assets::asset_exists(asset_id));

		assert!(matches!(
			XcmExecutor::<XcmConfig>::execute(
				Location::new(1, [Parachain(1000)]),
				WeighedMessage::new(
					Weight::zero(),
					Xcm(vec![
						ReserveAssetDeposited(
							vec![
								Asset {
									id: Location::parent().into(),
									fun: Fungible(10000000000)
								},
								Asset {
									id: Location::new(1, [Parachain(1000), PalletInstance(50), GeneralIndex(42)])
										.into(),
									fun: Fungible(10000000000)
								},
							]
							.into()
						),
						ClearOrigin,
						BuyExecution {
							fees: Asset {
								id: Location::parent().into(),
								fun: Fungible(10000000000)
							},
							weight_limit: Unlimited,
						},
						DepositAsset {
							assets: Wild(All),
							beneficiary: Location::new(
								0,
								[AccountId32 {
									network: None,
									id: [1u8; 32],
								}]
							)
						}
					])
				),
				&mut [0u8; 32],
				Weight::zero(),
			),
			Outcome::Complete { .. }
		));

		assert!(super::Assets::asset_exists(asset_id));
		assert_eq!(super::Assets::owner(asset_id), Some(TreasuryAccount::get()));
		assert_eq!(super::Assets::balance(asset_id, AccountId::new([1u8; 32])), 10000000000);
	})
}

#[test]
fn fungible_asset_location_encoded_sizes() {
	let asset_id = FungibleAssetLocation::Here(u32::MAX);
	assert_eq!(asset_id.encode().len(), 5);

	let asset_id = FungibleAssetLocation::Sibling(virto_common::Para {
		id: u16::MAX,
		pallet: u8::MAX,
		index: u32::MAX,
	});
	assert_eq!(asset_id.encode().len(), 8);

	// DOT, as stored on chain: `02 00 00`.
	let asset_id = FungibleAssetLocation::External {
		network: virto_common::NetworkId::Polkadot,
		child: None,
	};
	assert_eq!(asset_id.encode(), alloc::vec![2, 0, 0]);

	// An external *parachain* asset takes 10 bytes with this shape. Shrinking it (as #472 did)
	// changes the DOT id's encoding, which would need a storage migration for the `Assets` keys.
	let asset_id = FungibleAssetLocation::External {
		network: virto_common::NetworkId::Polkadot,
		child: Some(virto_common::Para {
			id: u16::MAX,
			pallet: u8::MAX,
			index: u32::MAX,
		}),
	};
	assert_eq!(asset_id.encode().len(), 10);
}

/// Helpers to drive `pallet_pass` from a test.
mod pass {
	use super::*;

	use frame_contrib_traits::authn::{util::AuthorityFromPalletId, Challenger};
	use frame_support::pallet_prelude::*;
	use pass_substrate_keys::{KeyRegistration, SignedMessage};
	use sp_core::{sr25519, Pair};
	use sp_runtime::MultiSignature;

	pub use frame_contrib_traits::authn::{DeviceId, HashedUserId};

	use crate::{
		config::system::{KreivoChallenger, PassDeviceAttestation, PassPalletId},
		BlockNumber, System,
	};

	/// Builds a valid `SubstrateKey` device attestation for `pass_account`.
	pub fn attestation(pass_account: &AccountId, seed: [u8; 32]) -> (PassDeviceAttestation, DeviceId) {
		let pair = sr25519::Pair::from_seed(&seed);
		let context: BlockNumber = System::block_number();
		// `pallet_pass` uses the pass account's encoding as the extrinsic context.
		let xtc = pass_account.encode();

		let message = SignedMessage {
			context,
			challenge: KreivoChallenger::generate(&context, &xtc),
			authority_id: AuthorityFromPalletId::<PassPalletId>::get(),
		};
		let public = AccountId::new(pair.public().0);
		let signature = MultiSignature::Sr25519(pair.sign(message.message().as_ref()));
		let device_id = *AsRef::<[u8; 32]>::as_ref(&public);

		(
			PassDeviceAttestation::SubstrateKey(KeyRegistration {
				public,
				message,
				signature,
			}),
			device_id,
		)
	}

	/// The address `pallet_pass` derives for a given user id.
	pub fn account(user: HashedUserId) -> AccountId {
		<() as pallet_pass::AddressGenerator<Runtime, ()>>::generate_address(user)
	}
}

/// A pass account keeps its first two devices without a deposit; the third one is charged.
#[test]
fn pass_accounts_hold_two_devices_for_free() {
	use frame_support::traits::fungible::InspectHold;

	use super::{config::system::AccountDevicesReason, Pass};

	TestExternalities::default().execute_with(|| {
		const ALICE: AccountId32 = AccountId32::new([1; 32]);
		assert_ok!(Balances::mint_into(&ALICE, UNITS));

		let account = pass::account([1u8; 32]);
		let (first, _) = pass::attestation(&account, [10u8; 32]);
		assert_ok!(Pass::register(RuntimeOrigin::signed(ALICE), [1u8; 32], first));
		assert_ok!(Balances::mint_into(&account, UNITS));

		let held = || Balances::balance_on_hold(&AccountDevicesReason::get(), &account);

		let (second, _) = pass::attestation(&account, [11u8; 32]);
		assert_ok!(Pass::add_device(RuntimeOrigin::signed(account.clone()), second));
		assert_eq!(held(), 0, "the first two devices are free");

		let (third, _) = pass::attestation(&account, [12u8; 32]);
		assert_ok!(Pass::add_device(RuntimeOrigin::signed(account.clone()), third));
		assert!(held() > 0, "the third device is charged");
	})
}
