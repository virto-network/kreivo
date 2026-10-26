//! The call matcher of `pallet_pass`: which calls a call carries, so that a device or session key
//! with a restricted filter cannot run a call that its filter denies by wrapping it in one that it
//! allows (`Utility.batch`, `Proxy.proxy`, ...).
//!
//! `pallet_pass` asks the matcher for the calls that a call dispatches itself, **one level deep**,
//! and walks the tree: the filter has to allow the wrapper and every call it carries, up to
//! `MaxNestedCallDepth` levels and `MaxNestedCalls` calls. [`NestedCalls::Opaque`] is denied.
//!
//! Be exhaustive: a wrapper that is not listed here is a wrapper that is not checked.
//! [`CLASSIFICATION`] lists every call of the runtime that carries a call (and says why it is, or is
//! not, a wrapper); a test reads the metadata and fails when a pallet or a call is missing from it.
//!
//! Deliberately **not** wrappers, besides the `NotAWrapper` entries of the table:
//!
//! - `Multisig::approve_as_multi` carries only the hash of a call. The `as_multi` that finally
//!   dispatches carries the call, and is checked. Policy caveat (accepted): a device that is
//!   allowed `approve_as_multi` can add its account's approval to a hash that the filter never sees.
//! - Contracts: `call_runtime` dispatches with the contract's own origin, and the runtime's
//!   `CallFilter` for contracts allows only `Balances::transfer_allow_death` and `Assets`
//!   (`config/contracts/mod.rs`), so no call of the signer is wrapped.

use crate::RuntimeCall;
use pallet_pass::{CallMatcher, NestedCalls, ScaleCallMatcher};

/// What [`KreivoCallMatcher`] does with a call that carries another call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classification {
	/// It dispatches the calls it carries: they are checked against the filter.
	Wrapper,
	/// It carries something that cannot be inspected: it is denied.
	Opaque,
	/// It carries a call, or something like one, that it does not dispatch with the signer's
	/// authority (or at all): nothing to check. Every one has its reason.
	None,
}

use Classification::{None as NotAWrapper, Opaque, Wrapper};

/// Every call of the runtime with an argument that carries a `RuntimeCall` (or an encoded one), and
/// what the matcher does with it. `tests::call_matcher_completeness` reads the metadata of the
/// runtime and fails when this table and the metadata differ: **a new pallet, or a new call that
/// carries a call, has to be classified here** (and, if it is a wrapper, listed in `nested_calls`).
pub const CLASSIFICATION: &[(&str, &str, Classification, &str)] = &[
	("Utility", "batch", Wrapper, "dispatches each call"),
	("Utility", "batch_all", Wrapper, "dispatches each call"),
	("Utility", "force_batch", Wrapper, "dispatches each call"),
	("Utility", "as_derivative", Wrapper, "dispatches the call as a derived account"),
	("Utility", "dispatch_as", Wrapper, "dispatches the call with another origin"),
	("Utility", "dispatch_as_fallible", Wrapper, "dispatches the call with another origin"),
	("Utility", "with_weight", Wrapper, "dispatches the call"),
	("Utility", "if_else", Wrapper, "dispatches `main`, and `fallback` if it fails: both are checked"),
	("Proxy", "proxy", Wrapper, "dispatches the call as the real account"),
	("Proxy", "proxy_announced", Wrapper, "dispatches the call as the real account"),
	("Multisig", "as_multi_threshold_1", Wrapper, "dispatches the call"),
	(
		"Multisig",
		"as_multi",
		Wrapper,
		"dispatches the call once the threshold is met; this is where the call is checked",
	),
	("Scheduler", "schedule", Wrapper, "schedules the call (inline, not a hash)"),
	("Scheduler", "schedule_named", Wrapper, "schedules the call (inline, not a hash)"),
	("Scheduler", "schedule_after", Wrapper, "schedules the call (inline, not a hash)"),
	("Scheduler", "schedule_named_after", Wrapper, "schedules the call (inline, not a hash)"),
	("Communities", "dispatch_as_account", Wrapper, "dispatches the call as the community's account"),
	("BlackHole", "dispatch_as_event_horizon", Wrapper, "dispatches the call as the event horizon"),
	(
		"PolkadotXcm",
		"execute",
		Opaque,
		"runs an XCM program locally, which can `Transact` any call with the signer's origin, inside an 		 encoded message: it cannot be inspected",
	),
	(
		"KreivoReferenda",
		"submit",
		NotAWrapper,
		"the proposal runs later, with the track's origin, and only if the vote approves it: never with 		 the signer's origin, so none of the device's authority is smuggled",
	),
	(
		"CommunityReferenda",
		"submit",
		NotAWrapper,
		"as `KreivoReferenda::submit`: the proposal runs with the track's origin after the vote",
	),
	(
		"PolkadotXcm",
		"send",
		NotAWrapper,
		"sends an XCM program to another chain: it does not run a call here",
	),
	(
		"PolkadotXcm",
		"transfer_assets_using_type_and_then",
		NotAWrapper,
		"the XCM it carries (`custom_xcm_on_dest`) runs on the destination chain; the assets it moves are 		 the spend the filter allows",
	),
];

/// A sample call of each wrapper and opaque call of [`CLASSIFICATION`], for the tests that check
/// that the table and [`KreivoCallMatcher::nested_calls`] agree.
#[cfg(test)]
pub fn samples() -> alloc::collections::BTreeMap<(&'static str, &'static str), RuntimeCall> {
	use alloc::{boxed::Box, collections::BTreeMap, vec};
	use frame_support::weights::Weight;
	use sp_runtime::{traits::StaticLookup, AccountId32};

	let inner = || RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
	let account = || AccountId32::new([1; 32]);
	let lookup = || crate::config::system::CommunityLookup::unlookup(account());
	let origin = || -> Box<crate::OriginCaller> { Box::new(frame_system::RawOrigin::Root.into()) };
	let scheduled = |f: fn(Box<RuntimeCall>) -> pallet_scheduler::Call<crate::Runtime>| {
		RuntimeCall::Scheduler(f(Box::new(inner())))
	};

	BTreeMap::from([
		(
			("Utility", "batch"),
			RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![inner()] }),
		),
		(
			("Utility", "batch_all"),
			RuntimeCall::Utility(pallet_utility::Call::batch_all { calls: vec![inner()] }),
		),
		(
			("Utility", "force_batch"),
			RuntimeCall::Utility(pallet_utility::Call::force_batch { calls: vec![inner()] }),
		),
		(
			("Utility", "as_derivative"),
			RuntimeCall::Utility(pallet_utility::Call::as_derivative {
				index: 0,
				call: Box::new(inner()),
			}),
		),
		(
			("Utility", "dispatch_as"),
			RuntimeCall::Utility(pallet_utility::Call::dispatch_as {
				as_origin: origin(),
				call: Box::new(inner()),
			}),
		),
		(
			("Utility", "dispatch_as_fallible"),
			RuntimeCall::Utility(pallet_utility::Call::dispatch_as_fallible {
				as_origin: origin(),
				call: Box::new(inner()),
			}),
		),
		(
			("Utility", "with_weight"),
			RuntimeCall::Utility(pallet_utility::Call::with_weight {
				call: Box::new(inner()),
				weight: Weight::zero(),
			}),
		),
		(
			("Utility", "if_else"),
			RuntimeCall::Utility(pallet_utility::Call::if_else {
				main: Box::new(inner()),
				fallback: Box::new(inner()),
			}),
		),
		(
			("Proxy", "proxy"),
			RuntimeCall::Proxy(pallet_proxy::Call::proxy {
				real: lookup(),
				force_proxy_type: None,
				call: Box::new(inner()),
			}),
		),
		(
			("Proxy", "proxy_announced"),
			RuntimeCall::Proxy(pallet_proxy::Call::proxy_announced {
				delegate: lookup(),
				real: lookup(),
				force_proxy_type: None,
				call: Box::new(inner()),
			}),
		),
		(
			("Multisig", "as_multi_threshold_1"),
			RuntimeCall::Multisig(pallet_multisig::Call::as_multi_threshold_1 {
				other_signatories: vec![account()],
				call: Box::new(inner()),
			}),
		),
		(
			("Multisig", "as_multi"),
			RuntimeCall::Multisig(pallet_multisig::Call::as_multi {
				threshold: 2,
				other_signatories: vec![account()],
				maybe_timepoint: None,
				call: Box::new(inner()),
				max_weight: Weight::zero(),
			}),
		),
		(
			("Scheduler", "schedule"),
			scheduled(|call| pallet_scheduler::Call::schedule {
				when: 1,
				maybe_periodic: None,
				priority: 0,
				call,
			}),
		),
		(
			("Scheduler", "schedule_named"),
			scheduled(|call| pallet_scheduler::Call::schedule_named {
				id: [0; 32],
				when: 1,
				maybe_periodic: None,
				priority: 0,
				call,
			}),
		),
		(
			("Scheduler", "schedule_after"),
			scheduled(|call| pallet_scheduler::Call::schedule_after {
				after: 1,
				maybe_periodic: None,
				priority: 0,
				call,
			}),
		),
		(
			("Scheduler", "schedule_named_after"),
			scheduled(|call| pallet_scheduler::Call::schedule_named_after {
				id: [0; 32],
				after: 1,
				maybe_periodic: None,
				priority: 0,
				call,
			}),
		),
		(
			("Communities", "dispatch_as_account"),
			RuntimeCall::Communities(pallet_communities::Call::dispatch_as_account {
				call: Box::new(inner()),
			}),
		),
		(
			("BlackHole", "dispatch_as_event_horizon"),
			RuntimeCall::BlackHole(pallet_black_hole::Call::dispatch_as_event_horizon {
				call: Box::new(inner()),
			}),
		),
		(
			("PolkadotXcm", "execute"),
			RuntimeCall::PolkadotXcm(pallet_xcm::Call::execute {
				message: Box::new(xcm::VersionedXcm::V5(xcm::v5::Xcm(vec![]))),
				max_weight: Weight::zero(),
			}),
		),
	])
}

pub struct KreivoCallMatcher;

impl CallMatcher<RuntimeCall> for KreivoCallMatcher {
	fn call_indices(call: &RuntimeCall) -> (u8, u8) {
		ScaleCallMatcher::call_indices(call)
	}

	fn nested_calls(call: &RuntimeCall) -> NestedCalls<'_, RuntimeCall> {
		match call {
			RuntimeCall::Utility(
				pallet_utility::Call::batch { calls }
				| pallet_utility::Call::batch_all { calls }
				| pallet_utility::Call::force_batch { calls },
			) => NestedCalls::Many(calls),
			RuntimeCall::Utility(
				pallet_utility::Call::as_derivative { call, .. }
				| pallet_utility::Call::dispatch_as { call, .. }
				| pallet_utility::Call::dispatch_as_fallible { call, .. }
				| pallet_utility::Call::with_weight { call, .. },
			) => NestedCalls::One(call),
			// Two separate calls: one of them runs, depending on the result of the first.
			RuntimeCall::Utility(pallet_utility::Call::if_else { main, fallback }) => {
				NestedCalls::from_calls([&**main, &**fallback].into_iter())
			}
			RuntimeCall::Proxy(
				pallet_proxy::Call::proxy { call, .. } | pallet_proxy::Call::proxy_announced { call, .. },
			) => NestedCalls::One(call),
			RuntimeCall::Multisig(
				pallet_multisig::Call::as_multi_threshold_1 { call, .. } | pallet_multisig::Call::as_multi { call, .. },
			) => NestedCalls::One(call),
			// The scheduler extrinsics carry the call itself (not a hash): a `Box<RuntimeCall>`.
			RuntimeCall::Scheduler(
				pallet_scheduler::Call::schedule { call, .. }
				| pallet_scheduler::Call::schedule_named { call, .. }
				| pallet_scheduler::Call::schedule_after { call, .. }
				| pallet_scheduler::Call::schedule_named_after { call, .. },
			) => NestedCalls::One(call),
			RuntimeCall::Communities(pallet_communities::Call::dispatch_as_account { call }) => NestedCalls::One(call),
			RuntimeCall::BlackHole(pallet_black_hole::Call::dispatch_as_event_horizon { call }) => {
				NestedCalls::One(call)
			}
			// Runs an XCM program locally, which can `Transact` any call with the signer's origin: the
			// calls are inside the encoded message, and cannot be inspected here.
			RuntimeCall::PolkadotXcm(pallet_xcm::Call::execute { .. }) => NestedCalls::Opaque,
			// `Referenda::submit` and the XCM `send` and transfers are `NotAWrapper` (see the table):
			// the proposal runs later with the track's origin, only if the vote approves it, and the XCM
			// goes to other chains.
			_ => NestedCalls::None,
		}
	}
}
