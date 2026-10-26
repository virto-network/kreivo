//! `pallet_pass`'s `MigrateV0ToV1` as Kreivo registers it (`ReleaseSteppedMigrations`).
//!
//! Version 0 had no per-device call filters, and `SessionKeys` held `(account, until)`. frame-contrib
//! 3.x denies a device that has no filter, so, until the migration reaches it, an existing device
//! cannot sign.

use super::*;

use crate::{
	config::system::{KreivoChallenger, MigrationsMaxServiceWeight, PassCredential, PassPalletId},
	BlockNumber, Pass, RuntimeCall, System,
};
use frame_contrib_traits::authn::{util::AuthorityFromPalletId, Challenger};
use frame_support::{
	dispatch::GetDispatchInfo,
	migrations::SteppedMigration,
	pallet_prelude::StorageVersion,
	storage::unhashed,
	traits::{Get, Hooks},
	weights::{Weight, WeightMeter},
	BoundedBTreeSet,
};
use pallet_pass::{migrations::MigrateV0ToV1, DeviceFilter, DeviceFilters, PassAuthenticate, SessionKeys};
use pass_substrate_keys::{KeySignature, SignedMessage};
use sp_core::crypto::AccountId32 as Account;
use sp_core::{sr25519, Pair};
use sp_io::hashing::blake2_256;
use sp_runtime::{
	traits::{TransactionExtension, TxBaseImplication},
	transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidityError},
	MultiSignature,
};

use super::pass;

type Migration = MigrateV0ToV1<Runtime>;

const ALICE: Account = Account::new([1; 32]);
const USER: [u8; 32] = [1u8; 32];
const SEED: [u8; 32] = [10u8; 32];

fn call() -> RuntimeCall {
	RuntimeCall::System(frame_system::Call::remark {
		remark: b"hello".to_vec(),
	})
}

/// A credential of the device made from `SEED`, for `call()`, as a wallet would sign it.
fn credential_for(call: &RuntimeCall) -> PassCredential {
	let pair = sr25519::Pair::from_seed(&SEED);
	let context: BlockNumber = System::block_number();
	// What the extension hands the challenger: the hash of the implication of the transaction.
	let xtc = TxBaseImplication(call.clone()).using_encoded(blake2_256);
	let message = SignedMessage {
		context,
		challenge: KreivoChallenger::generate(&context, &xtc),
		authority_id: AuthorityFromPalletId::<PassPalletId>::get(),
	};
	let signature = MultiSignature::Sr25519(pair.sign(message.message().as_ref()));
	PassCredential::SubstrateKey(KeySignature {
		user_id: USER,
		message,
		signature,
	})
}

/// Signs `call()` with the device through the real extension.
fn sign(device_id: pass::DeviceId) -> Result<(), TransactionValidityError> {
	let call = call();
	let extension = PassAuthenticate::<Runtime>::from(device_id, credential_for(&call));
	let info = call.get_dispatch_info();
	extension
		.validate(
			RuntimeOrigin::none(),
			&call,
			&info,
			0,
			(),
			&TxBaseImplication(call.clone()),
			TransactionSource::External,
		)
		.map(|_| ())
}

/// The state before the upgrade: a registered device that has no filter, at storage version 0.
fn before_the_upgrade() -> pass::DeviceId {
	System::set_block_number(1);
	assert_ok!(Balances::mint_into(&ALICE, UNITS));
	let account = pass::account(USER);
	let (attestation, device_id) = pass::attestation(&account, SEED);
	assert_ok!(Pass::register(RuntimeOrigin::signed(ALICE), USER, attestation));

	let _ = DeviceFilters::<Runtime>::clear_prefix(&account, u32::MAX, None);
	StorageVersion::new(0).put::<Pass>();
	device_id
}

/// Runs the migration to completion, one block's worth (`MaxServiceWeight`) at a time. Returns how
/// many steps it took.
fn migrate() -> u32 {
	let mut cursor = None;
	let mut steps = 0;
	loop {
		let mut meter = WeightMeter::with_limit(MigrationsMaxServiceWeight::get());
		steps += 1;
		cursor = Migration::step(cursor, &mut meter).expect("it does not fail");
		if cursor.is_none() {
			return steps;
		}
		assert!(steps < 10_000);
	}
}

// A device created before the upgrade cannot sign until the migration reaches it, and signs after.
#[test]
fn a_device_from_before_the_upgrade_signs_once_it_is_migrated() {
	TestExternalities::default().execute_with(|| {
		let device_id = before_the_upgrade();

		assert_eq!(sign(device_id), Err(InvalidTransaction::Call.into()));

		migrate();

		assert_ok!(sign(device_id));
		assert_eq!(
			DeviceFilters::<Runtime>::get(pass::account(USER), device_id),
			Some(DeviceFilter::Admin)
		);
		assert_eq!(StorageVersion::get::<Pass>(), 1);
	})
}

// A session key of version 0, `(account, until)`, becomes `(account, until, Admin)`.
#[test]
fn a_session_key_of_version_zero_gains_the_admin_filter() {
	TestExternalities::default().execute_with(|| {
		let _ = before_the_upgrade();
		let account = pass::account(USER);
		let session = Account::new([42; 32]);
		let until: BlockNumber = 1_000;

		let key = SessionKeys::<Runtime>::hashed_key_for(&session);
		unhashed::put(&key, &(account.clone(), until));

		migrate();

		assert_eq!(
			SessionKeys::<Runtime>::get(&session),
			Some((account, until, DeviceFilter::Admin))
		);
	})
}

// A second run changes nothing, and a session key that was already converted keeps its filter.
#[test]
fn running_the_migration_again_changes_nothing() {
	TestExternalities::default().execute_with(|| {
		let device_id = before_the_upgrade();
		let account = pass::account(USER);
		let session = Account::new([42; 32]);
		unhashed::put(
			&SessionKeys::<Runtime>::hashed_key_for(&session),
			&(account.clone(), 1_000u64),
		);
		migrate();
		// A device that gained a restricted filter since keeps it.
		let restricted = DeviceFilter::Pallets(
			BoundedBTreeSet::try_from(alloc::collections::BTreeSet::from([0u8])).expect("one fits"),
		);
		DeviceFilters::<Runtime>::insert(&account, device_id, restricted.clone());
		let filters = DeviceFilters::<Runtime>::iter().collect::<Vec<_>>();
		let sessions = SessionKeys::<Runtime>::iter().collect::<Vec<_>>();

		migrate();

		assert_eq!(DeviceFilters::<Runtime>::iter().collect::<Vec<_>>(), filters);
		assert_eq!(SessionKeys::<Runtime>::iter().collect::<Vec<_>>(), sessions);
		assert_eq!(DeviceFilters::<Runtime>::get(&account, device_id), Some(restricted));
	})
}

// A unit that does not fit the meter yields; none fits an empty tiny one.
#[test]
fn the_migration_yields_and_fails_only_when_nothing_fits_an_empty_meter() {
	use frame_support::migrations::SteppedMigrationError;

	TestExternalities::default().execute_with(|| {
		let _ = before_the_upgrade();

		let mut meter = WeightMeter::with_limit(Weight::from_parts(1, 1));
		assert!(matches!(
			Migration::step(None, &mut meter),
			Err(SteppedMigrationError::InsufficientWeight { .. })
		));

		// Not empty, but out of weight: it yields with the cursor it had.
		let mut meter = WeightMeter::with_limit(MigrationsMaxServiceWeight::get());
		assert_ok!(meter.try_consume(MigrationsMaxServiceWeight::get()).map_err(|_| ()));
		assert!(matches!(Migration::step(None, &mut meter), Ok(Some(_))));
	})
}

// Pass's own integrity test, which checks that the migration's cursor fits what the runtime stores.
#[test]
fn the_pass_integrity_test_holds() {
	TestExternalities::default().execute_with(|| {
		<Pass as Hooks<_>>::integrity_test();
	});
}
