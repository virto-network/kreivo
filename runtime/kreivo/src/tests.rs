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

/// The KSM charged for an incoming reserve-asset deposit is
/// `WeightToFee(<weight of the whole message>)`, and it is taken _in full_ by
/// the treasury: `AllowTopLevelPaidExecutionFrom` rewrites the sender's
/// `BuyExecution { weight_limit }` down to the weight the executor was
/// prepared with, and `UsingComponents` keeps whatever it isn't asked to
/// refund (this message carries no `RefundSurplus`).
#[test]
fn ensure_incoming_reserve_deposit_charges_expected_ksm_fee() {
	use frame_support::weights::WeightToFee as WeightToFeeT;
	use xcm::latest::prelude::*;
	use xcm_executor::traits::WeightBounds;

	TestExternalities::default().execute_with(|| {
		const BENEFICIARY: [u8; 32] = [1u8; 32];
		// What the sender puts into the holding register to pay for execution.
		const KSM_SENT: u128 = UNITS;
		const USD_SENT: u128 = 10 * UNITS;

		let usd = Location::new(1, [Parachain(1000), PalletInstance(50), GeneralIndex(42)]);
		let usd_asset_id = FungibleAssetLocation::Sibling(virto_common::Para {
			id: 1000,
			pallet: 50,
			index: 42,
		});

		let mut message = Xcm(vec![
			ReserveAssetDeposited(
				vec![
					Asset {
						id: Location::parent().into(),
						fun: Fungible(KSM_SENT),
					},
					Asset {
						id: usd.clone().into(),
						fun: Fungible(USD_SENT),
					},
				]
				.into(),
			),
			ClearOrigin,
			BuyExecution {
				fees: Asset {
					id: Location::parent().into(),
					fun: Fungible(KSM_SENT),
				},
				// This is what Asset Hub's `limited_reserve_transfer_assets` sends by default.
				weight_limit: Unlimited,
			},
			DepositAsset {
				assets: Wild(All),
				beneficiary: Location::new(
					0,
					[AccountId32 {
						network: None,
						id: BENEFICIARY,
					}],
				),
			},
		]);

		let weight = <XcmConfig as xcm_executor::Config>::Weigher::weight(&mut message, Weight::MAX)
			.expect("the message only uses instructions we have weights for; qed");
		let expected_fee = <super::WeightToFee as WeightToFeeT>::weight_to_fee(&weight);

		println!("message weight: {weight:?}");
		println!("KSM charged: {expected_fee} planck");

		assert!(matches!(
			XcmExecutor::<XcmConfig>::execute(
				AssetHubLocation::get(),
				WeighedMessage::new(weight, message),
				&mut [0u8; 32],
				Weight::zero(),
			),
			Outcome::Complete { .. }
		));

		// The whole KSM fee ends up in the treasury...
		assert_eq!(
			<Balances as frame_support::traits::fungible::Inspect<AccountId>>::balance(&TreasuryAccount::get()),
			expected_fee
		);
		// ...and the beneficiary keeps the change.
		assert_eq!(
			<Balances as frame_support::traits::fungible::Inspect<AccountId>>::balance(&AccountId::new(BENEFICIARY)),
			KSM_SENT - expected_fee
		);
		// Fees are only charged in KSM: the `Assets::Issued` amount is the full amount sent.
		assert_eq!(
			super::Assets::balance(usd_asset_id, AccountId::new(BENEFICIARY)),
			USD_SENT
		);

		// Pin the number down for this exact shape of message (4 instructions, 2 assets):
		// 16_477_291_000 units of ref time => 0.00050782014 KSM.
		#[cfg(not(feature = "paseo"))]
		{
			assert_eq!(weight.ref_time(), 16_477_291_000);
			assert_eq!(expected_fee, 507_820_140);
		}
	})
}

/// Helpers to drive `pallet_pass` from a test, without the `PassAuthenticate`
/// transaction extension that normally supplies the authentication context.
mod pass {
	use super::*;

	use frame_contrib_traits::authn::{util::AuthorityFromPalletId, Challenger};
	use frame_support::pallet_prelude::*;
	use pass_substrate_keys::{KeyRegistration, SignedMessage};
	use sp_core::{sr25519, Pair};
	use sp_runtime::MultiSignature;

	pub use crate::config::system::PassCredential;
	pub use frame_contrib_traits::authn::{DeviceId, HashedUserId};

	use crate::{
		config::system::{KreivoChallenger, PassDeviceAttestation, PassPalletId},
		BlockNumber, System,
	};

	// `add_device` reads the device the caller authenticated with from this storage
	// value, which the `PassAuthenticate` transaction extension sets.
	#[frame_support::storage_alias]
	pub type AuthenticatedDevice = StorageValue<Pass, (AccountId, DeviceId)>;

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

	/// Builds a valid `SubstrateKey` credential, as the `PassAuthenticate`
	/// transaction extension would carry it.
	pub fn credential(user_id: HashedUserId, seed: [u8; 32], xtc: &[u8]) -> PassCredential {
		let pair = sr25519::Pair::from_seed(&seed);
		let context: BlockNumber = System::block_number();
		let message = SignedMessage {
			context,
			challenge: KreivoChallenger::generate(&context, &xtc),
			authority_id: AuthorityFromPalletId::<PassPalletId>::get(),
		};
		let signature = MultiSignature::Sr25519(pair.sign(message.message().as_ref()));

		PassCredential::SubstrateKey(pass_substrate_keys::KeySignature {
			user_id,
			message,
			signature,
		})
	}

	/// The address `pallet_pass` derives for a given user id.
	pub fn account(user: HashedUserId) -> AccountId {
		<() as pallet_pass::AddressGenerator<Runtime, ()>>::generate_address(user)
	}
}

#[test]
fn ensure_pass_and_proxy_deposits_are_as_expected() {
	use frame_support::traits::{
		fungible::{Inspect as _, InspectHold},
		ReservableCurrency,
	};
	use parity_scale_codec::MaxEncodedLen;
	use pass::{attestation, AuthenticatedDevice};

	use super::{
		config::system::{AccountDevicesReason, AccountRegistrationReason},
		deposit, Balance, Pass, Proxy, ProxyType, MILLICENTS,
	};

	let pass_account = pass::account;

	TestExternalities::default().execute_with(|| {
		const ALICE: AccountId32 = AccountId32::new([1; 32]);
		const BOB: AccountId32 = AccountId32::new([2; 32]);

		assert_ok!(Balances::mint_into(&ALICE, UNITS));

		// 1. Creating a pass account.
		//
		// The registrar (here: ALICE, a plain signed origin) pays the
		// `RegistrarConsideration`: `ED + MILLICENTS * count * size_of(HashedUserId)`.
		let first = pass_account([1u8; 32]);
		let (first_attestation, first_device) = attestation(&first, [10u8; 32]);
		assert_ok!(Pass::register(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			first_attestation
		));

		let registrar_hold = Balances::balance_on_hold(&AccountRegistrationReason::get(), &ALICE);
		println!("pass account registration (1st): {registrar_hold} planck");
		assert_eq!(registrar_hold, EXISTENTIAL_DEPOSIT + 32 * MILLICENTS);

		// A second pass account created by the same registrar only adds the marginal cost.
		let second = pass_account([2u8; 32]);
		let (second_attestation, _) = attestation(&second, [20u8; 32]);
		assert_ok!(Pass::register(
			RuntimeOrigin::signed(ALICE),
			[2u8; 32],
			second_attestation
		));

		let registrar_hold_2 = Balances::balance_on_hold(&AccountRegistrationReason::get(), &ALICE);
		println!(
			"pass account registration (2nd): {} planck",
			registrar_hold_2 - registrar_hold
		);
		assert_eq!(registrar_hold_2, EXISTENTIAL_DEPOSIT + 64 * MILLICENTS);

		// The pass account itself holds no deposit: `register` only bumps its provider
		// reference. To hold any balance it still needs the existential deposit.
		assert_eq!(Balances::balance_on_hold(&AccountDevicesReason::get(), &first), 0);
		assert_eq!(Balances::minimum_balance(), EXISTENTIAL_DEPOSIT);
		assert!(Balances::mint_into(&first, EXISTENTIAL_DEPOSIT - 1).is_err());
		assert_ok!(Balances::mint_into(&first, UNITS));

		// 2. Registering additional devices on a pass account.
		//
		// `DeviceConsideration` is `SecondItemIsFree`, so devices #1 (the one added by
		// `register`) and #2 are free; every device after that is charged.
		AuthenticatedDevice::put((first.clone(), first_device));

		// `LinearStoragePrice<MILLICENTS, MILLICENTS / 10>` over the footprint
		// `FirstItemIsFree` hands down; the nesting shrinks both the count and the
		// (integer-divided) per-item size, so the totals below are not a clean
		// multiple of `max_encoded_len` of a device.
		let mel = pallet_pass::DeviceOf::<Runtime>::max_encoded_len();
		println!("size_of<PassDevice> (max encoded): {mel}");

		let mut held = Vec::new();
		for (n, seed) in [[11u8; 32], [12u8; 32], [13u8; 32]].into_iter().enumerate() {
			let (att, _) = attestation(&first, seed);
			assert_ok!(Pass::add_device(
				RuntimeOrigin::signed(first.clone()),
				att,
				pallet_pass::DeviceFilterOf::<Runtime>::Admin
			));

			let total = Balances::balance_on_hold(&AccountDevicesReason::get(), &first);
			println!(
				"device #{}: total held {total} planck (+{})",
				n + 2,
				total - held.last().copied().unwrap_or(0)
			);
			held.push(total);
		}

		// Devices #1 (from `register`) and #2 are free; #3 onwards are charged, and
		// the hold is re-computed over the whole set on every addition.
		assert_eq!(held[0], 0);
		assert_eq!(held[1], MILLICENTS + (MILLICENTS / 10) * (mel as Balance / 3 * 2));
		assert_eq!(
			held[2],
			MILLICENTS + (MILLICENTS / 10) * 2 * (mel as Balance / 4 * 3 / 3 * 2)
		);

		// 3. Adding a proxy controller.
		//
		// `pallet_proxy` reserves rather than holds, but `reserved_balance` also counts
		// the pass registrar holds taken above, so measure the delta.
		let reserved_before = <Balances as ReservableCurrency<AccountId>>::reserved_balance(&ALICE);
		assert_ok!(Proxy::add_proxy(
			RuntimeOrigin::signed(ALICE),
			CommunityLookup::unlookup(BOB),
			ProxyType::Any,
			0
		));

		let proxy_deposit = <Balances as ReservableCurrency<AccountId>>::reserved_balance(&ALICE) - reserved_before;
		println!("proxy (1st): {proxy_deposit} planck");
		assert_eq!(proxy_deposit, deposit(0, 100) + deposit(0, 33));

		#[cfg(not(feature = "paseo"))]
		{
			// Existential deposit needed to activate a (pass) account.
			assert_eq!(EXISTENTIAL_DEPOSIT, 33_333_333);
			// Registering a pass account: first, then each subsequent one.
			assert_eq!(registrar_hold, 43_999_989);
			assert_eq!(registrar_hold_2 - registrar_hold, 10_666_656);
			// Devices #2, #3, #4 (cumulative hold on the pass account).
			assert_eq!(held, alloc::vec![0, 3_199_971, 4_599_957]);
			// Adding the first proxy controller.
			assert_eq!(proxy_deposit, 44_333_289);
		}
	})
}

/// How much a community must hand a freshly registered pass account so it can
/// take on a proxy controller *and* register a second device.
///
/// The first device cannot be swapped out: `remove_device` has no last-device
/// guard, and once `Devices` is empty for the account `ensure_signer_is_pass_account`
/// rejects every subsequent call with `BadOrigin` — permanently. So the account
/// keeps both devices, and the funding has to cover holding two at once.
#[test]
fn ensure_community_pass_account_funding_covers_proxy_and_second_device() {
	use frame_support::traits::{
		fungible::{Inspect as _, InspectHold, Mutate as _},
		tokens::Preservation,
		ReservableCurrency,
	};
	use pass::{attestation, AuthenticatedDevice};

	use super::{
		config::system::{AccountDevicesReason, AccountRegistrationReason},
		deposit, Pass, Proxy, ProxyType, RuntimeCall,
	};

	const PROXY_DEPOSIT: u128 = deposit(0, 100) + deposit(0, 33);

	/// Registers a pass account through a community, transfers it `funding`, then
	/// adds a proxy and a second device. Returns the resulting
	/// `(free, reserved, device_hold)` of the pass account.
	fn attempt(funding: u128) -> Result<(u128, u128, u128), sp_runtime::DispatchError> {
		TestExternalities::default().execute_with(|| {
			const ALICE: AccountId32 = AccountId32::new([1; 32]);
			const CONTROLLER: AccountId32 = AccountId32::new([2; 32]);

			if cfg!(feature = "runtime-benchmarks") {
				// Memberships carry an item deposit when benchmarking.
				assert_ok!(Balances::mint_into(
					&TreasuryAccount::get(),
					EXISTENTIAL_DEPOSIT + 10 * CENTS
				));
			}

			// A community, with ALICE as its admin.
			assert_ok!(CommunitiesManager::create_memberships(
				RuntimeOrigin::root(),
				10,
				0,
				CENTS,
				TankConfig::default(),
				Some(8 * WEEKS),
			));
			assert_ok!(Balances::mint_into(&ALICE, UNITS));
			assert_ok!(CommunitiesManager::register(
				RuntimeOrigin::root(),
				1,
				BoundedVec::try_from(b"First Community".to_vec()).expect("meets max length; qed"),
				CommunityLookup::unlookup(ALICE),
				None,
				None,
			));
			let community = Communities::community_account(&1);
			assert_ok!(Balances::mint_into(&community, 100 * UNITS));
			let community_before = Balances::balance(&community);

			// 1. Register the pass account on behalf of the community.
			//
			// `SkipIfRootOrCommunity` short-circuits the `RegistrarConsideration`, so the
			// community is charged nothing at all for this.
			let account = pass::account([1u8; 32]);
			let (first_attestation, first_device) = attestation(&account, [10u8; 32]);
			assert_ok!(Communities::dispatch_as_account(
				RuntimeOrigin::signed(ALICE),
				Box::new(RuntimeCall::Pass(pallet_pass::Call::register {
					user: [1u8; 32],
					attestation: first_attestation,
				}))
			));
			assert_eq!(
				Balances::balance_on_hold(&AccountRegistrationReason::get(), &community),
				0
			);
			assert_eq!(Balances::balance(&community), community_before);
			assert_eq!(Balances::balance(&account), 0);

			// 2. Fund it, then add the proxy and the second device.
			Balances::transfer(&community, &account, funding, Preservation::Preserve)?;

			Proxy::add_proxy(
				RuntimeOrigin::signed(account.clone()),
				CommunityLookup::unlookup(CONTROLLER),
				ProxyType::Any,
				0,
			)?;

			AuthenticatedDevice::put((account.clone(), first_device));
			let (second, _) = attestation(&account, [11u8; 32]);
			Pass::add_device(
				RuntimeOrigin::signed(account.clone()),
				second,
				pallet_pass::DeviceFilterOf::<Runtime>::Admin,
			)?;

			// Both devices stay registered.
			assert_eq!(pallet_pass::Devices::<Runtime>::iter_key_prefix(&account).count(), 2);

			Ok((
				Balances::balance(&account),
				<Balances as ReservableCurrency<AccountId>>::reserved_balance(&account),
				Balances::balance_on_hold(&AccountDevicesReason::get(), &account),
			))
		})
	}

	// Binary search the smallest transfer that lets the whole flow through.
	let (mut lo, mut hi) = (0u128, 10 * UNITS);
	assert!(attempt(hi).is_ok(), "upper bound must succeed");
	while lo + 1 < hi {
		let mid = lo + (hi - lo) / 2;
		if attempt(mid).is_ok() {
			hi = mid;
		} else {
			lo = mid;
		}
	}
	let minimum = hi;

	let (free, reserved, device_hold) = attempt(minimum).expect("minimum succeeds; qed");
	println!("minimum funding:       {minimum} planck");
	println!("  proxy reserve:       {PROXY_DEPOSIT}");
	println!("  second device hold:  {device_hold}");
	println!("  existential deposit: {EXISTENTIAL_DEPOSIT}");
	println!("  leftover free:       {free} (reserved total {reserved})");
	println!("one planck short:      {:?}", attempt(minimum - 1).unwrap_err());

	// The proxy reserve is the only cost: `SecondItemIsFree` does not charge for a
	// set of two devices, and `Balances` checks the existential deposit against the
	// *total* balance, so the reserve alone keeps the account alive at zero free.
	assert_eq!(device_hold, 0);
	assert_eq!(free, 0);
	assert_eq!(reserved, PROXY_DEPOSIT);
	assert_eq!(minimum, PROXY_DEPOSIT);
	assert!(PROXY_DEPOSIT > EXISTENTIAL_DEPOSIT);

	#[cfg(not(feature = "paseo"))]
	assert_eq!(minimum, 44_333_289);
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

/// What a pass account *without* a community membership pays in transaction
/// fees to add a proxy controller and rotate its device.
///
/// With a membership, `ChargeGasTxPayment`'s `MembershipsGasTank` makes these
/// feeless; without one it falls through to `ChargeAssetTxPayment`, which
/// charges in KSM. Fees are dominated by length, not weight: `TransactionByteFee`
/// is `10 * MILLICENTS` per byte and a pass-authenticated extrinsic carries a
/// ~200-byte `PassAuthenticate` credential.
// Builds the full `TransactionExtensions` tuple, which has no `PassAuthenticate` (or fee
// payment) under `zombienet`.
#[cfg(not(feature = "zombienet"))]
#[test]
fn ensure_unfunded_pass_account_transaction_fees_are_as_expected() {
	use frame_support::dispatch::GetDispatchInfo;
	use pass::{attestation, credential};

	use super::{
		Balance, ChargeTransaction, ProxyType, RuntimeCall, TransactionExtensions, TransactionPayment,
		UncheckedExtrinsic, MILLICENTS,
	};

	/// Encodes `call` as the General (origin-from-extension) extrinsic a pass
	/// account submits, and prices it.
	fn quote(call: RuntimeCall, device_id: pass::DeviceId, cred: pass::PassCredential) -> (usize, Balance) {
		let extensions: TransactionExtensions = (
			pallet_pass::PassAuthenticate::<Runtime>::from(device_id, cred),
			frame_system::CheckNonZeroSender::new(),
			frame_system::CheckSpecVersion::new(),
			frame_system::CheckTxVersion::new(),
			frame_system::CheckGenesis::new(),
			frame_system::CheckEra::from(sp_runtime::generic::Era::Immortal),
			frame_system::CheckNonce::from(0),
			frame_system::CheckWeight::new(),
			pallet_skip_feeless_payment::SkipCheckIfFeeless::from(ChargeTransaction::new(
				pallet_asset_tx_payment::ChargeAssetTxPayment::from(0, None),
			)),
		);

		let xt = UncheckedExtrinsic::new_transaction(call, extensions);
		let len = xt.encoded_size();
		let fee = TransactionPayment::compute_fee(len as u32, &xt.get_dispatch_info(), 0);
		(len, fee)
	}

	TestExternalities::default().execute_with(|| {
		const USER: [u8; 32] = [1u8; 32];
		let account = pass::account(USER);
		let (_, device_id) = attestation(&account, [10u8; 32]);
		let cred = || credential(USER, [10u8; 32], &account.encode());

		// The fee multiplier starts at 1, so these are the floor values.
		assert_eq!(
			pallet_transaction_payment::NextFeeMultiplier::<Runtime>::get(),
			sp_runtime::FixedU128::from(1)
		);
		println!("TransactionByteFee: {} planck/byte", 10 * MILLICENTS);

		let (replacement, _) = attestation(&account, [11u8; 32]);
		let add_proxy = RuntimeCall::Proxy(pallet_proxy::Call::add_proxy {
			delegate: CommunityLookup::unlookup(AccountId32::new([9; 32])),
			proxy_type: ProxyType::Any,
			delay: 0,
		});
		let add_device = RuntimeCall::Pass(pallet_pass::Call::add_device {
			attestation: replacement,
			filter: pallet_pass::DeviceFilterOf::<Runtime>::Admin,
		});
		let remove_device = RuntimeCall::Pass(pallet_pass::Call::remove_device { device_id });

		let mut individual = 0;
		for (label, call) in [
			("add_proxy", add_proxy.clone()),
			("add_device", add_device.clone()),
			("remove_device", remove_device.clone()),
		] {
			let (len, fee) = quote(call, device_id, cred());
			println!("{label:<16} len={len:<4} fee={fee} planck");
			individual += fee;
		}

		let (batch_len, batch_fee) = quote(
			RuntimeCall::Utility(pallet_utility::Call::batch_all {
				calls: alloc::vec![add_proxy, add_device, remove_device],
			}),
			device_id,
			cred(),
		);
		println!("batch_all        len={batch_len:<4} fee={batch_fee} planck");
		println!("\nthree extrinsics: {individual} planck");
		println!(
			"one batch_all:    {batch_fee} planck (saves {})",
			individual - batch_fee
		);

		// Deposits, for comparison: proxy reserve + (on the released `FirstItemIsFree`
		// config) the second-device hold and the existential deposit it forces free.
		const DEPOSITS: Balance = 77_666_622;
		println!("\ndeposits:         {DEPOSITS} planck");
		println!("total (batched):  {} planck", DEPOSITS + batch_fee);

		#[cfg(not(feature = "paseo"))]
		{
			assert_eq!(batch_fee, 1_582_749_654);
			assert_eq!(individual, 3_013_621_011);
			// Fees are an order of magnitude above the deposits.
			assert!(batch_fee > 20 * DEPOSITS);
		}
	})
}
