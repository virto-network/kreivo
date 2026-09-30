//! What happens to a membership's gas tank, and to the rest of its attributes, as members use it.
//!
//! These tests document the **current** behaviour of the 0.17 runtime configuration
//! (`MembershipsGasTank`, `CopySystemAttributesOnAssign`, the transaction extensions), so they
//! pass today. Where the current behaviour is a bug, the test asserts it anyway and marks it with
//! a `// BUG:` comment explaining what is expected instead, so it can be flipped once fixed.
//!
//! A tank is the `membership_gas` item attribute (a `WeightTank`) of a membership its payer owns.
//! Using it writes `mbmshp_pays_gas` (the gas left, before dispatch) and clears it again after
//! dispatch, adding what the call used to the tank.

use super::*;

use crate::{
	config::{communities::MembershipsCollectionId, currency::MembershipsGasTank, RelaychainData},
	Balance, BlockNumber, ChargeTransaction, CheckedExtrinsic, RuntimeCall, RuntimeEvent, System,
	TransactionExtensions,
};
use core::fmt;
use frame_contrib_traits::gas_tank::{GasBurner, MakeTank};
use frame_support::{
	dispatch::GetDispatchInfo,
	traits::{fungible::Inspect as _, nonfungibles_v2::Mutate as _},
	weights::Weight,
};
use pallet_nfts::AttributeNamespace;
use parity_scale_codec::{Decode, DecodeAll};
use sp_runtime::{
	generic::{Era, ExtrinsicFormat},
	traits::{Applyable, BlockNumberProvider},
};
use std::collections::BTreeMap;

type Attribute = pallet_nfts::Attribute<Runtime, CommunityMembershipsInstance>;

const ALICE: AccountId32 = AccountId32::new([1; 32]);
const BOB: AccountId32 = AccountId32::new([2; 32]);
const CHARLIE: AccountId32 = AccountId32::new([3; 32]);

const COMMUNITY: u16 = 1;
const MANAGER: u16 = MembershipsCollectionId::get();

/// Where `NonFungiblesMemberships::assign` parks the manager item of an assigned membership
/// (`fc_traits_memberships::ASSIGNED_MEMBERSHIPS_ACCOUNT`, which isn't re-exported).
const ASSIGNED_MEMBERSHIPS_ACCOUNT: [u8; 32] = sp_runtime::str_array("memberships/assigned_memberships");

const GAS: &[u8] = b"membership_gas";
const PAYS_GAS: &[u8] = b"mbmshp_pays_gas";
const RANK: &[u8] = b"membership_member_rank";
/// `pallet_communities_manager` and `MembershipIsNotExpired` use `&b"membership_expiration"`, a
/// byte *array*, as the typed key: it encodes without a length prefix, unlike the other keys
/// (byte slices).
const EXPIRATION: [u8; 21] = *b"membership_expiration";

/// Mirrors `fc_traits_gas_tank::WeightTank`, whose fields aren't public.
#[derive(Decode, Clone, PartialEq, Eq)]
struct Tank {
	since: BlockNumber,
	used: Weight,
	period: Option<BlockNumber>,
	capacity_per_period: Option<Weight>,
}

impl fmt::Debug for Tank {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"Tank {{ since: {}, used: {}, period: {:?}, capacity: {:?} }}",
			self.since,
			self.used.ref_time(),
			self.period,
			self.capacity_per_period.map(|c| c.ref_time())
		)
	}
}

fn tank(collection: u16, item: u32) -> Option<Tank> {
	CommunityMemberships::typed_system_attribute(&collection, Some(&item), &GAS)
}

fn owner(collection: &u16, item: &u32) -> Option<AccountId> {
	<CommunityMemberships as Inspect<_>>::owner(collection, item)
}

fn pays_gas(collection: u16, item: u32) -> Option<Weight> {
	CommunityMemberships::typed_system_attribute(&collection, Some(&item), &PAYS_GAS)
}

/// Every attribute stored for an item, in any namespace, as `"namespace/key" => value`.
///
/// Reads the storage map directly (not through `Inspect`), so it also shows attributes left
/// behind by burnt items.
fn attributes(collection: u16, item: u32) -> BTreeMap<String, String> {
	Attribute::iter_prefix((collection, Some(item)))
		.map(|((namespace, key), (value, _))| {
			let namespace = match namespace {
				AttributeNamespace::Pallet => "Pallet".to_string(),
				other => format!("{other:?}"),
			};
			// Typed attribute keys are SCALE-encoded: byte slices get a length prefix, byte arrays
			// don't. Keys without the prefix are shown with a `[raw]` suffix.
			let (raw, suffix) = match Vec::<u8>::decode_all(&mut &key[..]) {
				Ok(raw) => (raw, ""),
				Err(_) => (key.to_vec(), "[raw]"),
			};
			let name = String::from_utf8(raw.clone()).unwrap_or(format!("0x{}", hex(&raw)));
			let value = match raw.as_slice() {
				GAS => format!("{:?}", Tank::decode(&mut &value[..]).expect("a tank; qed")),
				PAYS_GAS => format!(
					"remaining {}",
					Weight::decode(&mut &value[..]).expect("a weight; qed").ref_time()
				),
				b"membership_expiration" => format!(
					"relay block {}",
					BlockNumber::decode(&mut &value[..]).expect("a block number; qed")
				),
				_ => format!("0x{}", hex(&value)),
			};
			(format!("{namespace}/{name}{suffix}"), value)
		})
		.collect()
}

fn hex(bytes: &[u8]) -> String {
	bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn keys(attributes: &BTreeMap<String, String>) -> Vec<&str> {
	attributes.keys().map(String::as_str).collect()
}

fn show(step: &str, collection: u16, item: u32) -> BTreeMap<String, String> {
	let attributes = attributes(collection, item);
	let owner = owner(&collection, &item);
	println!("[{step}] item ({collection}, {item}) owner: {owner:?}");
	for (key, value) in &attributes {
		println!("    {key} = {value}");
	}
	attributes
}

fn at_relay_block(n: BlockNumber) {
	RelaychainData::set_block_number(n);
}

fn remark() -> RuntimeCall {
	frame_system::Call::remark {
		remark: b"gas tank".to_vec(),
	}
	.into()
}

/// The weight a remark estimates, the unit tanks are sized in here.
fn remark_weight() -> Weight {
	remark().get_dispatch_info().call_weight
}

fn tank_config(capacity: Weight, period: BlockNumber) -> TankConfig<Weight, BlockNumber> {
	// `TankConfig`'s fields are private: it can only be decoded, as it is from an extrinsic.
	TankConfig::decode(&mut &(Some(capacity), Some(period)).encode()[..]).expect("valid encoding; qed")
}

/// The runtime's transaction extensions, as a wallet would build them.
fn tx_ext(who: &AccountId) -> TransactionExtensions {
	TransactionExtensions::new((
		pallet_pass::PassAuthenticate::default(),
		frame_system::CheckNonZeroSender::new(),
		frame_system::CheckSpecVersion::new(),
		frame_system::CheckTxVersion::new(),
		frame_system::CheckGenesis::new(),
		frame_system::CheckEra::from(Era::Immortal),
		frame_system::CheckNonce::from(System::account_nonce(who)),
		frame_system::CheckWeight::new(),
		pallet_skip_feeless_payment::SkipCheckIfFeeless::from(ChargeTransaction::new(
			pallet_asset_tx_payment::ChargeAssetTxPayment::from(0, None),
		)),
	))
}

#[derive(Debug, PartialEq)]
enum Paid {
	/// `GasTxPayment` burnt gas; this is the remaining gas it reported.
	WithGas(Weight),
	/// The inner `ChargeAssetTxPayment` charged this fee instead.
	WithFee(Balance),
}

/// Applies a signed `remark` from `who` the way block execution does once the signature has
/// been checked: the transaction extensions `validate`, then `prepare`, the call is dispatched,
/// then `post_dispatch`.
fn submit_remark(who: &AccountId) -> Paid {
	let call = remark();
	let info = call.get_dispatch_info();
	let len = call.encoded_size();
	let balance = Balances::balance(who);
	System::reset_events();

	let xt = CheckedExtrinsic {
		format: ExtrinsicFormat::Signed(who.clone(), tx_ext(who)),
		function: call,
	};
	let result = xt.apply::<Runtime>(&info, len);
	assert!(
		matches!(result, Ok(Ok(_))),
		"the remark is applied and succeeds: {result:?}"
	);

	let burnt = System::events().into_iter().find_map(|record| match record.event {
		RuntimeEvent::GasTxPayment(pallet_gas_transaction_payment::Event::GasBurned { who: payer, remaining })
			if &payer == who =>
		{
			Some(remaining)
		}
		_ => None,
	});
	let fee = balance - Balances::balance(who);
	match burnt {
		Some(remaining) => {
			assert_eq!(fee, 0, "paying with gas charges no fee");
			Paid::WithGas(remaining)
		}
		None => {
			assert!(fee > 0, "without gas, the transaction pays a fee");
			Paid::WithFee(fee)
		}
	}
}

/// Registers community `COMMUNITY`, with `ALICE` as its admin and funds in its account.
fn register_community() {
	if cfg!(feature = "runtime-benchmarks") {
		assert_ok!(Balances::mint_into(
			&TreasuryAccount::get(),
			EXISTENTIAL_DEPOSIT + 10 * CENTS
		));
	}
	assert_ok!(Balances::mint_into(&ALICE, UNITS));
	assert_ok!(CommunitiesManager::register(
		RuntimeOrigin::root(),
		COMMUNITY,
		BoundedVec::try_from(b"Gas Tank Community".to_vec()).expect("meets max length; qed"),
		CommunityLookup::unlookup(ALICE),
		None,
		None,
	));
	assert_ok!(Balances::mint_into(&Communities::community_account(&COMMUNITY), UNITS));
}

fn new_test_ext() -> TestExternalities {
	let mut ext = TestExternalities::default();
	ext.execute_with(|| {
		// Events are only recorded after block 0.
		System::set_block_number(1);
		// Immortal transactions check the genesis block hash exists, as it does on chain.
		frame_system::BlockHash::<Runtime>::insert(0, crate::Hash::default());
		at_relay_block(1_000);
	});
	ext
}

/// 1. Using a tank for the first time keeps every attribute of the membership: it only adds
///    to `membership_gas.used`, and `mbmshp_pays_gas` comes and goes.
mod first_use {
	use super::*;

	/// `ALICE` gets membership `(COMMUNITY, 7)` straight in the community collection, carrying a
	/// tank for 5 remarks per 100 relay chain blocks, a rank and an expiration.
	fn membership_with_tank() {
		register_community();
		assert_ok!(
			<CommunityMemberships as frame_support::traits::nonfungibles_v2::Mutate<_, _>>::mint_into(
				&COMMUNITY,
				&7,
				&ALICE,
				&Default::default(),
				true
			)
		);
		assert_ok!(MembershipsGasTank::make_tank(
			&(COMMUNITY, 7),
			Some(remark_weight() * 5),
			Some(100)
		));
		assert_ok!(CommunityMemberships::set_typed_attribute(&COMMUNITY, &7, &RANK, &3u8));
		assert_ok!(CommunityMemberships::set_typed_attribute(
			&COMMUNITY,
			&7,
			&EXPIRATION,
			&(1_000 + 8 * WEEKS)
		));
	}

	#[test]
	fn through_a_signed_transaction() {
		new_test_ext().execute_with(|| {
			membership_with_tank();
			let before = show("before the first use", COMMUNITY, 7);
			assert_eq!(
				keys(&before),
				[
					"Pallet/membership_expiration[raw]",
					"Pallet/membership_gas",
					"Pallet/membership_member_rank"
				]
			);
			let tank_before = tank(COMMUNITY, 7).expect("the membership has a tank");
			assert_eq!(tank_before.used, Weight::zero());

			at_relay_block(1_010);
			let paid = submit_remark(&ALICE);
			let after = show("after the first use", COMMUNITY, 7);

			// It was paid with gas, not with a fee.
			assert_eq!(paid, Paid::WithGas(remark_weight() * 4));
			// Every attribute is still there, and only the tank changed.
			assert_eq!(keys(&after), keys(&before), "no attribute was deleted");
			assert_eq!(
				after["Pallet/membership_member_rank"],
				before["Pallet/membership_member_rank"]
			);
			assert_eq!(
				after["Pallet/membership_expiration[raw]"],
				before["Pallet/membership_expiration[raw]"]
			);
			assert_eq!(pays_gas(COMMUNITY, 7), None, "`mbmshp_pays_gas` is cleared");
			assert_eq!(
				tank(COMMUNITY, 7),
				Some(Tank {
					used: remark_weight(),
					..tank_before
				}),
				"only `used` moves"
			);
			assert_eq!(owner(&COMMUNITY, &7), Some(ALICE), "the item still exists");

			// A second use keeps adding up.
			assert_eq!(submit_remark(&ALICE), Paid::WithGas(remark_weight() * 3));
			assert_eq!(tank(COMMUNITY, 7).map(|t| t.used), Some(remark_weight() * 2));
			assert_eq!(keys(&show("after the second use", COMMUNITY, 7)), keys(&before));
		})
	}

	#[test]
	fn through_the_gas_burner() {
		new_test_ext().execute_with(|| {
			membership_with_tank();
			let before = show("before the first use", COMMUNITY, 7);
			let tank_before = tank(COMMUNITY, 7).expect("the membership has a tank");
			let weight = remark_weight();

			at_relay_block(1_010);
			let remaining = MembershipsGasTank::check_available_gas(&ALICE, &weight);
			let checked = show("after check_available_gas", COMMUNITY, 7);
			assert_eq!(remaining, Some(weight * 4));
			assert_eq!(
				pays_gas(COMMUNITY, 7),
				remaining,
				"the pre-dispatch check leaves a note"
			);
			assert_eq!(
				keys(&checked),
				[
					"Pallet/mbmshp_pays_gas",
					"Pallet/membership_expiration[raw]",
					"Pallet/membership_gas",
					"Pallet/membership_member_rank"
				]
			);

			let left = MembershipsGasTank::burn_gas(&ALICE, &(weight * 4), &weight);
			let after = show("after burn_gas", COMMUNITY, 7);
			assert_eq!(left, weight * 4);
			assert_eq!(pays_gas(COMMUNITY, 7), None, "the note is cleared, and only the note");
			assert_eq!(keys(&after), keys(&before), "no attribute was deleted");
			assert_eq!(
				tank(COMMUNITY, 7),
				Some(Tank {
					used: weight,
					..tank_before
				})
			);
		})
	}

	/// `TankConfig::default()` (what memberships have been created with so far) is an unlimited
	/// tank. It pays for anything, and using it writes nothing at all.
	#[test]
	fn of_an_unlimited_tank() {
		new_test_ext().execute_with(|| {
			membership_with_tank();
			assert_ok!(MembershipsGasTank::make_tank(&(COMMUNITY, 7), None, None));
			let before = show("unlimited, before the first use", COMMUNITY, 7);

			// The burner reports no remaining gas, since an unlimited tank doesn't track it.
			assert_eq!(submit_remark(&ALICE), Paid::WithGas(Weight::zero()));
			let after = show("unlimited, after the first use", COMMUNITY, 7);
			assert_eq!(after, before, "nothing changes");
		})
	}
}

/// 2. The lifecycle as users go through it: the community buys a membership from the manager
///    collection, and gives it to a new member.
mod lifecycle {
	use super::*;

	/// Memberships `(MANAGER, 0..3)`, 3 remarks every `period` relay chain blocks, expiring 8
	/// weeks from now; the community buys `(MANAGER, 0)`, and adds `BOB` as a member.
	pub(super) fn bob_is_a_member(period: BlockNumber) {
		register_community();
		assert_ok!(CommunitiesManager::create_memberships(
			RuntimeOrigin::root(),
			3,
			0,
			CENTS,
			tank_config(remark_weight() * 3, period),
			Some(1_000 + 8 * WEEKS),
		));
		show("created", MANAGER, 0);
		assert_eq!(owner(&MANAGER, &0), Some(TreasuryAccount::get()));

		assert_ok!(Communities::dispatch_as_account(
			RuntimeOrigin::signed(ALICE),
			Box::new(
				pallet_nfts::Call::<Runtime, CommunityMembershipsInstance>::buy_item {
					collection: MANAGER,
					item: 0,
					bid_price: CENTS
				}
				.into()
			)
		));
		show("bought by the community", MANAGER, 0);
		assert_eq!(owner(&MANAGER, &0), Some(Communities::community_account(&COMMUNITY)));

		assert_ok!(Balances::mint_into(&BOB, UNITS));
		assert_ok!(Communities::add_member(
			RuntimeOrigin::signed(ALICE),
			CommunityLookup::unlookup(BOB)
		));
	}

	#[test]
	fn members_get_a_copy_of_the_tank_and_use_it() {
		new_test_ext().execute_with(|| {
			bob_is_a_member(100);
			let manager = show("manager item, after add_member", MANAGER, 0);
			let member = show("member item, after add_member", COMMUNITY, 0);

			// The manager item stays behind, parked in the assigned memberships account.
			assert_eq!(owner(&MANAGER, &0), Some(AccountId::from(ASSIGNED_MEMBERSHIPS_ACCOUNT)));
			assert_eq!(
				keys(&manager),
				["Pallet/membership_expiration[raw]", "Pallet/membership_gas"]
			);
			// The member gets a new item, and `CopySystemAttributesOnAssign` copies the tank onto
			// it. So, contrary to the suspicion, members do get gas.
			assert_eq!(owner(&COMMUNITY, &0), Some(BOB));
			// BUG: the expiration isn't copied. The hook looks the well-known keys up as byte
			// slices (length-prefixed), but `create_memberships` stores the expiration under a byte
			// array key (no prefix), which is also how `MembershipIsNotExpired` reads it. Expected:
			// `Pallet/membership_expiration[raw]` here too.
			assert_eq!(
				keys(&member),
				["Pallet/membership_gas", "Pallet/membership_member_rank"]
			);
			assert_eq!(member["Pallet/membership_gas"], manager["Pallet/membership_gas"]);
			let copied = tank(COMMUNITY, 0).expect("the member's item has a tank");
			assert_eq!(
				copied.since, 1_000,
				"the copy keeps the creation block, not the assignment's"
			);

			at_relay_block(1_010);
			assert_eq!(submit_remark(&BOB), Paid::WithGas(remark_weight() * 2));
			let member_after = show("member item, after BOB's first transaction", COMMUNITY, 0);
			let manager_after = show("manager item, after BOB's first transaction", MANAGER, 0);

			// BOB spends from his own copy; the manager's tank is untouched.
			assert_eq!(keys(&member_after), keys(&member), "no attribute was deleted");
			assert_eq!(
				tank(COMMUNITY, 0),
				Some(Tank {
					used: remark_weight(),
					..copied.clone()
				})
			);
			assert_eq!(manager_after, manager);

			// Until the tank runs dry, after which BOB pays fees.
			assert_eq!(submit_remark(&BOB), Paid::WithGas(remark_weight()));
			assert_eq!(submit_remark(&BOB), Paid::WithGas(Weight::zero()));
			assert!(matches!(submit_remark(&BOB), Paid::WithFee(_)));
			assert_eq!(tank(COMMUNITY, 0).map(|t| t.used), Some(remark_weight() * 3));
			assert_eq!(keys(&show("member item, tank exhausted", COMMUNITY, 0)), keys(&member));
		})
	}

	/// The expiration of a membership should limit its tank too.
	#[test]
	fn expired_memberships_keep_paying_with_gas() {
		new_test_ext().execute_with(|| {
			// A period longer than the membership, so the tank doesn't reset (see `period_reset`).
			bob_is_a_member(12 * WEEKS);
			at_relay_block(1_000 + 8 * WEEKS + 1);

			// The manager item has expired...
			assert!(!crate::config::currency::MembershipIsNotExpired::get().select(MANAGER, 0));
			// BUG: ...but the member's item has no expiration (see above), so it never expires,
			// and BOB keeps paying with gas. Expected: `Paid::WithFee(_)`.
			assert!(crate::config::currency::MembershipIsNotExpired::get().select(COMMUNITY, 0));
			assert_eq!(submit_remark(&BOB), Paid::WithGas(remark_weight() * 2));
		})
	}
}

/// 3. A tank renews every `period` relay chain blocks.
mod period_reset {
	use super::*;

	/// `ALICE` owns `(COMMUNITY, 7)`, with a tank for 5 remarks every 10 relay chain blocks, made
	/// at relay chain block 1_000.
	fn tank_of_ten_blocks() {
		register_community();
		assert_ok!(
			<CommunityMemberships as frame_support::traits::nonfungibles_v2::Mutate<_, _>>::mint_into(
				&COMMUNITY,
				&7,
				&ALICE,
				&Default::default(),
				true
			)
		);
		assert_ok!(MembershipsGasTank::make_tank(
			&(COMMUNITY, 7),
			Some(remark_weight() * 5),
			Some(10)
		));
	}

	#[test]
	fn resetting_moves_the_start_a_whole_period_into_the_future() {
		new_test_ext().execute_with(|| {
			tank_of_ten_blocks();
			let weight = remark_weight();
			let check = || MembershipsGasTank::check_available_gas(&ALICE, &weight);

			// Within the first period (up to `since + period`), no reset.
			at_relay_block(1_010);
			assert_eq!(check(), Some(weight * 4));
			assert_eq!(tank(COMMUNITY, 7).map(|t| t.since), Some(1_000));

			// Past it, the tank resets...
			at_relay_block(1_011);
			assert_eq!(check(), Some(weight * 4));
			show("after the reset at relay block 1_011", COMMUNITY, 7);
			// BUG: the new period should start now (`since = 1_011`), but it starts a period
			// later, at `now + period`.
			assert_eq!(tank(COMMUNITY, 7).map(|t| t.since), Some(1_021));

			// ...so `now - since` underflows, and the tank has no gas until then, not even right
			// after resetting it, in the same block.
			// BUG: expected `Some(weight * 4)` in all of these.
			assert_eq!(check(), None, "same block as the reset");
			for block in 1_012..1_021 {
				at_relay_block(block);
				assert_eq!(check(), None, "relay chain block {block}");
			}
			at_relay_block(1_021);
			assert_eq!(check(), Some(weight * 4));
		})
	}

	/// In a transaction, `validate` and `prepare` both check the tank. When `validate` resets
	/// it, `prepare` finds no gas, falls back to the inner fee payment, and panics expecting the
	/// value the inner extension would have returned from `validate` (which never ran).
	#[test]
	fn a_transaction_that_resets_the_tank_panics() {
		new_test_ext().execute_with(|| {
			tank_of_ten_blocks();
			at_relay_block(1_005);
			assert_eq!(submit_remark(&ALICE), Paid::WithGas(remark_weight() * 4));

			// Past the first period, as block building does: apply the transaction in a storage
			// transaction, and roll it back if applying it fails.
			at_relay_block(1_011);
			let apply = || {
				sp_io::storage::start_transaction();
				let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| submit_remark(&ALICE)));
				sp_io::storage::rollback_transaction();
				let panic = result.expect_err("applying the transaction panics");
				panic
					.downcast_ref::<String>()
					.cloned()
					.or(panic.downcast_ref::<&str>().map(|s| s.to_string()))
			};

			// BUG: `validate` resets the tank and picks the gas path, `prepare` then finds no gas
			// and panics. Expected: the tank resets and pays for the transaction.
			let message = apply();
			println!("panic: {message:?}");
			assert_eq!(message.as_deref(), Some("value was given on validate; qed"));

			// BUG: the rollback undoes the reset too, so the tank never resets, and every later
			// transaction from ALICE (that fits in the tank) panics the same way, whether or not
			// she could pay a fee: she can't transact at all until she stops owning the membership
			// (or it expires). The pool keeps accepting them, since `validate` alone succeeds.
			show("after the rolled back transaction", COMMUNITY, 7);
			assert_eq!(
				tank(COMMUNITY, 7).map(|t| (t.since, t.used)),
				Some((1_000, remark_weight()))
			);
			for block in [1_012, 1_050, 2_000] {
				at_relay_block(block);
				assert_eq!(
					apply().as_deref(),
					Some("value was given on validate; qed"),
					"relay chain block {block}"
				);
			}
		})
	}
}

/// 4. Removing a member.
mod removal {
	use super::*;

	#[test]
	fn the_member_item_is_burnt_but_its_attributes_stay() {
		new_test_ext().execute_with(|| {
			lifecycle::bob_is_a_member(100);
			at_relay_block(1_010);
			assert_eq!(submit_remark(&BOB), Paid::WithGas(remark_weight() * 2));
			let member = show("member item, before remove_member", COMMUNITY, 0);
			let manager = show("manager item, before remove_member", MANAGER, 0);

			assert_ok!(Communities::remove_member(
				RuntimeOrigin::signed(ALICE),
				CommunityLookup::unlookup(BOB),
				0
			));
			let member_after = show("member item, after remove_member", COMMUNITY, 0);
			let manager_after = show("manager item, after remove_member", MANAGER, 0);

			// BOB's item is gone, so he has no gas left.
			assert_eq!(owner(&COMMUNITY, &0), None);
			assert_eq!(MembershipsGasTank::check_available_gas(&BOB, &remark_weight()), None);
			assert!(matches!(submit_remark(&BOB), Paid::WithFee(_)));

			// BUG: `pallet_nfts` doesn't clear attributes when burning an item, so the burnt
			// member item's tank (with what BOB used), expiration and rank stay in storage.
			// Expected: `release` clears them.
			assert_eq!(keys(&member_after), keys(&member));
			assert_eq!(member_after["Pallet/membership_gas"], member["Pallet/membership_gas"]);
			assert_eq!(tank(COMMUNITY, 0).map(|t| t.used), Some(remark_weight()));

			// The manager item goes back to the community, with its tank untouched.
			assert_eq!(owner(&MANAGER, &0), Some(Communities::community_account(&COMMUNITY)));
			assert_eq!(manager_after, manager);
		})
	}

	#[test]
	fn the_next_member_gets_a_fresh_copy_of_the_tank() {
		new_test_ext().execute_with(|| {
			lifecycle::bob_is_a_member(100);
			at_relay_block(1_010);
			assert_eq!(submit_remark(&BOB), Paid::WithGas(remark_weight() * 2));
			assert_ok!(Communities::remove_member(
				RuntimeOrigin::signed(ALICE),
				CommunityLookup::unlookup(BOB),
				0
			));

			assert_ok!(Balances::mint_into(&CHARLIE, UNITS));
			assert_ok!(Communities::add_member(
				RuntimeOrigin::signed(ALICE),
				CommunityLookup::unlookup(CHARLIE)
			));
			let member = show("member item, re-assigned to CHARLIE", COMMUNITY, 0);

			// Same membership id, minted again: the copy overwrites the tank BOB left behind.
			assert_eq!(owner(&COMMUNITY, &0), Some(CHARLIE));
			// BUG: still no expiration (see `lifecycle`).
			assert_eq!(
				keys(&member),
				["Pallet/membership_gas", "Pallet/membership_member_rank"]
			);
			assert_eq!(
				tank(COMMUNITY, 0),
				tank(MANAGER, 0),
				"a fresh copy of the manager's tank"
			);
			assert_eq!(submit_remark(&CHARLIE), Paid::WithGas(remark_weight() * 2));
		})
	}
}
