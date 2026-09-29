//! Migrations owned by the runtime, rather than by any single pallet.

use super::*;

use alloc::collections::btree_map::BTreeMap;
use frame_support::traits::{
	schedule::{v3::Anon, DispatchTime},
	OnRuntimeUpgrade, StorePreimage,
};
use pallet_referenda::{ReferendumInfo, ReferendumInfoFor};
use pallet_scheduler::{Agenda, IncompleteSince, Lookup, Retries, ScheduledOf, TaskAddress};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::BlockNumberProvider;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "runtime::kreivo::migrations";

/// Where the scheduler switched clocks: the last parachain block counted on the parachain clock,
/// and the relay chain block the scheduler saw then.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ClockSwitch {
	/// The parachain block number when the migration ran.
	pub parachain: BlockNumber,
	/// The relay chain block number the scheduler saw when the migration ran.
	pub relay_chain: BlockNumber,
}

impl ClockSwitch {
	/// The clocks as the scheduler sees them now.
	fn now() -> Self {
		Self {
			parachain: frame_system::Pallet::<Runtime>::block_number(),
			// Exactly what the scheduler reads: `RelaychainDataProvider` reads the relay parent of
			// the last block (from `ValidationData`, which `ParachainSystem` only clears in its
			// `on_initialize`, or `LastRelayChainBlockNumber`, set to the same value in its
			// `on_finalize`).
			relay_chain: RelaychainData::current_block_number(),
		}
	}

	/// A block number on the parachain clock, on the relay chain clock: the same distance from
	/// `now` (one relay chain block per parachain block, the rate Kreivo had on 0.16).
	fn on_relay_chain(&self, block: BlockNumber) -> BlockNumber {
		if block >= self.parachain {
			self.relay_chain.saturating_add(block - self.parachain)
		} else {
			self.relay_chain.saturating_sub(self.parachain - block)
		}
	}

	/// The first agenda the scheduler hasn't serviced yet on the parachain clock. The ones
	/// before it are unreachable: the scheduler never goes back to them.
	fn first_pending_agenda(&self) -> BlockNumber {
		let next = self.parachain.saturating_add(1);
		IncompleteSince::<Runtime>::get().unwrap_or(next).min(next)
	}

	/// Where an agenda pending on the parachain clock goes on the relay chain clock. Those that
	/// are already due (a backlog the scheduler didn't get to) go to the next block it services.
	fn destination(&self, when: BlockNumber) -> BlockNumber {
		self.relay_chain
			.saturating_add(when.saturating_sub(self.parachain).max(1))
	}
}

/// Set once [`SchedulerToRelayChainClock`] has run, with the blocks where the clocks switched.
#[frame_support::storage_alias(verbatim)]
pub type SchedulerClockSwitch = StorageValue<KreivoMigrations, ClockSwitch>;

type TaskAddressOf = TaskAddress<BlockNumber>;

/// Moves what the scheduler and referenda keep on the parachain clock to the relay chain clock.
///
/// 0.17.0 switches `pallet_scheduler`, both `pallet_referenda` instances and every pallet that
/// schedules through them (e.g. `pallet_pass`) from `System` to `RelaychainData` as their
/// `BlockNumberProvider`. Kreivo's relay chain block number is ~3.6M blocks *behind* its
/// parachain block number, so, left alone:
///
/// - the agendas already in storage are keyed by parachain block numbers, and would only come
///   due when the relay chain reaches them, ~250 days later. That's every pass session's expiry
///   (`pallet_pass` gives a session its lifetime only through a scheduled `remove_session_key`),
///   every referendum's alarm, enactment and payment cancellation;
/// - referenda keep `submitted`, `deciding.since`, `deciding.confirming` and their alarm on the
///   parachain clock, so their periods would be off by those ~3.6M blocks.
///
/// This migration, run once:
/// 1. removes the agendas the scheduler would never reach on the parachain clock (before its
///    `IncompleteSince`),
/// 2. moves every other agenda to the same distance from the relay chain block number (the
///    backlog, if any, to the next block the scheduler services), merging agendas that land on
///    the same block and spilling what doesn't fit (`MaxScheduledPerBlock`) to the next blocks,
/// 3. rewrites `Lookup` and `Retries` to the tasks' new addresses,
/// 4. moves every ongoing referendum's block numbers to the relay chain clock, keeping the time
///    elapsed and left, and points its alarm to the moved task (or sets it again if it had none),
/// 5. has the scheduler start at the next relay chain block.
///
/// It records where the clocks switched in [`SchedulerClockSwitch`], and does nothing if that's
/// already set.
pub struct SchedulerToRelayChainClock;

/// What the migration did.
#[derive(Default, Debug, PartialEq, Eq, Encode, Decode)]
pub struct MigrationSummary {
	/// Agendas removed because the scheduler would never reach them.
	pub removed_agendas: u32,
	/// Tasks removed with them.
	pub removed_tasks: u32,
	/// Tasks moved to the relay chain clock.
	pub moved_tasks: u32,
	/// Tasks that didn't fit in their agenda, and went to a later one.
	pub spilled_tasks: u32,
	/// `Lookup` entries removed because their task was removed or doesn't exist.
	pub removed_lookups: u32,
	/// `Retries` entries removed because their task was removed or doesn't exist.
	pub removed_retries: u32,
	/// Ongoing referenda moved to the relay chain clock.
	pub remapped_referenda: u32,
	/// Referenda whose alarm wasn't a pending task, and got a new one.
	pub rearmed_referenda: u32,
}

/// Counts storage accesses, for the weight.
#[derive(Default)]
struct Accesses {
	reads: u64,
	writes: u64,
}

impl Accesses {
	fn weight(&self) -> Weight {
		<Runtime as frame_system::Config>::DbWeight::get().reads_writes(self.reads, self.writes)
	}
}

impl SchedulerToRelayChainClock {
	/// Moves the agendas, and returns where each task went.
	fn migrate_agendas(
		clocks: &ClockSwitch,
		summary: &mut MigrationSummary,
		db: &mut Accesses,
	) -> BTreeMap<TaskAddressOf, TaskAddressOf> {
		let first_pending = clocks.first_pending_agenda();
		db.reads += 1;

		// Take every agenda out, so the ones we write can't be confused with the old ones.
		let keys = Agenda::<Runtime>::iter_keys().collect::<Vec<_>>();
		let mut pending = Vec::new();
		for when in keys {
			db.reads += 1;
			let agenda = Agenda::<Runtime>::try_get(when);
			if when < first_pending {
				summary.removed_agendas += 1;
				summary.removed_tasks += agenda.map_or(0, |a| a.iter().flatten().count() as u32);
				Agenda::<Runtime>::remove(when);
				db.writes += 1;
				continue;
			}
			match agenda {
				Ok(agenda) => {
					Agenda::<Runtime>::remove(when);
					db.writes += 1;
					pending.push((when, agenda));
				}
				// Can't be moved: leave it where it is, for `try-runtime` to catch.
				Err(()) => log::error!(target: LOG_TARGET, "agenda #{when} can't be decoded, left in place"),
			}
		}
		pending.sort_by_key(|(when, _)| *when);

		let max_per_block = <Runtime as pallet_scheduler::Config>::MaxScheduledPerBlock::get() as usize;
		let mut agendas = BTreeMap::<BlockNumber, Vec<Option<ScheduledOf<Runtime>>>>::new();
		let mut moved = BTreeMap::new();
		for (when, agenda) in pending {
			let mut destination = clocks.destination(when);
			for (index, task) in agenda.into_iter().enumerate() {
				// A cancelled task leaves an empty slot. There's nothing to move.
				let Some(task) = task else { continue };
				let wanted = destination;
				loop {
					let agenda = agendas.entry(destination).or_insert_with(|| {
						// An agenda we left in place, since it couldn't be decoded, can't take
						// more tasks.
						db.reads += 1;
						if Agenda::<Runtime>::contains_key(destination) {
							Agenda::<Runtime>::try_get(destination)
								.map(|agenda| agenda.into_inner())
								.unwrap_or_else(|_| vec![None; max_per_block])
						} else {
							Vec::new()
						}
					});
					if agenda.len() < max_per_block {
						agenda.push(Some(task));
						let new_address = (destination, agenda.len() as u32 - 1);
						moved.insert((when, index as u32), new_address);
						summary.moved_tasks += 1;
						if destination != wanted {
							summary.spilled_tasks += 1;
						}
						break;
					}
					if destination == BlockNumber::MAX {
						log::error!(target: LOG_TARGET, "no room for task ({when}, {index}), dropped");
						break;
					}
					destination += 1;
				}
			}
		}

		for (when, agenda) in agendas {
			if agenda.iter().all(Option::is_none) {
				continue;
			}
			// No agenda is longer than `MaxScheduledPerBlock`.
			Agenda::<Runtime>::insert(when, BoundedVec::truncate_from(agenda));
			db.writes += 1;
		}

		moved
	}

	/// Points `Lookup` and `Retries` to the tasks' new addresses.
	fn migrate_task_indexes(
		moved: &BTreeMap<TaskAddressOf, TaskAddressOf>,
		summary: &mut MigrationSummary,
		db: &mut Accesses,
	) {
		for (name, address) in Lookup::<Runtime>::iter().collect::<Vec<_>>() {
			db.reads += 1;
			match moved.get(&address) {
				Some(new_address) => Lookup::<Runtime>::insert(name, new_address),
				None => {
					summary.removed_lookups += 1;
					Lookup::<Runtime>::remove(name);
				}
			}
			db.writes += 1;
		}

		for (address, retry) in Retries::<Runtime>::drain().collect::<Vec<_>>() {
			db.reads += 1;
			db.writes += 1;
			match moved.get(&address) {
				Some(new_address) => {
					Retries::<Runtime>::insert(new_address, retry);
					db.writes += 1;
				}
				None => summary.removed_retries += 1,
			}
		}
	}

	/// Moves the ongoing referenda of an instance to the relay chain clock.
	fn migrate_referenda<I: 'static>(
		clocks: &ClockSwitch,
		moved: &BTreeMap<TaskAddressOf, TaskAddressOf>,
		summary: &mut MigrationSummary,
		db: &mut Accesses,
	) where
		Runtime: pallet_referenda::Config<
			I,
			Scheduler = Scheduler,
			BlockNumberProvider = RelaychainData,
			RuntimeCall = RuntimeCall,
		>,
		RuntimeCall: From<pallet_referenda::Call<Runtime, I>>,
	{
		let ongoing = ReferendumInfoFor::<Runtime, I>::iter()
			.filter_map(|(index, info)| {
				db.reads += 1;
				match info {
					ReferendumInfo::Ongoing(status) => Some((index, status)),
					_ => None,
				}
			})
			.collect::<Vec<_>>();

		for (index, mut status) in ongoing {
			status.submitted = clocks.on_relay_chain(status.submitted);
			if let DispatchTime::At(when) = status.enactment {
				status.enactment = DispatchTime::At(clocks.on_relay_chain(when));
			}
			if let Some(deciding) = status.deciding.as_mut() {
				deciding.since = clocks.on_relay_chain(deciding.since);
				deciding.confirming = deciding.confirming.map(|when| clocks.on_relay_chain(when));
			}
			if let Some((when, address)) = status.alarm {
				status.alarm = match moved.get(&address) {
					Some(&new_address) => Some((new_address.0, new_address)),
					// The alarm wasn't a pending task: without a new one, the referendum would
					// never be serviced again.
					None => {
						summary.rearmed_referenda += 1;
						Self::set_alarm::<I>(
							index,
							clocks.on_relay_chain(when).max(clocks.relay_chain.saturating_add(1)),
							db,
						)
					}
				};
			}
			ReferendumInfoFor::<Runtime, I>::insert(index, ReferendumInfo::Ongoing(status));
			db.writes += 1;
			summary.remapped_referenda += 1;
		}
	}

	/// Schedules a referendum's alarm, as `pallet_referenda` does.
	fn set_alarm<I: 'static>(
		index: pallet_referenda::ReferendumIndex,
		when: BlockNumber,
		db: &mut Accesses,
	) -> Option<(BlockNumber, TaskAddressOf)>
	where
		Runtime: pallet_referenda::Config<
			I,
			Scheduler = Scheduler,
			BlockNumberProvider = RelaychainData,
			RuntimeCall = RuntimeCall,
		>,
		RuntimeCall: From<pallet_referenda::Call<Runtime, I>>,
	{
		let call = RuntimeCall::from(pallet_referenda::Call::<Runtime, I>::nudge_referendum { index });
		let call = <Runtime as pallet_referenda::Config<I>>::Preimages::bound(call).ok()?;
		// Like `pallet_referenda`, when there's no room in an agenda, try the next one.
		(when..when.saturating_add(10)).find_map(|when| {
			db.reads += 1;
			db.writes += 1;
			<Scheduler as Anon<_, _, _>>::schedule(
				DispatchTime::At(when),
				None,
				128u8,
				frame_system::RawOrigin::Root.into(),
				call.clone(),
			)
			.ok()
			.map(|address| (when, address))
		})
	}

	/// Does the migration, and says what it did.
	pub fn migrate() -> (Weight, Option<MigrationSummary>) {
		let mut db = Accesses { reads: 1, writes: 0 };
		if SchedulerClockSwitch::exists() {
			log::info!(target: LOG_TARGET, "the scheduler already runs on the relay chain clock");
			return (db.weight(), None);
		}

		let clocks = ClockSwitch::now();
		db.reads += 3;
		log::info!(
			target: LOG_TARGET,
			"moving the scheduler from parachain block #{} to relay chain block #{}",
			clocks.parachain,
			clocks.relay_chain,
		);

		let mut summary = MigrationSummary::default();
		let moved = Self::migrate_agendas(&clocks, &mut summary, &mut db);
		Self::migrate_task_indexes(&moved, &mut summary, &mut db);
		// The scheduler services from the next relay chain block: every agenda we wrote is there
		// or after it.
		IncompleteSince::<Runtime>::put(clocks.relay_chain.saturating_add(1));
		db.writes += 1;

		Self::migrate_referenda::<pallet_referenda::Instance1>(&clocks, &moved, &mut summary, &mut db);
		Self::migrate_referenda::<pallet_referenda::Instance2>(&clocks, &moved, &mut summary, &mut db);

		SchedulerClockSwitch::put(clocks);
		db.writes += 1;

		log::info!(target: LOG_TARGET, "the scheduler runs on the relay chain clock: {summary:?}");
		(db.weight(), Some(summary))
	}
}

impl OnRuntimeUpgrade for SchedulerToRelayChainClock {
	fn on_runtime_upgrade() -> Weight {
		Self::migrate().0
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		if SchedulerClockSwitch::exists() {
			return Ok(None::<try_runtime::PreUpgrade>.encode());
		}
		Ok(Some(try_runtime::PreUpgrade::read()).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		let Some(before) = Option::<try_runtime::PreUpgrade>::decode(&mut &state[..])
			.map_err(|_| TryRuntimeError::Other("can't decode the pre-upgrade state"))?
		else {
			return Ok(());
		};
		try_runtime::check(before)
	}
}

#[cfg(feature = "try-runtime")]
mod try_runtime {
	use super::*;
	use frame_support::ensure;

	/// What the migration should keep.
	#[derive(Debug, Encode, Decode)]
	pub struct PreUpgrade {
		clocks: ClockSwitch,
		/// Tasks in agendas the scheduler would still service.
		pending_tasks: u32,
		/// Ongoing referenda whose alarm isn't one of those tasks: they get a new one.
		dangling_alarms: u32,
		/// Ongoing referenda, per instance.
		ongoing_referenda: (u32, u32),
	}

	type KreivoReferendaInstance = pallet_referenda::Instance1;
	type CommunityReferendaInstance = pallet_referenda::Instance2;

	fn is_pending_task(address: &TaskAddressOf) -> bool {
		Agenda::<Runtime>::get(address.0)
			.get(address.1 as usize)
			.is_some_and(Option::is_some)
	}

	fn ongoing_alarms<I: 'static>() -> Vec<Option<(BlockNumber, TaskAddressOf)>>
	where
		Runtime: pallet_referenda::Config<
			I,
			Scheduler = Scheduler,
			BlockNumberProvider = RelaychainData,
			RuntimeCall = RuntimeCall,
		>,
	{
		ReferendumInfoFor::<Runtime, I>::iter_values()
			.filter_map(|info| match info {
				ReferendumInfo::Ongoing(status) => Some(status.alarm),
				_ => None,
			})
			.collect()
	}

	impl PreUpgrade {
		pub fn read() -> Self {
			let clocks = ClockSwitch::now();
			let first_pending = clocks.first_pending_agenda();
			let pending_tasks = Agenda::<Runtime>::iter()
				.filter(|(when, _)| *when >= first_pending)
				.map(|(_, agenda)| agenda.iter().flatten().count() as u32)
				.sum();

			let kreivo = ongoing_alarms::<KreivoReferendaInstance>();
			let community = ongoing_alarms::<CommunityReferendaInstance>();
			let dangling_alarms = kreivo
				.iter()
				.chain(community.iter())
				.flatten()
				.filter(|(_, address)| address.0 < first_pending || !is_pending_task(address))
				.count() as u32;

			let before = Self {
				clocks,
				pending_tasks,
				dangling_alarms,
				ongoing_referenda: (kreivo.len() as u32, community.len() as u32),
			};
			log::info!(target: LOG_TARGET, "before the clock switch: {before:?}");
			before
		}
	}

	pub fn check(before: PreUpgrade) -> Result<(), TryRuntimeError> {
		let clocks = SchedulerClockSwitch::get().ok_or("the clock switch wasn't recorded")?;
		ensure!(clocks == before.clocks, "the clocks moved during the migration");
		ensure!(
			IncompleteSince::<Runtime>::get() == Some(clocks.relay_chain + 1),
			"the scheduler doesn't start at the next relay chain block"
		);

		// Every agenda is on the relay chain clock, and no task was lost.
		let mut tasks = 0u32;
		for (when, agenda) in Agenda::<Runtime>::iter() {
			ensure!(
				when > clocks.relay_chain,
				"an agenda was left behind the relay chain block number"
			);
			tasks += agenda.iter().flatten().count() as u32;
		}
		ensure!(
			tasks == before.pending_tasks + before.dangling_alarms,
			"the pending tasks weren't all moved"
		);

		for (name, address) in Lookup::<Runtime>::iter() {
			let task = Agenda::<Runtime>::get(address.0)
				.get(address.1 as usize)
				.cloned()
				.flatten()
				.ok_or("a `Lookup` entry points to no task")?;
			ensure!(task.maybe_id == Some(name), "a `Lookup` entry points to another task");
		}
		for address in Retries::<Runtime>::iter_keys() {
			ensure!(is_pending_task(&address), "a `Retries` entry points to no task");
		}

		let kreivo = ongoing_alarms::<KreivoReferendaInstance>();
		let community = ongoing_alarms::<CommunityReferendaInstance>();
		ensure!(
			(kreivo.len() as u32, community.len() as u32) == before.ongoing_referenda,
			"the ongoing referenda changed"
		);
		for (when, address) in kreivo.into_iter().chain(community).flatten() {
			ensure!(when == address.0, "a referendum's alarm isn't when its task is");
			ensure!(is_pending_task(&address), "a referendum's alarm points to no task");
		}

		log::info!(target: LOG_TARGET, "the scheduler runs on the relay chain clock, checks passed");
		Ok(())
	}
}
