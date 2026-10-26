//! Re-keys the community referenda's storage from `u16` track ids to `u32` ones.
//!
//! frame-contrib 3.x makes a community track's id `combine(community_id: u16, sub: u16)` (a `u32`,
//! see `pallet_referenda_tracks::SplitId`), and `pallet_referenda_tracks::migration::MigrateV0ToV1`
//! moves old track `n` to `combine(n, 0)`. `pallet_referenda` (the `CommunityReferenda` instance)
//! keeps its own storage keyed by the track id, which this migration moves the same way:
//!
//! 1. [`Phase::DecidingCount`]: one `DecidingCount` entry per unit moves to the new key.
//! 2. [`Phase::TrackQueue`]: one `TrackQueue` entry per unit moves to the new key.
//! 3. [`Phase::Referenda`]: one `ReferendumInfoFor` entry per unit. An `Ongoing` one carries its
//!    track, which is rewritten as a `u32` (`ReferendumInfoFor` is keyed by the referendum index,
//!    so its key does not change). The other variants carry no track, and are left alone.
//!
//! # The contract
//!
//! It is a [`SteppedMigration`], with the contract of frame-contrib's migrations. One unit is one
//! entry. Before each unit it calls `meter.try_consume(unit)`; when it does not fit, it stops and
//! returns `Ok(Some(cursor))`. It returns [`SteppedMigrationError::InsufficientWeight`] only when
//! not even one unit fits an empty meter. Anything odd (a key or a value that does not decode, a
//! new key that is already taken) is skipped and logged, never an error. The cursor is a phase and
//! a bounded raw storage key.
//!
//! Old and new `DecidingCount` and `TrackQueue` keys share one storage prefix (`Twox64Concat`
//! over the encoded id), so the phases walk raw keys and tell the shapes apart by the exact length
//! of the key and by its hash. A key in the new shape is left alone, which makes it idempotent. A
//! `ReferendumInfoFor` value is told apart by decoding it entirely, as the old shape first.
//!
//! Must run in the same upgrade as `pallet_referenda_tracks::migration::MigrateV0ToV1` (it is
//! registered right after it). Until it has run, an ongoing referendum's track does not exist.

use crate::{config::communities::governance::CommunityReferendaInstance as I, AccountId, BlockNumber, Runtime};
use alloc::vec::Vec;
use frame_support::{
	migrations::{MigrationId, SteppedMigration, SteppedMigrationError},
	pallet_prelude::{BoundedVec, ConstU32},
	storage::{storage_prefix, unhashed, KeyPrefixIterator},
	traits::PalletInfoAccess,
	weights::{Weight, WeightMeter},
};
use frame_support::{StorageHasher, Twox64Concat};
use pallet_referenda::{
	BalanceOf, BoundedCallOf, Pallet as Referenda, PalletsOriginOf, ReferendumInfo, ReferendumStatus,
	ScheduleAddressOf, TallyOf,
};
use pallet_referenda_tracks::SplitId;
use parity_scale_codec::{Decode, DecodeAll, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(feature = "try-runtime")]
use {alloc::collections::BTreeMap, pallet_referenda::ReferendumInfoFor, sp_runtime::TryRuntimeError};

const LOG_TARGET: &str = "runtime::kreivo::migrations::referenda_rekey";

/// The size of a storage prefix (pallet and storage name hashes).
const PREFIX_LEN: usize = 32;
/// The size of the `Twox64` part of a `Twox64Concat` key.
const TWOX_LEN: usize = 8;
/// The size of the `Blake2_128` part of a `Blake2_128Concat` key.
#[cfg(feature = "try-runtime")]
const BLAKE_LEN: usize = 16;

/// The most bytes of a raw storage key that a [`Cursor`] can hold.
pub const MAX_KEY_LEN: u32 = 256;

/// A raw storage key, bounded.
pub type CursorKey = BoundedVec<u8, ConstU32<MAX_KEY_LEN>>;

/// The phases of [`ReferendaTrackIds`], in order.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
	/// Moves the `DecidingCount` entries.
	DecidingCount,
	/// Moves the `TrackQueue` entries.
	TrackQueue,
	/// Rewrites the track of the ongoing referenda.
	Referenda,
}

/// The progress of [`ReferendaTrackIds`]: the phase, and the raw key of the last entry that was
/// processed in it (empty when the phase has just begun).
pub type Cursor = (Phase, CursorKey);

/// `ReferendumInfo` of the community referenda with the track id `Track`: `u16` before the
/// upgrade, `u32` after.
pub type Info<Track> = ReferendumInfo<
	Track,
	PalletsOriginOf<Runtime>,
	BlockNumber,
	BoundedCallOf<Runtime, I>,
	BalanceOf<Runtime, I>,
	TallyOf<Runtime, I>,
	AccountId,
	ScheduleAddressOf<Runtime, I>,
>;

/// The track id before the upgrade.
pub type OldTrackId = u16;
/// The track id after it: `combine(community_id, sub)`.
pub type NewTrackId = u32;

/// What a `TrackQueue` value is.
type Queue = BoundedVec<
	(u32, <Runtime as pallet_referenda::Config<I>>::Votes),
	<Runtime as pallet_referenda::Config<I>>::MaxQueued,
>;

/// The track that the old track `old` becomes: the first one of the group `old`, as
/// `pallet_referenda_tracks::migration::MigrateV0ToV1` does.
pub fn new_track(old: OldTrackId) -> NewTrackId {
	<NewTrackId as SplitId>::combine(old, 0)
}

/// The raw key that follows `from`, if it is under `prefix`.
fn next_raw_key(prefix: &[u8], from: &[u8]) -> Option<Vec<u8>> {
	let mut keys = KeyPrefixIterator::new(prefix.to_vec(), from.to_vec(), |_| Ok(()));
	keys.next()?;
	Some(keys.last_raw_key().to_vec())
}

/// Estimated proof size of one storage access, in bytes, besides the value: the key and the trie
/// nodes, doubled (as the memberships migration does).
const PROOF_PER_ACCESS: u64 = 1_000;
/// What a unit's code costs besides its storage accesses, in picoseconds of `ref_time`.
const EXECUTION_REF_TIME: u64 = 50_000_000;

/// A conservative bound of a unit that does `reads` reads and `writes` writes of values of at most
/// `value_len` bytes: Kreivo's `DbWeight` for the accesses, a constant for the code, and, for the
/// proof, every access at `PROOF_PER_ACCESS` plus the largest value (`MaxEncodedLen`). It is not
/// measured: it is to be replaced by a benchmark.
fn entry(reads: u64, writes: u64, value_len: usize) -> Weight {
	<Runtime as frame_system::Config>::DbWeight::get()
		.reads_writes(reads, writes)
		.saturating_add(Weight::from_parts(
			EXECUTION_REF_TIME,
			(reads + writes).saturating_mul(PROOF_PER_ACCESS.saturating_add(value_len as u64)),
		))
}

impl Phase {
	/// The phase after this one.
	pub fn next(self) -> Option<Self> {
		match self {
			Phase::DecidingCount => Some(Phase::TrackQueue),
			Phase::TrackQueue => Some(Phase::Referenda),
			Phase::Referenda => None,
		}
	}

	/// The weight of one unit of the phase. Every unit reads the next key, and the value; the first
	/// two also check the new key, write it, and kill the old one; the last one rewrites a value.
	pub fn unit(self) -> Weight {
		match self {
			Phase::DecidingCount => entry(3, 2, u32::max_encoded_len()),
			Phase::TrackQueue => entry(3, 2, Queue::max_encoded_len()),
			Phase::Referenda => entry(2, 1, Info::<NewTrackId>::max_encoded_len()),
		}
	}
}

/// The heaviest unit of any phase.
pub fn heaviest_unit() -> Weight {
	let mut phase = Some(Phase::DecidingCount);
	let mut heaviest = Weight::zero();
	while let Some(p) = phase {
		heaviest = heaviest.max(p.unit());
		phase = p.next();
	}
	heaviest
}

/// Re-keys `pallet_referenda`'s storage by track for the community referenda. See the
/// [module docs][self].
pub struct ReferendaTrackIds;

impl ReferendaTrackIds {
	fn prefix_of(phase: Phase) -> Vec<u8> {
		let name = match phase {
			Phase::DecidingCount => b"DecidingCount".as_slice(),
			Phase::TrackQueue => b"TrackQueue".as_slice(),
			Phase::Referenda => b"ReferendumInfoFor".as_slice(),
		};
		storage_prefix(Referenda::<Runtime, I>::name().as_bytes(), name).to_vec()
	}

	/// The raw key that follows `last` (or the first one of `prefix`), if it is under `prefix`.
	fn next_key(prefix: &[u8], last: &CursorKey) -> Option<Vec<u8>> {
		next_raw_key(prefix, if last.is_empty() { prefix } else { last })
	}

	fn to_cursor_key(raw: &[u8]) -> Result<CursorKey, SteppedMigrationError> {
		CursorKey::try_from(raw.to_vec()).map_err(|_| {
			log::error!(target: LOG_TARGET, "a storage key does not fit the cursor");
			SteppedMigrationError::Failed
		})
	}

	/// The track of a `Twox64Concat` raw key of the old shape (`u16`), or `None`. A key of the new
	/// shape is not logged: it is what a previous run, or the new code, wrote.
	fn old_track_of(raw_key: &[u8]) -> Option<OldTrackId> {
		let old_len = PREFIX_LEN + TWOX_LEN + OldTrackId::max_encoded_len();
		let new_len = PREFIX_LEN + TWOX_LEN + NewTrackId::max_encoded_len();
		if raw_key.len() == new_len {
			return None;
		}
		let decoded = (raw_key.len() == old_len)
			.then(|| {
				let (_, encoded) = raw_key[PREFIX_LEN..].split_at(TWOX_LEN);
				// `Twox64Concat` is the hash of the encoded id, then the encoded id.
				(Twox64Concat::hash(encoded) == raw_key[PREFIX_LEN..])
					.then(|| OldTrackId::decode_all(&mut &encoded[..]).ok())
					.flatten()
			})
			.flatten();
		if decoded.is_none() {
			log::warn!(target: LOG_TARGET, "skipping a key that is not a track: {:?}", raw_key);
		}
		decoded
	}

	/// The raw key of the same map, under `prefix`, for the track `track`.
	fn key_of(prefix: &[u8], track: NewTrackId) -> Vec<u8> {
		let encoded = track.encode();
		[prefix, &Twox64Concat::hash(&encoded)[..]].concat()
	}

	/// Moves one entry to the key of the new track, if `valid` accepts its value.
	fn move_entry(raw_key: &[u8], prefix: &[u8], valid: impl Fn(&[u8]) -> bool) {
		let Some(old) = Self::old_track_of(raw_key) else {
			return;
		};
		let Some(value) = unhashed::get_raw(raw_key) else {
			return;
		};
		if !valid(&value) {
			log::warn!(target: LOG_TARGET, "skipping the value of track {old}: it does not decode");
			return;
		}
		let new_key = Self::key_of(prefix, new_track(old));
		if unhashed::exists(&new_key) {
			log::warn!(target: LOG_TARGET, "skipping track {old}: its new key is taken");
			return;
		}
		unhashed::put_raw(&new_key, &value);
		unhashed::kill(raw_key);
	}

	/// Rewrites the track of an ongoing referendum, unless it is already a `u32`.
	fn rewrite_referendum(raw_key: &[u8]) {
		let Some(raw_value) = unhashed::get_raw(raw_key) else {
			return;
		};
		match Info::<OldTrackId>::decode_all(&mut &raw_value[..]) {
			Ok(ReferendumInfo::Ongoing(status)) => {
				let migrated = Info::<NewTrackId>::Ongoing(ReferendumStatus {
					track: new_track(status.track),
					origin: status.origin,
					proposal: status.proposal,
					enactment: status.enactment,
					submitted: status.submitted,
					submission_deposit: status.submission_deposit,
					decision_deposit: status.decision_deposit,
					deciding: status.deciding,
					tally: status.tally,
					in_queue: status.in_queue,
					alarm: status.alarm,
				});
				unhashed::put(raw_key, &migrated);
			}
			// Finished referenda carry no track: both shapes are the same bytes.
			Ok(_) => {}
			Err(_) if Info::<NewTrackId>::decode_all(&mut &raw_value[..]).is_ok() => {
				// Already migrated.
			}
			Err(e) => {
				log::warn!(target: LOG_TARGET, "skipping a referendum that does not decode: {e:?}");
			}
		}
	}
}

impl SteppedMigration for ReferendaTrackIds {
	type Cursor = Cursor;
	type Identifier = MigrationId<16>;

	fn id() -> Self::Identifier {
		MigrationId {
			pallet_id: *b"kreivo/rekey-ref",
			version_from: 0,
			version_to: 1,
		}
	}

	fn migrating_prefixes() -> Option<impl IntoIterator<Item = Vec<u8>>> {
		Some([Phase::DecidingCount, Phase::TrackQueue, Phase::Referenda].map(Self::prefix_of))
	}

	fn step(
		cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		let (mut phase, mut last) = cursor.unwrap_or((Phase::DecidingCount, CursorKey::default()));
		let mut progressed = false;

		loop {
			let required = phase.unit();
			if meter.try_consume(required).is_err() {
				// Not even one unit would fit an empty meter: waiting does not help.
				if !progressed && required.any_gt(meter.limit()) {
					return Err(SteppedMigrationError::InsufficientWeight { required });
				}
				return Ok(Some((phase, last)));
			}
			progressed = true;

			let prefix = Self::prefix_of(phase);
			match Self::next_key(&prefix, &last) {
				Some(raw_key) => {
					match phase {
						Phase::DecidingCount => {
							Self::move_entry(&raw_key, &prefix, |value| u32::decode_all(&mut &value[..]).is_ok())
						}
						Phase::TrackQueue => {
							Self::move_entry(&raw_key, &prefix, |value| Queue::decode_all(&mut &value[..]).is_ok())
						}
						Phase::Referenda => Self::rewrite_referendum(&raw_key),
					}
					last = Self::to_cursor_key(&raw_key)?;
				}
				None => match phase.next() {
					Some(next) => (phase, last) = (next, CursorKey::default()),
					None => return Ok(None),
				},
			}
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		Ok(Snapshot::take().encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		let before = Snapshot::decode(&mut &state[..])
			.map_err(|_| TryRuntimeError::Other("cannot decode the pre-upgrade state"))?;
		let after = Snapshot::take();

		frame_support::ensure!(
			after.old_counters.is_empty(),
			"a deciding count is still keyed by a u16"
		);
		frame_support::ensure!(after.old_queues.is_empty(), "a track queue is still keyed by a u16");
		frame_support::ensure!(
			after.old_ongoing.is_empty(),
			"an ongoing referendum still has a u16 track"
		);
		frame_support::ensure!(
			after.new_counters.len() == before.new_counters.len() + before.old_counters.len(),
			"deciding counts were added or removed"
		);
		frame_support::ensure!(
			after.new_queues.len() == before.new_queues.len() + before.old_queues.len(),
			"track queues were added or removed"
		);
		frame_support::ensure!(after.referenda == before.referenda, "referenda were added or removed");
		for (track, count) in &before.old_counters {
			frame_support::ensure!(
				after.new_counters.get(&new_track(*track)) == Some(count),
				"a deciding count changed"
			);
		}
		for (track, queue) in &before.old_queues {
			frame_support::ensure!(
				after.new_queues.get(&new_track(*track)) == Some(queue),
				"a track queue changed"
			);
		}
		for (index, track) in &before.old_ongoing {
			match ReferendumInfoFor::<Runtime, I>::get(index) {
				Some(ReferendumInfo::Ongoing(status)) => frame_support::ensure!(
					status.track == new_track(*track),
					"an ongoing referendum has not the combine of its old track"
				),
				_ => return Err(TryRuntimeError::Other("an ongoing referendum is gone")),
			}
		}
		Ok(())
	}
}

/// What the `try-runtime` checks compare before and after the migration.
#[cfg(feature = "try-runtime")]
#[derive(Encode, Decode)]
struct Snapshot {
	/// Entries of `DecidingCount` with a `u16` key, and with a `u32` key.
	old_counters: BTreeMap<OldTrackId, u32>,
	new_counters: BTreeMap<NewTrackId, u32>,
	/// The raw values of the entries of `TrackQueue`, by `u16` and by `u32` key.
	old_queues: BTreeMap<OldTrackId, Vec<u8>>,
	new_queues: BTreeMap<NewTrackId, Vec<u8>>,
	/// The ongoing referenda with a `u16` track, and the number of referenda.
	old_ongoing: BTreeMap<u32, OldTrackId>,
	referenda: u32,
}

#[cfg(feature = "try-runtime")]
impl Snapshot {
	fn take() -> Self {
		use frame_support::Blake2_128Concat;

		let raw_entries = |phase: Phase| {
			let prefix = ReferendaTrackIds::prefix_of(phase);
			let mut keys = Vec::new();
			let mut last = prefix.clone();
			while let Some(key) = next_raw_key(&prefix, &last) {
				last = key.clone();
				keys.push(key);
			}
			keys
		};
		let track_of = |raw_key: &[u8]| {
			let encoded = raw_key.get(PREFIX_LEN + TWOX_LEN..)?;
			(Twox64Concat::hash(encoded) == raw_key[PREFIX_LEN..]).then_some(encoded.to_vec())
		};

		let (mut old_counters, mut new_counters) = (BTreeMap::new(), BTreeMap::new());
		for key in raw_entries(Phase::DecidingCount) {
			let value = unhashed::get::<u32>(&key).unwrap_or_default();
			match track_of(&key).map(|encoded| (encoded.len(), encoded)) {
				Some((2, encoded)) => {
					old_counters.insert(OldTrackId::decode(&mut &encoded[..]).unwrap_or_default(), value);
				}
				Some((4, encoded)) => {
					new_counters.insert(NewTrackId::decode(&mut &encoded[..]).unwrap_or_default(), value);
				}
				_ => {}
			}
		}
		let (mut old_queues, mut new_queues) = (BTreeMap::new(), BTreeMap::new());
		for key in raw_entries(Phase::TrackQueue) {
			let value = unhashed::get_raw(&key).unwrap_or_default();
			match track_of(&key).map(|encoded| (encoded.len(), encoded)) {
				Some((2, encoded)) => {
					old_queues.insert(OldTrackId::decode(&mut &encoded[..]).unwrap_or_default(), value);
				}
				Some((4, encoded)) => {
					new_queues.insert(NewTrackId::decode(&mut &encoded[..]).unwrap_or_default(), value);
				}
				_ => {}
			}
		}
		let (mut old_ongoing, mut referenda) = (BTreeMap::new(), 0);
		for key in raw_entries(Phase::Referenda) {
			referenda += 1;
			let Some(raw) = unhashed::get_raw(&key) else { continue };
			let Some(index) = key
				.get(PREFIX_LEN + BLAKE_LEN..)
				.filter(|encoded| Blake2_128Concat::hash(encoded) == key[PREFIX_LEN..])
				.and_then(|encoded| u32::decode_all(&mut &encoded[..]).ok())
			else {
				continue;
			};
			if let Ok(ReferendumInfo::Ongoing(status)) = Info::<OldTrackId>::decode_all(&mut &raw[..]) {
				old_ongoing.insert(index, status.track);
			}
		}
		Self {
			old_counters,
			new_counters,
			old_queues,
			new_queues,
			old_ongoing,
			referenda,
		}
	}
}
