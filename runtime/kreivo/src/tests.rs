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

/// Fees as pallet-revive's `BlockRatioFee` computed them before pallet-revive was removed:
/// the local copy must charge exactly the same.
#[test]
fn weight_to_fee_is_unchanged_without_pallet_revive() {
	use frame_support::weights::{constants::ExtrinsicBaseWeight, Weight, WeightToFee as _};

	#[cfg(not(feature = "paseo"))]
	let fees = [0, 30_819_395, 1_175_666_627, 587_833_313, 58_783_331_357, 3_333_333];
	// Paseo prices in its own `CENTS`.
	#[cfg(feature = "paseo")]
	let fees = [0, 9_245_818, 352_699_988, 176_349_994, 17_634_999_425, 1_000_000];

	for (weight, fee) in [
		Weight::from_parts(0, 0),
		Weight::from_parts(1_000_000_000, 0),
		Weight::from_parts(0, 100_000),
		Weight::from_parts(250_000_000, 50_000),
		Weight::from_parts(1_000_000_000, 5_000_000),
		ExtrinsicBaseWeight::get(),
	]
	.into_iter()
	.zip(fees)
	{
		assert_eq!(crate::WeightToFee::weight_to_fee(&weight), fee, "{weight:?}");
	}
}

/// `RuntimeBlockLength` is built as `BlockLength::max_with_normal_ratio` built it, before that
/// was deprecated.
#[test]
fn block_length_is_unchanged() {
	use parachains_common::NORMAL_DISPATCH_RATIO;

	#[allow(deprecated)]
	let before = frame_system::limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	assert_eq!(
		crate::config::system::RuntimeBlockLength::get().encode(),
		before.encode()
	);
}

#[test]
fn view_functions_api_dispatches_to_the_pallets() {
	use frame_support::view_functions::{
		runtime_api::runtime_decl_for_runtime_view_function::RuntimeViewFunctionV1, ViewFunctionDispatchError,
		ViewFunctionId,
	};

	TestExternalities::default().execute_with(|| {
		let unknown = ViewFunctionId {
			prefix: [0; 16],
			suffix: [0; 16],
		};
		assert!(matches!(
			<Runtime as RuntimeViewFunctionV1<crate::Block>>::execute_view_function(unknown, vec![]),
			Err(ViewFunctionDispatchError::NotFound(_))
		));
	})
}

mod block_bundling {
	use super::*;
	use cumulus_primitives_core::{
		relay_chain::{ClaimQueueOffset, CoreSelector},
		BlockBundleInfo, CoreInfo,
	};
	use frame_support::{
		traits::Get,
		weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
	};
	use sp_runtime::Perbill;

	const MIB: u64 = 1024 * 1024;

	fn with_cores(cores: u16) {
		crate::System::deposit_log(
			CoreInfo {
				selector: CoreSelector(0),
				claim_queue_offset: ClaimQueueOffset(0),
				number_of_cores: cores.into(),
			}
			.to_digest_item(),
		);
	}

	pub(super) fn in_bundle(index: u8) {
		crate::System::deposit_log(BlockBundleInfo { index, is_last: false }.to_digest_item());
	}

	/// A block gets a share of a core: `TargetBlockRate` blocks per relay slot (3),
	/// over the cores Kreivo has.
	#[test]
	fn a_block_gets_its_share_of_the_cores() {
		type MaximumBlockWeight = crate::config::system::MaximumBlockWeight;
		let rate = <crate::config::system::TargetBlockRate as Get<u32>>::get() as u64;

		TestExternalities::default().execute_with(|| {
			// One core per block: each gets the full PoV, and the blocks of a relay slot share
			// the 6s a node has to import them (at most the core's 2s).
			with_cores(rate as u16);
			in_bundle(1);
			assert_eq!(
				MaximumBlockWeight::get(),
				Weight::from_parts(
					(6 * WEIGHT_REF_TIME_PER_SECOND / rate).min(2 * WEIGHT_REF_TIME_PER_SECOND),
					10 * MIB
				)
			);
		});

		TestExternalities::default().execute_with(|| {
			// All the blocks of a relay slot in one core share its 2s and 10 MiB.
			with_cores(1);
			in_bundle(1);
			assert_eq!(
				MaximumBlockWeight::get(),
				Weight::from_parts(2 * WEIGHT_REF_TIME_PER_SECOND / rate, 10 * MIB / rate)
			);
		});
	}

	/// The scheduler only runs in the first block of a core, where the whole core is
	/// available, so a scheduled call is never dropped for being too heavy for a share.
	#[test]
	fn the_scheduler_runs_in_the_first_block_of_a_core() {
		type MaximumSchedulerWeight = crate::config::utilities::MaximumSchedulerWeight;
		let full_core = Weight::from_parts(2 * WEIGHT_REF_TIME_PER_SECOND, 10 * MIB);

		TestExternalities::default().execute_with(|| {
			in_bundle(0);
			assert_eq!(MaximumSchedulerWeight::get(), Perbill::from_percent(80) * full_core);
		});
		TestExternalities::default().execute_with(|| {
			in_bundle(3);
			assert_eq!(MaximumSchedulerWeight::get(), Weight::zero());
		});
		TestExternalities::default().execute_with(|| {
			// No bundle info: the block is alone in its PoV.
			assert_eq!(MaximumSchedulerWeight::get(), Perbill::from_percent(80) * full_core);
		});
	}

	#[test]
	fn target_block_rate_is_three_blocks_per_relay_slot() {
		use cumulus_primitives_core::runtime_decl_for_target_block_rate::TargetBlockRateV1;
		assert_eq!(Runtime::target_block_rate(), 3);
	}
}

/// Every time-based pallet counts relay chain blocks: parachain blocks come at a rate that
/// depends on the cores Kreivo has, and on how many share one with block bundling.
#[test]
fn time_is_measured_in_relay_chain_blocks() {
	use crate::config::RelaychainData;
	use core::any::TypeId;
	// Kreivo's referenda, the communities' referenda, and the community memberships.
	type KreivoReferendaInstance = pallet_referenda::Instance1;
	type CommunityReferendaInstance = pallet_referenda::Instance2;
	type CommunityMembershipsInstance = pallet_nfts::Instance2;

	fn relay<P: 'static>() -> bool {
		TypeId::of::<P>() == TypeId::of::<RelaychainData>()
	}

	assert!(relay::<<Runtime as pallet_scheduler::Config>::BlockNumberProvider>());
	assert!(relay::<
		<Runtime as pallet_referenda::Config<KreivoReferendaInstance>>::BlockNumberProvider,
	>());
	assert!(relay::<
		<Runtime as pallet_referenda::Config<CommunityReferendaInstance>>::BlockNumberProvider,
	>());
	assert!(relay::<<Runtime as pallet_communities::Config>::BlockNumberProvider>());
	assert!(relay::<<Runtime as pallet_pass::Config>::BlockNumberProvider>());
	assert!(relay::<<Runtime as pallet_payments::Config>::BlockNumberProvider>());
	assert!(relay::<<Runtime as pallet_treasury::Config>::BlockNumberProvider>());
	assert!(relay::<<Runtime as pallet_vesting::Config>::BlockNumberProvider>());
	assert!(relay::<<Runtime as pallet_proxy::Config>::BlockNumberProvider>());
	assert!(relay::<<Runtime as pallet_multisig::Config>::BlockNumberProvider>());
	assert!(relay::<
		<Runtime as pallet_nfts::Config<CommunityMembershipsInstance>>::BlockNumberProvider,
	>());

	// A relay chain block every 6s.
	assert_eq!(runtime_constants::time::DAYS, 14_400);
}

/// What used to count parachain blocks now counts relay chain blocks, and what still counts
/// parachain blocks keeps doing so. Each test moves the two block numbers apart, so it can tell
/// which one a pallet follows.
mod relay_chain_time {
	use super::*;

	use crate::{
		config::RelaychainData, BlockNumber, KreivoReferenda, Pass, RuntimeCall, RuntimeEvent, Scheduler, System,
	};
	use frame_support::traits::{schedule::DispatchTime, Bounded, OnInitialize};
	use runtime_constants::time::{parachain, DAYS, MINUTES};
	use sp_runtime::traits::BlockNumberProvider;

	fn at(parachain_block: BlockNumber, relay_block: BlockNumber) {
		System::set_block_number(parachain_block);
		RelaychainData::set_block_number(relay_block);
	}

	fn run_scheduler() {
		Scheduler::on_initialize(System::block_number());
	}

	fn dispatched_at(when: BlockNumber) -> bool {
		System::events().iter().any(|record| {
			matches!(
				record.event,
				RuntimeEvent::Scheduler(pallet_scheduler::Event::Dispatched { task: (at, _), .. }) if at == when
			)
		})
	}

	fn schedule_at(when: BlockNumber) {
		let call: RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
		assert_ok!(Scheduler::schedule(
			RuntimeOrigin::root(),
			when,
			None,
			0,
			Box::new(call)
		));
	}

	#[test]
	fn scheduled_calls_run_at_their_relay_chain_block() {
		TestExternalities::default().execute_with(|| {
			at(1, 100);
			schedule_at(110);

			// The parachain is far past block 110, but the relay chain isn't.
			at(10_000, 109);
			run_scheduler();
			assert!(!dispatched_at(110));

			at(10_001, 110);
			run_scheduler();
			assert!(dispatched_at(110));
		})
	}

	/// With block bundling, several parachain blocks share a relay chain block and a core.
	/// A call that comes due in a later block of a core waits for the first block of the next
	/// core, and is not lost.
	#[test]
	fn calls_due_in_a_later_block_of_a_core_wait_for_the_next_core() {
		TestExternalities::default().execute_with(|| {
			at(1, 100);
			schedule_at(101);

			at(2, 101);
			super::block_bundling::in_bundle(1);
			run_scheduler();
			assert!(!dispatched_at(101));

			// The next block starts with a fresh digest.
			System::initialize(&3, &Default::default(), &Default::default());
			at(3, 101);
			super::block_bundling::in_bundle(0);
			run_scheduler();
			assert!(dispatched_at(101));
		})
	}

	#[test]
	fn referenda_time_out_in_relay_chain_blocks() {
		type ReferendumInfoFor = pallet_referenda::ReferendumInfoFor<Runtime, pallet_referenda::Instance1>;
		const ALICE: AccountId32 = AccountId32::new([1; 32]);

		TestExternalities::default().execute_with(|| {
			assert_ok!(Balances::mint_into(&ALICE, 10 * UNITS));
			at(1, 1_000);

			let proposal: RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
			assert_ok!(KreivoReferenda::submit(
				RuntimeOrigin::signed(ALICE),
				Box::new(frame_system::RawOrigin::Root.into()),
				Bounded::Inline(proposal.encode().try_into().expect("a remark is small; qed")),
				DispatchTime::After(1),
			));

			// Without a decision deposit, it times out 2 days after submission, in relay chain
			// blocks (6s each).
			let Some(pallet_referenda::ReferendumInfo::Ongoing(status)) = ReferendumInfoFor::get(0) else {
				panic!("the referendum is ongoing");
			};
			assert_eq!(status.submitted, 1_000);
			let timeout = 1_000 + 2 * DAYS;
			assert_eq!(status.alarm.map(|(when, _)| when), Some(timeout));

			at(1_000_000, timeout - 1);
			run_scheduler();
			assert!(matches!(
				ReferendumInfoFor::get(0),
				Some(pallet_referenda::ReferendumInfo::Ongoing(_))
			));

			at(1_000_001, timeout);
			run_scheduler();
			assert!(matches!(
				ReferendumInfoFor::get(0),
				Some(pallet_referenda::ReferendumInfo::TimedOut(..))
			));
		})
	}

	#[test]
	fn pass_sessions_last_relay_chain_blocks() {
		const ALICE: AccountId32 = AccountId32::new([1; 32]);
		let session = AccountId32::new([42; 32]);

		TestExternalities::default().execute_with(|| {
			assert_ok!(Balances::mint_into(&ALICE, UNITS));
			at(1, 1_000);

			let account = pass::account([1u8; 32]);
			let (device, _) = pass::attestation(&account, [10u8; 32]);
			assert_ok!(Pass::register(RuntimeOrigin::signed(ALICE), [1u8; 32], device));
			assert_ok!(Pass::add_session_key(
				RuntimeOrigin::signed(account),
				CommunityLookup::unlookup(session.clone()),
				Some(10 * MINUTES),
			));
			let active = || pallet_pass::SessionKeys::<Runtime>::contains_key(&session);

			at(1_000_000, 1_000 + 10 * MINUTES - 1);
			run_scheduler();
			assert!(active(), "ten minutes haven't passed on the relay chain");

			at(1_000_001, 1_000 + 10 * MINUTES + 1);
			run_scheduler();
			assert!(!active(), "the session ended after ten minutes");
		})
	}

	/// Device attestations sign a parachain block hash, so their lifetime is in parachain
	/// blocks.
	#[test]
	fn pass_challenges_expire_in_parachain_blocks() {
		const ALICE: AccountId32 = AccountId32::new([1; 32]);

		TestExternalities::default().execute_with(|| {
			assert_ok!(Balances::mint_into(&ALICE, UNITS));
			at(1_000, 5);
			let (device, _) = pass::attestation(&pass::account([1u8; 32]), [10u8; 32]);

			// Too many parachain blocks later: the challenge expired.
			at(1_000 + 30 * parachain::MINUTES + 1, 5);
			assert!(Pass::register(RuntimeOrigin::signed(ALICE), [1u8; 32], device.clone()).is_err());

			// The relay chain moving on doesn't expire it.
			at(1_000 + 10, 1_000_000);
			assert_ok!(Pass::register(RuntimeOrigin::signed(ALICE), [1u8; 32], device));
		})
	}

	#[test]
	fn memberships_expire_in_relay_chain_blocks() {
		use crate::config::currency::MembershipIsNotExpired;
		use runtime_constants::time::WEEKS;

		TestExternalities::default().execute_with(|| {
			if cfg!(feature = "runtime-benchmarks") {
				assert_ok!(Balances::mint_into(
					&TreasuryAccount::get(),
					EXISTENTIAL_DEPOSIT + 10 * CENTS
				));
			}
			at(1, 1);
			// Memberships 0..10, expiring at relay chain block `8 * WEEKS`.
			assert_ok!(CommunitiesManager::create_memberships(
				RuntimeOrigin::root(),
				10,
				0,
				CENTS,
				TankConfig::default(),
				Some(8 * WEEKS),
			));
			let not_expired = || MembershipIsNotExpired::get().select(0, 0);

			at(8 * WEEKS * 20, 8 * WEEKS);
			assert!(not_expired(), "the parachain block number doesn't matter");

			at(1, 8 * WEEKS + 1);
			assert!(!not_expired());
		})
	}

	#[test]
	fn collator_sessions_rotate_every_hour_of_parachain_blocks() {
		assert_eq!(crate::config::collator_support::Period::get(), parachain::HOURS);
		// 3 parachain blocks per 6s relay chain slot.
		assert_eq!(parachain::HOURS, 1_800);
	}
}

/// 0.17.0 moves the scheduler and referenda from the parachain clock to the relay chain clock,
/// on a live chain whose relay chain block number is millions of blocks behind its parachain
/// block number. Each test sets things up as 0.16 did (counting parachain blocks), switches
/// the clocks, migrates, and checks that everything happens as far from the switch as it would
/// have before.
mod scheduler_clock_switch {
	use super::*;

	use crate::{
		config::{utilities::MaxScheduledPerBlock, RelaychainData},
		migrations::{ClockSwitch, MigrationSummary, SchedulerClockSwitch, SchedulerToRelayChainClock},
		BlockNumber, CommunityReferenda, KreivoReferenda, OriginCaller, Pass, RuntimeCall, RuntimeEvent, Scheduler,
		System,
	};
	use frame_support::traits::{schedule::DispatchTime, Bounded, OnInitialize, OnRuntimeUpgrade};
	use pallet_referenda::{ReferendumInfo, ReferendumInfoFor, ReferendumStatusOf};
	use pallet_scheduler::{Agenda, IncompleteSince, Lookup, Retries, RetryConfig, Scheduled, ScheduledOf};
	use runtime_constants::time::DAYS;
	use sp_runtime::traits::BlockNumberProvider;

	/// Kreivo on Kusama, around the upgrade to 0.17.0.
	const PARA: BlockNumber = 39_000_000;
	const RELAY: BlockNumber = 35_400_000;

	const ALICE: AccountId32 = AccountId32::new([1; 32]);
	const BOB: AccountId32 = AccountId32::new([2; 32]);

	type KreivoReferendaInstance = pallet_referenda::Instance1;
	type CommunityReferendaInstance = pallet_referenda::Instance2;

	fn at(parachain_block: BlockNumber, relay_block: BlockNumber) {
		System::set_block_number(parachain_block);
		RelaychainData::set_block_number(relay_block);
	}

	/// On 0.16, the scheduler and referenda counted parachain blocks: have them see the
	/// parachain block number.
	fn on_the_parachain_clock(parachain_block: BlockNumber) {
		at(parachain_block, parachain_block);
	}

	/// What the upgrade does, with the `try-runtime` checks when they're built.
	fn upgrade() -> MigrationSummary {
		#[cfg(feature = "try-runtime")]
		let state = SchedulerToRelayChainClock::pre_upgrade().expect("pre-upgrade checks pass");
		let (_, summary) = SchedulerToRelayChainClock::migrate();
		#[cfg(feature = "try-runtime")]
		SchedulerToRelayChainClock::post_upgrade(state).expect("post-upgrade checks pass");
		summary.expect("the migration ran")
	}

	/// Runs the scheduler in a new block with `relay_block` as its relay parent, until it
	/// services every agenda up to it (it services ~50 per block).
	fn run_scheduler_at(relay_block: BlockNumber) {
		at(System::block_number() + 1, relay_block);
		for _ in 0..1_000 {
			if IncompleteSince::<Runtime>::get().is_some_and(|next| next > relay_block) {
				return;
			}
			Scheduler::on_initialize(System::block_number());
		}
		panic!("the scheduler didn't catch up");
	}

	fn remark(remark: &[u8]) -> RuntimeCall {
		frame_system::Call::remark {
			remark: remark.to_vec(),
		}
		.into()
	}

	fn inline(call: RuntimeCall) -> Bounded<RuntimeCall, sp_runtime::traits::BlakeTwo256> {
		Bounded::Inline(call.encode().try_into().expect("a remark is small; qed"))
	}

	fn root_task(name: Option<[u8; 32]>, call: RuntimeCall) -> ScheduledOf<Runtime> {
		Scheduled {
			maybe_id: name,
			priority: 0,
			call: inline(call),
			maybe_periodic: None,
			origin: frame_system::RawOrigin::Root.into(),
			_phantom: Default::default(),
		}
	}

	fn name(name: &str) -> [u8; 32] {
		sp_io::hashing::blake2_256(name.as_bytes())
	}

	fn schedule(when: BlockNumber, call: RuntimeCall) {
		assert_ok!(Scheduler::schedule(
			RuntimeOrigin::root(),
			when,
			None,
			0,
			Box::new(call)
		));
	}

	fn schedule_named(name: [u8; 32], when: BlockNumber) {
		assert_ok!(Scheduler::schedule_named(
			RuntimeOrigin::root(),
			name,
			when,
			None,
			0,
			Box::new(remark(&name))
		));
	}

	fn ongoing<I: 'static>(index: u32) -> ReferendumStatusOf<Runtime, I>
	where
		Runtime: pallet_referenda::Config<I>,
	{
		match ReferendumInfoFor::<Runtime, I>::get(index) {
			Some(ReferendumInfo::Ongoing(status)) => status,
			other => panic!("referendum {index} isn't ongoing: {other:?}"),
		}
	}

	fn timed_out<I: 'static>(index: u32) -> bool
	where
		Runtime: pallet_referenda::Config<I>,
	{
		matches!(
			ReferendumInfoFor::<Runtime, I>::get(index),
			Some(ReferendumInfo::TimedOut(..))
		)
	}

	fn dispatched_at(when: BlockNumber) -> usize {
		System::events()
			.iter()
			.filter(|record| {
				matches!(
					record.event,
					RuntimeEvent::Scheduler(pallet_scheduler::Event::Dispatched { task: (at, _), .. }) if at == when
				)
			})
			.count()
	}

	/// Community 1, with `ALICE` as its admin and `BOB` as a member.
	fn community_with_a_member() {
		if cfg!(feature = "runtime-benchmarks") {
			assert_ok!(Balances::mint_into(
				&TreasuryAccount::get(),
				EXISTENTIAL_DEPOSIT + 10 * CENTS
			));
		}
		assert_ok!(CommunitiesManager::create_memberships(
			RuntimeOrigin::root(),
			10,
			0,
			CENTS,
			TankConfig::default(),
			None,
		));
		assert_ok!(CommunitiesManager::register(
			RuntimeOrigin::root(),
			1,
			BoundedVec::try_from(b"First Community".to_vec()).expect("meets max length; qed"),
			CommunityLookup::unlookup(ALICE),
			None,
			None,
		));
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
	}

	/// The task `pallet_pass` schedules to end a session.
	fn session_removal(session: &AccountId) -> Option<(BlockNumber, u32)> {
		Lookup::<Runtime>::get(sp_io::hashing::blake2_256(&("remove_session_key", session).encode()))
	}

	#[test]
	fn everything_scheduled_happens_as_far_from_the_switch_as_before() {
		let session = AccountId32::new([42; 32]);
		let max_per_block = MaxScheduledPerBlock::get();

		TestExternalities::default().execute_with(|| {
			assert_ok!(Balances::mint_into(&ALICE, 10 * UNITS));

			// A community referendum, submitted 14 days (its undeciding timeout) minus 200
			// blocks before the switch.
			on_the_parachain_clock(PARA - 14 * DAYS + 200);
			community_with_a_member();
			assert_ok!(CommunityReferenda::submit(
				RuntimeOrigin::signed(BOB),
				Box::new(OriginCaller::from(pallet_communities::Origin::<Runtime>::new(1))),
				inline(remark(b"community")),
				DispatchTime::After(1),
			));
			assert_eq!(
				ongoing::<CommunityReferendaInstance>(0).alarm.map(|(when, _)| when),
				Some(PARA + 200)
			);

			// A Kreivo referendum, submitted 2 days (its undeciding timeout) minus 100 blocks
			// before the switch, to be enacted at a given block.
			on_the_parachain_clock(PARA - 2 * DAYS + 100);
			assert_ok!(KreivoReferenda::submit(
				RuntimeOrigin::signed(ALICE),
				Box::new(frame_system::RawOrigin::Root.into()),
				inline(remark(b"kreivo")),
				DispatchTime::At(PARA + 5_000),
			));
			assert_eq!(
				ongoing::<KreivoReferendaInstance>(0).alarm.map(|(when, _)| when),
				Some(PARA + 100)
			);

			// A pass session of 1800 blocks, opened right before the switch.
			on_the_parachain_clock(PARA);
			let account = pass::account([1u8; 32]);
			let (device, _) = pass::attestation(&account, [10u8; 32]);
			assert_ok!(Pass::register(RuntimeOrigin::signed(ALICE), [1u8; 32], device));
			assert_ok!(Pass::add_session_key(
				RuntimeOrigin::signed(account),
				CommunityLookup::unlookup(session.clone()),
				Some(1_800),
			));
			assert_eq!(session_removal(&session), Some((PARA + 1_801, 0)));

			// A full agenda right after the switch, one of its tasks with retries.
			for _ in 0..max_per_block {
				schedule(PARA + 1, remark(b"next"));
			}
			let retry = RetryConfig {
				total_retries: 3,
				remaining: 3,
				period: 10,
			};
			Retries::<Runtime>::insert((PARA + 1, 7), retry.clone());

			// The scheduler serviced every agenda up to the last block.
			IncompleteSince::<Runtime>::put(PARA + 1);
			// Agendas it will never go back to, as on Kusama: an empty one, and a named task
			// with its lookup and retries. And one where a moved agenda will land.
			let dead = name("the scheduler won't reach");
			Agenda::<Runtime>::insert(1_000, BoundedVec::new());
			Agenda::<Runtime>::insert(
				3_570_164,
				BoundedVec::truncate_from(vec![Some(root_task(Some(dead), remark(b"dead")))]),
			);
			Lookup::<Runtime>::insert(dead, (3_570_164, 0));
			Retries::<Runtime>::insert((3_570_164, 0), retry.clone());
			Agenda::<Runtime>::insert(
				RELAY + 1,
				BoundedVec::truncate_from(vec![Some(root_task(None, remark(b"dead")))]),
			);

			// The upgrade: now the scheduler and referenda see the relay chain block number.
			at(PARA, RELAY);
			assert_eq!(
				upgrade(),
				MigrationSummary {
					removed_agendas: 3,
					removed_tasks: 2,
					moved_tasks: max_per_block + 3,
					removed_lookups: 1,
					removed_retries: 1,
					remapped_referenda: 2,
					..Default::default()
				}
			);

			// The scheduler goes on from the next relay chain block, where the agenda that came
			// next on the parachain is now. What it would never reach is gone.
			assert_eq!(IncompleteSince::<Runtime>::get(), Some(RELAY + 1));
			assert_eq!(
				SchedulerClockSwitch::get(),
				Some(ClockSwitch {
					parachain: PARA,
					relay_chain: RELAY
				})
			);
			let next = Agenda::<Runtime>::get(RELAY + 1);
			assert_eq!(next.len() as u32, max_per_block);
			assert!(next.iter().flatten().all(|task| task.call == inline(remark(b"next"))));
			assert_eq!(Retries::<Runtime>::get((RELAY + 1, 7)), Some(retry));
			assert!(!Agenda::<Runtime>::contains_key(1_000));
			assert!(!Agenda::<Runtime>::contains_key(3_570_164));
			assert_eq!(Lookup::<Runtime>::get(dead), None);
			assert_eq!(Retries::<Runtime>::get((3_570_164, 0)), None);

			// The session ends 1800 relay chain blocks after the switch, not ~3.6M later.
			assert_eq!(session_removal(&session), Some((RELAY + 1_801, 0)));

			// Referenda keep the time that passed, and the time left.
			let kreivo = ongoing::<KreivoReferendaInstance>(0);
			assert_eq!(kreivo.submitted, RELAY - 2 * DAYS + 100);
			assert_eq!(kreivo.enactment, DispatchTime::At(RELAY + 5_000));
			assert_eq!(kreivo.alarm, Some((RELAY + 100, (RELAY + 100, 0))));
			let community = ongoing::<CommunityReferendaInstance>(0);
			assert_eq!(community.submitted, RELAY - 14 * DAYS + 200);
			assert_eq!(community.alarm, Some((RELAY + 200, (RELAY + 200, 0))));

			// And everything happens when it should.
			run_scheduler_at(RELAY + 1);
			assert_eq!(dispatched_at(RELAY + 1), max_per_block as usize);

			run_scheduler_at(RELAY + 99);
			assert!(!timed_out::<KreivoReferendaInstance>(0));
			run_scheduler_at(RELAY + 100);
			assert!(timed_out::<KreivoReferendaInstance>(0));

			run_scheduler_at(RELAY + 199);
			assert!(!timed_out::<CommunityReferendaInstance>(0));
			run_scheduler_at(RELAY + 200);
			assert!(timed_out::<CommunityReferendaInstance>(0));

			let active = || pallet_pass::SessionKeys::<Runtime>::contains_key(&session);
			run_scheduler_at(RELAY + 1_800);
			assert!(active());
			run_scheduler_at(RELAY + 1_801);
			assert!(!active(), "the session ended 1800 blocks after it was opened");
		})
	}

	/// If the scheduler fell behind, what it didn't get to runs first; what doesn't fit in an
	/// agenda goes to the next one.
	#[test]
	fn a_backlog_runs_first_and_full_agendas_spill_over() {
		let max_per_block = MaxScheduledPerBlock::get();

		TestExternalities::default().execute_with(|| {
			on_the_parachain_clock(PARA - 20);
			schedule_named(name("before the scheduler fell behind"), PARA - 10);
			schedule_named(name("the scheduler didn't reach"), PARA - 3);
			for _ in 0..max_per_block {
				schedule(PARA + 1, remark(b"next"));
			}
			schedule_named(name("two blocks after the switch"), PARA + 2);
			// It fell behind at `PARA - 5`.
			IncompleteSince::<Runtime>::put(PARA - 5);

			at(PARA, RELAY);
			let summary = upgrade();
			assert_eq!(summary.removed_agendas, 1);
			assert_eq!(summary.moved_tasks, max_per_block + 2);
			assert_eq!(summary.spilled_tasks, 1);

			assert_eq!(Lookup::<Runtime>::get(name("before the scheduler fell behind")), None);
			assert_eq!(
				Lookup::<Runtime>::get(name("the scheduler didn't reach")),
				Some((RELAY + 1, 0))
			);
			assert_eq!(Agenda::<Runtime>::get(RELAY + 1).len() as u32, max_per_block);
			// The last task of `PARA + 1`, then the one of `PARA + 2`.
			let after = Agenda::<Runtime>::get(RELAY + 2);
			assert_eq!(after.len(), 2);
			assert_eq!(after[0].as_ref().map(|task| &task.call), Some(&inline(remark(b"next"))));
			assert_eq!(
				Lookup::<Runtime>::get(name("two blocks after the switch")),
				Some((RELAY + 2, 1))
			);

			run_scheduler_at(RELAY + 2);
			assert_eq!(dispatched_at(RELAY + 1), max_per_block as usize);
			assert_eq!(dispatched_at(RELAY + 2), 2);
		})
	}

	/// A referendum whose alarm isn't a pending task would never be serviced: it gets a new
	/// alarm.
	#[test]
	fn a_referendum_without_its_alarm_gets_a_new_one() {
		TestExternalities::default().execute_with(|| {
			assert_ok!(Balances::mint_into(&ALICE, 10 * UNITS));
			on_the_parachain_clock(PARA - 2 * DAYS + 100);
			assert_ok!(KreivoReferenda::submit(
				RuntimeOrigin::signed(ALICE),
				Box::new(frame_system::RawOrigin::Root.into()),
				inline(remark(b"kreivo")),
				DispatchTime::After(1),
			));
			Agenda::<Runtime>::remove(PARA + 100);
			IncompleteSince::<Runtime>::put(PARA + 1);

			at(PARA, RELAY);
			assert_eq!(upgrade().rearmed_referenda, 1);
			assert_eq!(
				ongoing::<KreivoReferendaInstance>(0).alarm,
				Some((RELAY + 100, (RELAY + 100, 0)))
			);

			run_scheduler_at(RELAY + 100);
			assert!(timed_out::<KreivoReferendaInstance>(0));
		})
	}

	#[test]
	fn the_migration_runs_once() {
		TestExternalities::default().execute_with(|| {
			on_the_parachain_clock(PARA);
			schedule(PARA + 10, remark(b"later"));
			IncompleteSince::<Runtime>::put(PARA + 1);

			at(PARA, RELAY);
			upgrade();
			assert_eq!(Agenda::<Runtime>::get(RELAY + 10).len(), 1);

			// Even once the clocks moved on, it does nothing.
			at(PARA + 10, RELAY + 5);
			let root = sp_io::storage::root(sp_runtime::StateVersion::V1);
			#[cfg(feature = "try-runtime")]
			let state = SchedulerToRelayChainClock::pre_upgrade().expect("pre-upgrade checks pass");
			SchedulerToRelayChainClock::on_runtime_upgrade();
			#[cfg(feature = "try-runtime")]
			SchedulerToRelayChainClock::post_upgrade(state).expect("post-upgrade checks pass");
			assert_eq!(sp_io::storage::root(sp_runtime::StateVersion::V1), root);
			assert_eq!(SchedulerToRelayChainClock::migrate().1, None);
		})
	}
}
