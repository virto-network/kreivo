//! `pallet_migrations` as Kreivo wires it (`DEC-30`, `F-12`).
//!
//! The stepped migration these tests run is a no-op that costs most of `MaxServiceWeight` per
//! step (`migrations::test_migration`): it is only registered in test builds, and the benchmark
//! build swaps it for the pallet's mocked migrations, so the tests that run it are compiled out
//! under `runtime-benchmarks`.

use super::*;

use crate::{
	config::system::{
		MigrationsInherentReserve, MigrationsMaxServiceWeight, MigrationsStatusHandler, ReleaseSteppedMigrations,
	},
	weights::BlockExecutionWeight,
	MultiBlockMigrations,
};
use frame_support::{
	traits::{Get, Hooks},
	weights::Weight,
};

// NFR-9, DEC-30: the deployment document's values (§1).
#[test]
fn the_service_weight_and_the_limits_are_the_deployment_values() {
	// Derived, about 50 % of the block's share (`m3-max-service-weight`).
	assert_eq!(
		MigrationsMaxServiceWeight::get(),
		Weight::from_parts(332_414_064_333, 1_743_482)
	);
	assert_eq!(
		<<Runtime as pallet_migrations::Config>::CursorMaxLen as Get<u32>>::get(),
		65_536
	);
	assert_eq!(
		<<Runtime as pallet_migrations::Config>::IdentifierMaxLen as Get<u32>>::get(),
		256
	);
}

// DEC-30, RISK-14, M-2: in the third-of-a-core mode, a step plus the message queue's share, the base
// block and the inherent reserve fit in the block, in both components.
#[test]
fn a_step_leaves_room_for_the_message_queue_the_base_block_and_the_inherents() {
	TestExternalities::default().execute_with(|| {
		let max_block = <Runtime as frame_system::Config>::BlockWeights::get().max_block;
		// The mode this test pins: a third of a core.
		assert_eq!(max_block, Weight::from_parts(666_666_666_666, 3_495_253));

		let message_queue = <<Runtime as pallet_message_queue::Config>::ServiceWeight as Get<Option<Weight>>>::get()
			.expect("the message queue has a service weight");
		let step = MigrationsMaxServiceWeight::get();
		let sum = step
			.saturating_add(message_queue)
			.saturating_add(BlockExecutionWeight::get())
			.saturating_add(MigrationsInherentReserve::get());
		assert!(sum.ref_time() <= max_block.ref_time());
		assert!(sum.proof_size() <= max_block.proof_size());

		<MultiBlockMigrations as Hooks<_>>::integrity_test();
	});
}

// DEC-35, INV-21: while the status handler is `()`, no stepped migration may be registered.
// `F-08` binds the handler, and must land before `F-10` registers the memberships migration.
#[test]
fn no_stepped_migration_is_registered_while_the_status_handler_does_nothing() {
	use core::any::TypeId;
	use frame_support::migrations::SteppedMigrations;

	if TypeId::of::<MigrationsStatusHandler>() == TypeId::of::<()>() {
		assert_eq!(<ReleaseSteppedMigrations as SteppedMigrations>::len(), 0);
	}
}

#[cfg(not(feature = "runtime-benchmarks"))]
mod stepped {
	use super::*;

	use crate::{migrations::test_migration::*, RuntimeEvent, System};
	use frame_support::{migrations::MultiStepMigrator, traits::OnRuntimeUpgrade};
	use pallet_migrations::{Cursor, Event as MigrationsEvent, MbmIsOngoing, MigrationCursor};

	/// A block's worth of migration: the executive steps the migrator once after the inherents.
	fn next_block() -> Weight {
		System::set_block_number(System::block_number() + 1);
		<Runtime as frame_system::Config>::MultiBlockMigrator::step()
	}

	fn the_upgrade_block() {
		System::set_block_number(1);
		<MultiBlockMigrations as OnRuntimeUpgrade>::on_runtime_upgrade();
	}

	// NFR-9, DEC-30: a stepped migration takes one step a block, each under `MaxServiceWeight`, and
	// the block builder takes only inherents (the executive's `OnlyInherents` mode, which is
	// `ongoing()`) until it is done; user transactions wait in the pool and go in after.
	#[test]
	fn a_stepped_migration_spans_several_blocks_and_then_lets_transactions_in() {
		TestExternalities::default().execute_with(|| {
			NoopSteps::set(&4);
			the_upgrade_block();

			assert!(<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
			assert_eq!(MultiBlockMigrations::ongoing_status(), MbmIsOngoing::Yes);

			for _ in 0..3 {
				let used = next_block();
				assert!(used.all_gte(NoopStepped::step_weight()));
				assert!(used.all_lte(MigrationsMaxServiceWeight::get()));
				assert!(
					<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing(),
					"transactions are still held"
				);
			}

			next_block();
			assert!(!<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
			assert!(Cursor::<Runtime>::get().is_none());
			assert!(System::events().iter().any(|e| matches!(
				e.event,
				RuntimeEvent::MultiBlockMigrations(MigrationsEvent::UpgradeCompleted)
			)));

			// Done: nothing runs again.
			assert!(next_block().all_lt(NoopStepped::step_weight()));
		});
	}

	// NFR-9, DEC-30: a migration that has run is recorded, and a later upgrade doesn't run it again.
	#[test]
	fn a_finished_migration_does_not_run_again_on_a_later_upgrade() {
		TestExternalities::default().execute_with(|| {
			NoopSteps::set(&1);
			the_upgrade_block();
			next_block();
			assert!(!<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());

			// The upgrade onboards the list again; the next step skips what the history holds.
			<MultiBlockMigrations as OnRuntimeUpgrade>::on_runtime_upgrade();
			assert!(next_block().all_lt(NoopStepped::step_weight()));
			assert!(!<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
		});
	}

	// DEC-30, RISK-14: a failing migration freezes the chain: it stays stuck, and transactions are
	// never let in, until governance clears the cursor.
	#[test]
	fn a_failing_migration_freezes_the_chain() {
		TestExternalities::default().execute_with(|| {
			NoopSteps::set(&4);
			NoopFailsAt::set(&Some(1));
			the_upgrade_block();

			next_block();
			assert_eq!(MultiBlockMigrations::ongoing_status(), MbmIsOngoing::Yes);
			next_block();

			assert_eq!(Cursor::<Runtime>::get(), Some(MigrationCursor::Stuck));
			assert_eq!(MultiBlockMigrations::ongoing_status(), MbmIsOngoing::Stuck);
			assert!(<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
			assert!(System::events().iter().any(|e| matches!(
				e.event,
				RuntimeEvent::MultiBlockMigrations(MigrationsEvent::MigrationFailed { .. })
			)));

			// It stays frozen however many blocks go by.
			for _ in 0..5 {
				next_block();
				assert_eq!(Cursor::<Runtime>::get(), Some(MigrationCursor::Stuck));
				assert!(<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
			}
		});
	}

	// DEC-30, M-3: root can clear a stuck cursor, but a frozen chain only has inherents, so Root is
	// unreachable inside Kreivo: no signed transaction, so no referendum. Recovery needs a Kusama
	// referendum that force-sets para 2281's code to a runtime whose `SingleBlockMigrations`
	// clears `MultiBlockMigrations::Cursor` (see F-12 NOTES). Owner ruling: `m3-mbm-failure`.
	#[test]
	fn a_stuck_cursor_clears_only_with_root_which_a_frozen_chain_cannot_reach() {
		TestExternalities::default().execute_with(|| {
			NoopFailsAt::set(&Some(0));
			the_upgrade_block();
			next_block();
			assert_eq!(MultiBlockMigrations::ongoing_status(), MbmIsOngoing::Stuck);

			frame_support::assert_ok!(MultiBlockMigrations::force_set_cursor(RuntimeOrigin::root(), None));
			assert!(!<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
		});
	}
}
