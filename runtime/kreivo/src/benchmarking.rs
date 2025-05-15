use super::*;
use frame_benchmarking::{BenchmarkBatch, BenchmarkConfig, BenchmarkList};
use frame_support::traits::{StorageInfo, StorageInfoTrait};

use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
use frame_system_benchmarking::Pallet as SystemBench;

type XcmBalances = pallet_xcm_benchmarks::fungible::Pallet<Runtime>;
type XcmGeneric = pallet_xcm_benchmarks::generic::Pallet<Runtime>;

frame_benchmarking::define_benchmarks!(
	// System support
	[frame_system, SystemBench::<Runtime>]
	[cumulus_pallet_parachain_system, ParachainSystem]
	[pallet_timestamp, Timestamp]
	[pallet_pass, Pass]

	// Monetary stuff
	[pallet_balances, Balances]
	[pallet_assets, Assets]
	[pallet_vesting, Vesting]
	[pallet_transaction_payment, TransactionPayment]
	[pallet_asset_tx_payment, AssetsTxPayment]
	[pallet_gas_transaction_payment, GasTxPayment]

	// Collator support
	[pallet_collator_selection, CollatorSelection]
	[pallet_session, SessionBench::<Runtime>]

	// XCM
	[cumulus_pallet_xcmp_queue, XcmpQueue]
	[pallet_message_queue, MessageQueue]
	// NOTE: Make sure you point to the individual modules below.
	[pallet_xcm_benchmarks::fungible, XcmBalances]
	[pallet_xcm_benchmarks::generic, XcmGeneric]

	// Utils
	[pallet_multisig, Multisig]
	[pallet_utility, Utility]
	[pallet_proxy, Proxy]
	[pallet_scheduler, Scheduler]
	[pallet_preimage, Preimage]

	// Governance
	[pallet_treasury, Treasury]
	[pallet_ranked_collective, KreivoCollective]
	[pallet_referenda, KreivoReferenda]

	// Virto Tooling
	[pallet_payments, Payments]

	// Communities at Kreivo
	[pallet_communities, Communities]
	[pallet_referenda_tracks, CommunityTracks]
	// [pallet_referenda, CommunityReferenda]
	[pallet_nfts, CommunityMemberships]
	[pallet_communities_manager, CommunitiesManager]

	// Contracts
	[pallet_contracts, Contracts]
);

pub(crate) fn benchmark_metadata(extra: bool) -> (Vec<BenchmarkList>, Vec<StorageInfo>) {
	// This is defined once again in dispatch_benchmark, because list_benchmarks!
	// and add_benchmarks! are macros exported by define_benchmarks! macros and
	// those types are referenced in that call.
	let mut list = Vec::<BenchmarkList>::new();
	list_benchmarks!(list, extra);

	let storage_info = AllPalletsWithSystem::storage_info();
	(list, storage_info)
}

pub(crate) fn dispatch_benchmark(config: BenchmarkConfig) -> Result<Vec<BenchmarkBatch>, String> {
	use frame_support::traits::WhitelistedStorageKeys;
	let whitelist = AllPalletsWithSystem::whitelisted_storage_keys();

	let mut batches = Vec::<BenchmarkBatch>::new();
	let params = (&config, &whitelist);
	add_benchmarks!(params, batches);

	Ok(batches)
}
