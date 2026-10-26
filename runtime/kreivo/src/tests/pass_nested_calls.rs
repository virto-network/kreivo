//! A device with a restricted filter cannot run a call its filter denies by wrapping it in a call
//! that the filter allows (`KreivoCallMatcher`, `pallet_pass` #159). Every test goes through the
//! real `PassAuthenticate` extension, with a credential of the device.

use super::*;

use crate::{
	config::{
		call_matcher::KreivoCallMatcher,
		system::{KreivoChallenger, PassCredential, PassPalletId},
	},
	BlockNumber, Pass, RuntimeCall, System,
};
use alloc::collections::BTreeSet;
use frame_contrib_traits::authn::{util::AuthorityFromPalletId, Challenger};
use frame_support::{dispatch::GetDispatchInfo, traits::Get, BoundedBTreeSet, BoundedVec};
use pallet_pass::{CallMatcher, DeviceFilter, DeviceFilters, PassAuthenticate};
use parity_scale_codec::Encode;
use pass_substrate_keys::{KeySignature, SignedMessage};
use sp_core::{sr25519, Pair};
use sp_io::hashing::blake2_256;
use sp_runtime::{
	traits::{TransactionExtension, TxBaseImplication},
	transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidityError},
	MultiSignature,
};

use super::pass;

const ALICE: AccountId32 = AccountId32::new([1; 32]);
const USER: [u8; 32] = [1u8; 32];
const SEED: [u8; 32] = [10u8; 32];

fn rejected() -> Result<(), TransactionValidityError> {
	Err(InvalidTransaction::Call.into())
}

// Calls to wrap.
fn remark() -> RuntimeCall {
	RuntimeCall::System(frame_system::Call::remark { remark: b"hi".to_vec() })
}

fn transfer_all() -> RuntimeCall {
	RuntimeCall::Balances(pallet_balances::Call::transfer_all {
		dest: CommunityLookup::unlookup(AccountId32::new([9; 32])),
		keep_alive: false,
	})
}

fn batch(calls: Vec<RuntimeCall>) -> RuntimeCall {
	RuntimeCall::Utility(pallet_utility::Call::batch { calls })
}

fn proxy(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Proxy(pallet_proxy::Call::proxy {
		real: CommunityLookup::unlookup(AccountId32::new([8; 32])),
		force_proxy_type: None,
		call: Box::new(call),
	})
}

fn as_multi_threshold_1(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Multisig(pallet_multisig::Call::as_multi_threshold_1 {
		other_signatories: vec![AccountId32::new([7; 32])],
		call: Box::new(call),
	})
}

fn dispatch_as(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Utility(pallet_utility::Call::dispatch_as {
		as_origin: Box::new(frame_system::RawOrigin::Root.into()),
		call: Box::new(call),
	})
}

fn schedule(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Scheduler(pallet_scheduler::Call::schedule {
		when: 100,
		maybe_periodic: None,
		priority: 0,
		call: Box::new(call),
	})
}

fn as_derivative(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Utility(pallet_utility::Call::as_derivative {
		index: 0,
		call: Box::new(call),
	})
}

fn dispatch_as_fallible(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Utility(pallet_utility::Call::dispatch_as_fallible {
		as_origin: Box::new(frame_system::RawOrigin::Root.into()),
		call: Box::new(call),
	})
}

fn with_weight(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Utility(pallet_utility::Call::with_weight {
		call: Box::new(call),
		weight: frame_support::weights::Weight::zero(),
	})
}

fn proxy_announced(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Proxy(pallet_proxy::Call::proxy_announced {
		delegate: CommunityLookup::unlookup(AccountId32::new([6; 32])),
		real: CommunityLookup::unlookup(AccountId32::new([8; 32])),
		force_proxy_type: None,
		call: Box::new(call),
	})
}

fn as_multi(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Multisig(pallet_multisig::Call::as_multi {
		threshold: 2,
		other_signatories: vec![AccountId32::new([7; 32])],
		maybe_timepoint: None,
		call: Box::new(call),
		max_weight: frame_support::weights::Weight::zero(),
	})
}

fn schedule_named(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Scheduler(pallet_scheduler::Call::schedule_named {
		id: [0; 32],
		when: 100,
		maybe_periodic: None,
		priority: 0,
		call: Box::new(call),
	})
}

fn schedule_after(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Scheduler(pallet_scheduler::Call::schedule_after {
		after: 100,
		maybe_periodic: None,
		priority: 0,
		call: Box::new(call),
	})
}

fn schedule_named_after(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Scheduler(pallet_scheduler::Call::schedule_named_after {
		id: [0; 32],
		after: 100,
		maybe_periodic: None,
		priority: 0,
		call: Box::new(call),
	})
}

fn if_else(main: RuntimeCall, fallback: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Utility(pallet_utility::Call::if_else {
		main: Box::new(main),
		fallback: Box::new(fallback),
	})
}

fn dispatch_as_account(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::Communities(pallet_communities::Call::dispatch_as_account { call: Box::new(call) })
}

fn event_horizon(call: RuntimeCall) -> RuntimeCall {
	RuntimeCall::BlackHole(pallet_black_hole::Call::dispatch_as_event_horizon { call: Box::new(call) })
}

fn xcm_execute() -> RuntimeCall {
	RuntimeCall::PolkadotXcm(pallet_xcm::Call::execute {
		message: Box::new(xcm::VersionedXcm::V5(xcm::v5::Xcm(vec![]))),
		max_weight: frame_support::weights::Weight::zero(),
	})
}

/// A credential of the device made from `SEED`, for `call`, as a wallet would sign it.
fn credential_for(call: &RuntimeCall) -> PassCredential {
	let pair = sr25519::Pair::from_seed(&SEED);
	let context: BlockNumber = System::block_number();
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

/// The device signs `call` through the real extension.
fn sign(device_id: pass::DeviceId, call: RuntimeCall) -> Result<(), TransactionValidityError> {
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

/// A registered device (`Admin`, as `register` makes it), and the account it belongs to.
fn device() -> (AccountId, pass::DeviceId) {
	System::set_block_number(1);
	assert_ok!(Balances::mint_into(&ALICE, UNITS));
	let account = pass::account(USER);
	let (attestation, device_id) = pass::attestation(&account, SEED);
	assert_ok!(Pass::register(RuntimeOrigin::signed(ALICE), USER, attestation));
	(account, device_id)
}

/// Restricts the device to exactly the calls `allowed` (the wrappers included).
fn allow_only(account: &AccountId, device_id: pass::DeviceId, allowed: &[RuntimeCall]) {
	let calls: BTreeSet<(u8, u8)> = allowed.iter().map(KreivoCallMatcher::call_indices).collect();
	DeviceFilters::<Runtime>::insert(
		account,
		device_id,
		DeviceFilter::Calls(BoundedBTreeSet::try_from(calls).expect("fits MaxFilteredCalls")),
	);
}

// A restricted device runs the calls its filter allows, so the setup is sound.
#[test]
fn a_device_runs_the_calls_its_filter_allows() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[batch(vec![]), remark()]);

		assert_ok!(sign(id, remark()));
		assert_ok!(sign(id, batch(vec![])));
		assert_eq!(sign(id, transfer_all()), rejected());
	})
}

// A batch of calls that the filter allows passes.
#[test]
fn a_batch_of_allowed_calls_passes() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[batch(vec![]), remark()]);

		assert_ok!(sign(id, batch(vec![remark(), remark(), remark()])));
	})
}

// The adversarial cases: the device is allowed the wrapper (and the harmless `remark`), and wraps a
// call that its filter denies.
#[test]
fn a_utility_batch_cannot_smuggle_a_denied_call() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[batch(vec![]), remark()]);

		assert_eq!(sign(id, batch(vec![remark(), transfer_all()])), rejected());
		for wrapper in [
			|calls| RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }),
			|calls| RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }),
		] {
			allow_only(&account, id, &[wrapper(vec![]), remark()]);
			assert_eq!(sign(id, wrapper(vec![transfer_all()])), rejected());
			assert_ok!(sign(id, wrapper(vec![remark()])));
		}
	})
}

#[test]
fn a_proxy_call_cannot_smuggle_a_denied_call() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[proxy(remark()), remark()]);

		assert_eq!(sign(id, proxy(transfer_all())), rejected());
		assert_ok!(sign(id, proxy(remark())));
	})
}

#[test]
fn a_multisig_call_cannot_smuggle_a_denied_call() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[as_multi_threshold_1(remark()), remark()]);

		assert_eq!(sign(id, as_multi_threshold_1(transfer_all())), rejected());
		assert_ok!(sign(id, as_multi_threshold_1(remark())));
	})
}

#[test]
fn a_dispatch_as_cannot_smuggle_a_denied_call() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[dispatch_as(remark()), remark()]);

		assert_eq!(sign(id, dispatch_as(transfer_all())), rejected());
		assert_ok!(sign(id, dispatch_as(remark())));
	})
}

#[test]
fn a_scheduled_call_cannot_smuggle_a_denied_call() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[schedule(remark()), remark()]);

		assert_eq!(sign(id, schedule(transfer_all())), rejected());
		assert_ok!(sign(id, schedule(remark())));
	})
}

// The rest of the wrappers, each one: the device is allowed the wrapper and `remark`, and cannot
// smuggle a denied call; with an allowed call it passes.
#[test]
fn the_other_wrappers_cannot_smuggle_a_denied_call() {
	let wrappers: [(&str, fn(RuntimeCall) -> RuntimeCall); 8] = [
		("as_derivative", as_derivative),
		("dispatch_as_fallible", dispatch_as_fallible),
		("with_weight", with_weight),
		("proxy_announced", proxy_announced),
		("as_multi", as_multi),
		("schedule_named", schedule_named),
		("schedule_after", schedule_after),
		("schedule_named_after", schedule_named_after),
	];
	for (name, wrap) in wrappers {
		TestExternalities::default().execute_with(|| {
			let (account, id) = device();
			allow_only(&account, id, &[wrap(remark()), remark()]);

			assert_eq!(
				sign(id, wrap(transfer_all())),
				rejected(),
				"{name} let a denied call through"
			);
			assert_ok!(sign(id, wrap(remark())));
		})
	}
}

// `Referenda::submit` is not a wrapper: its proposal runs later with the track's origin, after a
// vote. A restricted device allowed `Referenda` can submit, whatever the proposal.
#[test]
fn a_device_allowed_to_submit_a_referendum_can_submit() {
	use frame_support::traits::{schedule::DispatchTime, Bounded};

	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		let submit = |proposal: RuntimeCall| {
			RuntimeCall::CommunityReferenda(pallet_referenda::Call::submit {
				proposal_origin: Box::new(frame_system::RawOrigin::Root.into()),
				proposal: Bounded::Inline(BoundedVec::truncate_from(proposal.encode())),
				enactment_moment: DispatchTime::After(1),
			})
		};
		allow_only(&account, id, &[submit(remark())]);

		assert_ok!(sign(id, submit(remark())));
		assert_ok!(sign(id, submit(transfer_all())));
		// It still is restricted to the call it is allowed.
		assert_eq!(sign(id, transfer_all()), rejected());
	})
}

// `if_else` carries two calls: both are checked.
#[test]
fn an_if_else_checks_both_of_its_calls() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[if_else(remark(), remark()), remark()]);

		assert_eq!(sign(id, if_else(remark(), transfer_all())), rejected());
		assert_eq!(sign(id, if_else(transfer_all(), remark())), rejected());
		assert_ok!(sign(id, if_else(remark(), remark())));
	})
}

// The wrappers that are not in `Utility`: a community account dispatch and the black hole.
#[test]
fn the_community_and_black_hole_dispatches_cannot_smuggle_a_denied_call() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(
			&account,
			id,
			&[dispatch_as_account(remark()), event_horizon(remark()), remark()],
		);

		assert_eq!(sign(id, dispatch_as_account(transfer_all())), rejected());
		assert_eq!(sign(id, event_horizon(transfer_all())), rejected());
		assert_ok!(sign(id, dispatch_as_account(remark())));
		assert_ok!(sign(id, event_horizon(remark())));
	})
}

// Chains of wrappers are followed to the end.
#[test]
fn a_chain_of_wrappers_cannot_smuggle_a_denied_call() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(
			&account,
			id,
			&[
				batch(vec![]),
				proxy(remark()),
				dispatch_as(remark()),
				as_multi_threshold_1(remark()),
				schedule(remark()),
				remark(),
			],
		);

		// Within the limits (three wrappers, the leaf at depth 3): an allowed leaf passes, and a
		// denied one is rejected because it is denied.
		let short = |leaf: RuntimeCall| batch(vec![proxy(dispatch_as(leaf))]);
		assert_ok!(sign(id, short(remark())));
		assert_eq!(sign(id, short(transfer_all())), rejected());

		// Four wrappers (the leaf at depth 4, the limit): the same.
		let at_the_limit = |leaf: RuntimeCall| batch(vec![proxy(dispatch_as(as_multi_threshold_1(leaf)))]);
		assert_ok!(sign(id, at_the_limit(remark())));
		assert_eq!(sign(id, at_the_limit(transfer_all())), rejected());

		// Five wrappers, the leaf at depth 5: over `MaxNestedCallDepth` (4), even when every call of the
		// chain is allowed. This is the depth, not the filter.
		let too_deep = |leaf: RuntimeCall| batch(vec![proxy(dispatch_as(as_multi_threshold_1(schedule(leaf))))]);
		assert_eq!(sign(id, too_deep(remark())), rejected());
		assert_eq!(sign(id, too_deep(transfer_all())), rejected());
	})
}

// A transaction nests at most `MaxNestedCallDepth` levels (4), and carries at most `MaxNestedCalls`
// calls (64).
#[test]
fn nesting_over_the_depth_or_the_count_limits_is_rejected() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[batch(vec![]), remark()]);

		let nested = |wrappers: u32| (0..wrappers).fold(remark(), |inner, _| batch(vec![inner]));
		assert_ok!(sign(id, nested(4)));
		assert_eq!(sign(id, nested(5)), rejected());

		assert_ok!(sign(id, batch(vec![remark(); 64])));
		assert_eq!(sign(id, batch(vec![remark(); 65])), rejected());
	})
}

// A call that carries an XCM program cannot be inspected: it is denied, even if the filter allows it.
#[test]
fn an_opaque_call_is_rejected() {
	TestExternalities::default().execute_with(|| {
		let (account, id) = device();
		allow_only(&account, id, &[xcm_execute(), batch(vec![]), remark()]);

		assert_eq!(sign(id, xcm_execute()), rejected());
		assert_eq!(sign(id, batch(vec![xcm_execute()])), rejected());
	})
}

// An `Admin` device is not inspected.
#[test]
fn an_admin_device_runs_nested_calls() {
	TestExternalities::default().execute_with(|| {
		let (_, id) = device();

		assert_ok!(sign(id, batch(vec![proxy(dispatch_as(transfer_all()))])));
		assert_ok!(sign(id, xcm_execute()));
	})
}
