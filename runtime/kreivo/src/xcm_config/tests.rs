//! Acceptance tests for the message-backed bridging groundwork (specs/xcm-config/PLAN.md).

use super::*;
use crate::{ParachainInfo, PolkadotXcm, Runtime, RuntimeOrigin};

use frame_support::{
	assert_noop,
	traits::{EnsureOrigin, ProcessMessageError},
};
use parachains_common::AccountId;
use parity_scale_codec::Encode;
use sp_io::TestExternalities;
use xcm::{VersionedLocation, VersionedXcm};
use xcm_executor::traits::{ConvertLocation, ConvertOrigin, Properties, QueryHandler, ShouldExecute, WeightBounds};

const ESCROW_ID: [u8; 32] = [0xE5; 32];
const ALICE: [u8; 32] = [1; 32];

/// The escrow contract on Polkadot Asset Hub, as seen from Kreivo.
fn escrow() -> Location {
	Location::new(
		2,
		[
			GlobalConsensus(Polkadot),
			Parachain(ASSET_HUB_ID),
			AccountId32 {
				network: None,
				id: ESCROW_ID,
			},
		],
	)
}

fn sibling(id: u32) -> Location {
	Location::new(1, [Parachain(id)])
}

fn ksm(amount: u128) -> Asset {
	(Location::parent(), amount).into()
}

/// Runs `message` from `origin`, crediting its full weight up front, so the barrier's
/// `TakeWeightCredit` lets it through. Only the executor-level behaviour (aliasing,
/// expectations) is exercised.
fn execute_credited(origin: Location, message: Xcm<RuntimeCall>) -> Outcome {
	let mut message = message;
	let weight = <XcmConfig as xcm_executor::Config>::Weigher::weight(&mut message, Weight::MAX)
		.expect("the message only uses instructions we have weights for; qed");
	let mut hash = message.using_encoded(sp_io::hashing::blake2_256);
	XcmExecutor::<XcmConfig>::prepare_and_execute(origin, message, &mut hash, weight, weight)
}

fn barrier(origin: &Location, mut message: Xcm<RuntimeCall>) -> Result<(), ProcessMessageError> {
	Barrier::should_execute(
		origin,
		message.inner_mut(),
		Weight::from_parts(10_000_000_000, 1_000_000),
		&mut Properties {
			weight_credit: Weight::zero(),
			message_id: None,
		},
	)
}

// 1. Universal location

#[test]
fn universal_location_is_relay_then_para() {
	TestExternalities::default().execute_with(|| {
		let para: u32 = ParachainInfo::parachain_id().into();
		#[cfg(not(feature = "paseo"))]
		let relay = Kusama;
		// Paseo Asset Hub also identifies the Paseo relay as `Polkadot`.
		#[cfg(feature = "paseo")]
		let relay = Polkadot;
		assert_eq!(
			UniversalLocation::get(),
			InteriorLocation::from([GlobalConsensus(relay), Parachain(para)])
		);
	})
}

// 2. Barrier: version negotiation and query responses

#[test]
fn barrier_admits_version_subscriptions_from_siblings_and_relay() {
	TestExternalities::default().execute_with(|| {
		for origin in [sibling(ASSET_HUB_ID), Location::parent()] {
			assert_eq!(
				barrier(
					&origin,
					Xcm(vec![SubscribeVersion {
						query_id: 0,
						max_response_weight: Weight::zero(),
					}])
				),
				Ok(())
			);
		}
		// ...but not from arbitrary locations.
		assert!(barrier(
			&escrow(),
			Xcm(vec![SubscribeVersion {
				query_id: 0,
				max_response_weight: Weight::zero(),
			}])
		)
		.is_err());
	})
}

#[test]
fn barrier_admits_only_expected_query_responses() {
	TestExternalities::default().execute_with(|| {
		let responder = sibling(ASSET_HUB_ID);
		let query_id = <PolkadotXcm as QueryHandler>::new_query(responder.clone(), 100u32.into(), Here);
		let response = |query_id| {
			Xcm(vec![QueryResponse {
				query_id,
				response: Response::Null,
				max_weight: Weight::zero(),
				querier: Some(Here.into()),
			}])
		};

		assert_eq!(barrier(&responder, response(query_id)), Ok(()));
		// Unsolicited: unknown query id, or a response from someone we didn't ask.
		assert!(barrier(&responder, response(query_id + 1)).is_err());
		assert!(barrier(&sibling(2000), response(query_id)).is_err());
	})
}

// 3. Aliasing preserved origins

#[test]
fn asset_hub_can_alias_a_preserved_polkadot_origin() {
	TestExternalities::default().execute_with(|| {
		let outcome = execute_credited(
			sibling(ASSET_HUB_ID),
			Xcm(vec![AliasOrigin(escrow()), ExpectOrigin(Some(escrow()))]),
		);
		assert!(matches!(outcome, Outcome::Complete { .. }), "{outcome:?}");
	})
}

#[test]
fn a_transact_under_the_aliased_origin_dispatches_as_that_xcm_origin() {
	// C4: bridge messages use `OriginKind::Xcm`, which `XcmPassthrough` turns into
	// `pallet_xcm::Origin::Xcm(<location>)`, so the bridge pallet can gate on `EnsureXcm`.
	let origin =
		<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(escrow(), OriginKind::Xcm)
			.expect("XcmPassthrough converts any location; qed");
	assert_eq!(
		pallet_xcm::EnsureXcm::<Everything>::try_origin(origin).ok(),
		Some(escrow())
	);
}

#[test]
fn other_siblings_cannot_alias_a_polkadot_origin() {
	TestExternalities::default().execute_with(|| {
		let outcome = execute_credited(
			sibling(2000),
			Xcm(vec![AliasOrigin(escrow()), ExpectOrigin(Some(escrow()))]),
		);
		assert!(
			matches!(
				outcome,
				Outcome::Incomplete {
					error: InstructionError {
						error: XcmError::NoPermission,
						..
					},
					..
				}
			),
			"{outcome:?}"
		);
	})
}

#[test]
fn asset_hub_cannot_alias_other_kusama_locations() {
	TestExternalities::default().execute_with(|| {
		for target in [
			// A relay-chain account.
			Location::new(
				1,
				[AccountId32 {
					network: None,
					id: ALICE,
				}],
			),
			// Another parachain.
			sibling(2000),
		] {
			let outcome = execute_credited(sibling(ASSET_HUB_ID), Xcm(vec![AliasOrigin(target.clone())]));
			assert!(
				matches!(
					outcome,
					Outcome::Incomplete {
						error: InstructionError {
							error: XcmError::NoPermission,
							..
						},
						..
					}
				),
				"{target:?}: {outcome:?}"
			);
		}
	})
}

// 4. Location to account

#[test]
fn asset_hub_accounts_map_to_the_same_account() {
	TestExternalities::default().execute_with(|| {
		let on_asset_hub = Location::new(
			1,
			[
				Parachain(ASSET_HUB_ID),
				AccountId32 {
					network: None,
					id: ALICE,
				},
			],
		);
		assert_eq!(
			LocationToAccountId::convert_location(&on_asset_hub),
			Some(AccountId::new(ALICE))
		);

		// The malformed `parents: 2` form (no `GlobalConsensus`) no longer converts.
		let malformed = Location::new(
			2,
			[
				Parachain(ASSET_HUB_ID),
				AccountId32 {
					network: None,
					id: ALICE,
				},
			],
		);
		assert_eq!(LocationToAccountId::convert_location(&malformed), None);
	})
}

// On `paseo`, our own consensus is `Polkadot`, so the escrow location is not remote and has no
// bridged account; there is no Polkadot <> Kusama bridge on the testnet.
#[cfg(not(feature = "paseo"))]
#[test]
fn bridged_polkadot_origins_get_a_deterministic_account() {
	TestExternalities::default().execute_with(|| {
		let account = LocationToAccountId::convert_location(&escrow()).expect("bridged origins convert");
		assert_eq!(LocationToAccountId::convert_location(&escrow()), Some(account.clone()));
		// Derived, not the raw key: the escrow does not control the same-key local account.
		assert_ne!(account, AccountId::new(ESCROW_ID));
	})
}

// 5. Reserve-transfer guard for message-backed assets

#[test]
fn bridge_backed_assets_cannot_be_reserve_transferred() {
	let dest = sibling(ASSET_HUB_ID);
	for bridge_backed in BridgeBackedAssets::get() {
		assert!(!NotBridgeBacked::contains(&(
			dest.clone(),
			vec![(bridge_backed.clone(), 1u128).into()]
		)));
		// Also when mixed with other assets.
		assert!(!NotBridgeBacked::contains(&(
			dest.clone(),
			vec![ksm(1), (bridge_backed, 1u128).into()]
		)));
	}
	assert!(NotBridgeBacked::contains(&(dest.clone(), vec![ksm(1)])));
	assert!(NotBridgeBacked::contains(&(dest, vec![(Dot::get(), 1u128).into()])));

	TestExternalities::default().execute_with(|| {
		let usdt = BridgeBackedAssets::get()[1].clone();
		assert_noop!(
			PolkadotXcm::transfer_assets(
				RuntimeOrigin::signed(AccountId::new(ALICE)),
				Box::new(VersionedLocation::from(sibling(ASSET_HUB_ID))),
				Box::new(VersionedLocation::from(Location::new(
					0,
					[AccountId32 {
						network: None,
						id: ALICE
					}]
				))),
				Box::new(vec![(usdt, 1_000u128).into()].into()),
				0,
				WeightLimit::Unlimited,
			),
			pallet_xcm::Error::<Runtime>::Filtered
		);
	})
}

// 6. Governance can send XCM

#[test]
fn root_can_send_xcm_as_here_but_plain_accounts_cannot() {
	TestExternalities::default().execute_with(|| {
		type SendOrigin = <Runtime as pallet_xcm::Config>::SendXcmOrigin;
		assert_eq!(
			SendOrigin::try_origin(RuntimeOrigin::root()).ok(),
			Some(Location::here())
		);
		assert!(SendOrigin::try_origin(RuntimeOrigin::signed(AccountId::new(ALICE))).is_err());
	})
}

// 7. Runtime APIs

#[test]
fn xcm_payment_api_accepts_ksm_and_dot() {
	use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV2;

	TestExternalities::default().execute_with(|| {
		let accepted = Runtime::query_acceptable_payment_assets(XCM_VERSION).expect("supported version");
		assert!(accepted.contains(&xcm::VersionedAssetId::from(AssetId(Ksm::get()))));
		assert!(accepted.contains(&xcm::VersionedAssetId::from(AssetId(Dot::get()))));

		let weight = Runtime::query_xcm_weight(VersionedXcm::from(Xcm::<()>(vec![ClearOrigin]))).expect("weighable");
		let fee = Runtime::query_weight_to_asset_fee(weight, AssetId(Ksm::get()).into()).expect("KSM pays fees");
		assert!(fee > 0);
	})
}

// On `paseo`, our own consensus is `Polkadot`, so the escrow location is not remote and has no
// bridged account; there is no Polkadot <> Kusama bridge on the testnet.
#[cfg(not(feature = "paseo"))]
#[test]
fn dry_run_of_a_bridged_message_from_asset_hub_completes() {
	use crate::{Balances, UNITS};
	use xcm_runtime_apis::dry_run::runtime_decl_for_dry_run_api::DryRunApiV2;

	TestExternalities::default().execute_with(|| {
		// What Kusama Asset Hub forwards after `InitiateTransfer { preserve_origin: true }`.
		let message = Xcm::<RuntimeCall>(vec![
			ReserveAssetDeposited(ksm(UNITS).into()),
			PayFees { asset: ksm(UNITS) },
			AliasOrigin(escrow()),
			ExpectOrigin(Some(escrow())),
			RefundSurplus,
			DepositAsset {
				assets: Wild(AllCounted(1)),
				beneficiary: escrow(),
			},
		]);

		let effects = Runtime::dry_run_xcm(
			VersionedLocation::from(sibling(ASSET_HUB_ID)),
			VersionedXcm::from(message),
		)
		.expect("dry run executes");
		assert!(
			matches!(effects.execution_result, Outcome::Complete { .. }),
			"{:?}",
			effects.execution_result
		);

		// The change lands in the escrow's derived account.
		let escrow_account = LocationToAccountId::convert_location(&escrow()).expect("converts");
		assert!(Balances::free_balance(escrow_account) > 0);
	})
}

#[test]
fn location_to_account_api_uses_the_runtime_converter() {
	use xcm_runtime_apis::conversions::runtime_decl_for_location_to_account_api::LocationToAccountApiV1;

	TestExternalities::default().execute_with(|| {
		assert_eq!(
			Runtime::convert_location(VersionedLocation::from(escrow())).ok(),
			LocationToAccountId::convert_location(&escrow())
		);
	})
}
