//! Regression tests that pin what a pass account and an incoming XCM deposit cost: the KSM fee of a
//! reserve deposit, the deposits of a pass account (registration, devices, proxy), what a
//! community must fund a pass account with, and the transaction fees of an account without a
//! membership. Ported from the 0.18 line on frame-contrib 3.x (kreivo#487).

use super::*;

use super::pass;
use crate::config::system::CommunityLookup;
use crate::constants::currency::EXISTENTIAL_DEPOSIT;
use crate::{
	AccountId, Balances, Communities, CommunitiesManager, FungibleAssetLocation, TreasuryAccount, CENTS, UNITS,
};
use xcm_executor::{WeighedMessage, XcmExecutor};

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
		let expected_fee = <crate::WeightToFee as WeightToFeeT>::weight_to_fee(&weight);

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
			crate::Assets::balance(usd_asset_id, AccountId::new(BENEFICIARY)),
			USD_SENT
		);

		// Pin the number down for this exact shape of message (4 instructions, 2 assets).
		// The XCM instruction weights are measured now, with a proof size: 362_893 bytes. `WeightToFee`
		// charges the larger of the ref time and the proof size, and here it is the proof size
		// (362_893 * 11_757 planck per byte), not the ref time (15_504_890_000, about 478 M planck).
		#[cfg(not(feature = "paseo"))]
		{
			assert_eq!(weight.ref_time(), 15_504_890_000);
			assert_eq!(weight.proof_size(), 362_893);
			assert_eq!(expected_fee, 4_266_411_893);
		}
	})
}

#[test]
fn ensure_pass_and_proxy_deposits_are_as_expected() {
	use frame_support::traits::{
		fungible::{Inspect as _, InspectHold},
		ReservableCurrency,
	};
	use parity_scale_codec::MaxEncodedLen;
	use pass::attestation;

	use crate::{
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
		// `DeviceConsideration` is `FirstItemsAreFree<2>`, so devices #1 (the one added by
		// `register`) and #2 are free; every device after that is charged.
		pass::authenticated_as(&first, first_device);

		// `LinearStoragePrice<MILLICENTS, MILLICENTS / 10>` over the footprint
		// `FirstItemIsFree` hands down; the nesting shrinks both the count and the
		// (integer-divided) per-item size, so the totals below are not a clean
		// multiple of `max_encoded_len` of a device.
		let mel = pallet_pass::DeviceOf::<Runtime>::max_encoded_len();
		println!("size_of<PassDevice> (max encoded): {mel}");

		let mut held = Vec::new();
		for (n, seed) in [[11u8; 32], [12u8; 32], [13u8; 32]].into_iter().enumerate() {
			let (att, _) = attestation(&first, seed);
			assert_ok!(Pass::add_device(RuntimeOrigin::signed(first.clone()), att));

			let total = Balances::balance_on_hold(&AccountDevicesReason::get(), &first);
			println!(
				"device #{}: total held {total} planck (+{})",
				n + 2,
				total as i128 - held.last().copied().unwrap_or(0) as i128
			);
			held.push(total);
		}

		// Devices #1 (from `register`) and #2 are free: `FirstItemsAreFree<2>` charges the footprint
		// from the third on. Device #3 is charged through `new` and #4 through `update`; since
		// frame-contrib#162 both price the same footprint (the items beyond the free ones, with the
		// per-item size scaled to them), so the hold of #3 is lower than the one of #4. Before it,
		// `new` priced the first charged item on the whole size of a device (130 bytes, 4_666_623
		// planck for #3, more than the 4_599_957 of #4); now it prices `mel / 3` = 43 bytes.
		assert_eq!(held[0], 0);
		assert_eq!(held[1], MILLICENTS + (MILLICENTS / 10) * (mel / 3) as Balance);

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
			assert_eq!(held, alloc::vec![0, 1_766_652, 4_599_957]);
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
	use pass::attestation;

	use crate::{
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

			// A community, with ALICE as its admin.
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

			pass::authenticated_as(&account, first_device);
			let (second, _) = attestation(&account, [11u8; 32]);
			Pass::add_device(RuntimeOrigin::signed(account.clone()), second)?;

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

/// What a pass account *without* a community membership pays in transaction
/// fees to add a proxy controller and rotate its device.
///
/// With a membership, `ChargeGasTxPayment`'s `MembershipsGasTank` makes these
/// feeless; without one it falls through to `ChargeAssetTxPayment`, which
/// charges in KSM. Fees are dominated by length, not weight: `TransactionByteFee`
/// is `10 * MILLICENTS` per byte and a pass-authenticated extrinsic carries a
/// ~200-byte `PassAuthenticate` credential.
#[test]
fn ensure_unfunded_pass_account_transaction_fees_are_as_expected() {
	use frame_support::dispatch::GetDispatchInfo;
	use pass::{attestation, credential};

	use crate::{
		Balance, ChargeTransaction, ProxyType, RuntimeCall, TransactionExtensions, TransactionPayment,
		UncheckedExtrinsic, MILLICENTS,
	};

	/// Encodes `call` as the General (origin-from-extension) extrinsic a pass
	/// account submits, and prices it.
	fn quote(
		call: RuntimeCall,
		device_id: pass::DeviceId,
		cred: crate::config::system::PassCredential,
	) -> (usize, Balance) {
		let extensions: TransactionExtensions = TransactionExtensions::new((
			pallet_pass::PassAuthenticate::<Runtime>::from(device_id, cred.clone()),
			frame_system::CheckNonZeroSender::new(),
			frame_system::CheckSpecVersion::new(),
			frame_system::CheckTxVersion::new(),
			frame_system::CheckGenesis::new(),
			frame_system::CheckEra::from(sp_runtime::generic::Era::Immortal),
			frame_system::CheckNonce::from(0),
			frame_system::CheckWeight::new(),
			pallet_skip_feeless_payment::SkipCheckIfFeeless::from(ChargeTransaction::from(
				pallet_asset_tx_payment::ChargeAssetTxPayment::from(0, None),
			)),
		));

		let xt = UncheckedExtrinsic::new_transaction(call, extensions);
		let len = xt.encoded_size();
		let info = xt.get_dispatch_info();
		let fee = TransactionPayment::compute_fee(len as u32, &info, 0);
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

		// Deposits, for comparison: the proxy reserve. The second device is free (`FirstItemsAreFree<2>`),
		// and the reserve alone keeps the account alive (see the funding test).
		const DEPOSITS: Balance = 44_333_289;
		println!("\ndeposits:         {DEPOSITS} planck");
		println!("total (batched):  {} planck", DEPOSITS + batch_fee);

		#[cfg(not(feature = "paseo"))]
		{
			// Every transaction declares 2_825_425_000 ref time and 35_098 proof size for its extensions
			// (the pass step's 6_392 proof size and the usage-subscription payment step's 25_113), which
			// `WeightToFee` prices by proof size: 412_635_472 planck a transaction, apart from its
			// length (3_333_330 planck a byte) and its call.
			assert_eq!(batch_fee, 2_890_372_193);
			assert_eq!(individual, 5_092_308_433);
			// Fees are an order of magnitude above the deposits.
			assert!(batch_fee > 20 * DEPOSITS);
		}
	})
}
