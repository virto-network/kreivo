use super::*;

pub(super) use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
pub(super) use frame_system_benchmarking::Pallet as SystemBench;

use super::{xcm_config, Balances, PriceForParentDelivery, Runtime, UNITS};
use crate::RuntimeCall;
use frame_benchmarking::BenchmarkError;
use frame_support::parameter_types;
use pallet_xcm_benchmarks::asset_instance_from;
use xcm::latest::prelude::{
	Asset, AssetId, Assets as XcmAssets, Fungible, GeneralIndex, Here, InteriorLocation, Junction, Location, NetworkId,
	NonFungible, Response,
};
use xcm_config::RelayLocation;

impl frame_system_benchmarking::Config for Runtime {
	fn setup_set_code_requirements(code: &Vec<u8>) -> Result<(), BenchmarkError> {
		ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
		Ok(())
	}

	fn verify_set_code() {
		System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
	}
}

impl cumulus_pallet_session_benchmarking::Config for Runtime {}

parameter_types! {
	pub ExistentialDepositAsset: Option<Asset> = Some((
		RelayLocation::get(),
		ExistentialDeposit::get()
	).into());
}

impl pallet_xcm_benchmarks::Config for Runtime {
	type XcmConfig = xcm_config::XcmConfig;
	type AccountIdConverter = xcm_config::LocationToAccountId;
	type DeliveryHelper = cumulus_primitives_utility::ToParentDeliveryHelper<
		xcm_config::XcmConfig,
		ExistentialDepositAsset,
		PriceForParentDelivery,
	>;

	fn valid_destination() -> Result<Location, BenchmarkError> {
		Ok(RelayLocation::get())
	}
	fn worst_case_holding(depositable_count: u32) -> XcmAssets {
		// A mix of fungible, non-fungible, and concrete assets.
		let holding_non_fungibles = xcm_config::MaxAssetsIntoHolding::get() / 2 - depositable_count;
		let holding_fungibles = holding_non_fungibles.saturating_sub(1);
		let fungibles_amount: u128 = 100;
		let mut assets = (0..holding_fungibles)
			.map(|i| (AssetId(GeneralIndex(i as u128).into()), fungibles_amount * i as u128).into())
			.chain(core::iter::once((AssetId(Here.into()), u128::MAX).into()))
			.chain((0..holding_non_fungibles).map(|i| {
				(
					AssetId(GeneralIndex(i as u128).into()),
					NonFungible(asset_instance_from(i)),
				)
					.into()
			}))
			.collect::<Vec<_>>();

		assets.push((RelayLocation::get(), 1_000_000 * UNITS).into());
		assets.into()
	}
}

parameter_types! {
	pub const TrustedTeleporter: Option<(Location, Asset)> = Some((
		RelayLocation::get(),
		Asset { fun: Fungible(UNITS), id: AssetId(RelayLocation::get()) },
	));
	pub const TrustedReserve: Option<(Location, Asset)> = None;
	pub const CheckedAccount: Option<(AccountId, xcm_builder::MintLocation)> = None;

}

impl pallet_xcm_benchmarks::fungible::Config for Runtime {
	type TransactAsset = Balances;
	type CheckedAccount = CheckedAccount;
	type TrustedTeleporter = TrustedTeleporter;
	type TrustedReserve = TrustedReserve;

	fn get_asset() -> Asset {
		(RelayLocation::get(), UNITS).into()
	}
}

impl pallet_xcm_benchmarks::generic::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type TransactAsset = Balances;

	fn worst_case_response() -> (u64, Response) {
		(0u64, Response::Version(Default::default()))
	}

	fn worst_case_asset_exchange() -> Result<(XcmAssets, XcmAssets), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}

	fn universal_alias() -> Result<(Location, Junction), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}

	fn transact_origin_and_runtime_call() -> Result<(Location, RuntimeCall), BenchmarkError> {
		Ok((
			RelayLocation::get(),
			frame_system::Call::remark_with_event { remark: vec![] }.into(),
		))
	}

	fn subscribe_origin() -> Result<Location, BenchmarkError> {
		Ok(RelayLocation::get())
	}

	fn claimable_asset() -> Result<(Location, Location, XcmAssets), BenchmarkError> {
		let origin = RelayLocation::get();
		let assets: XcmAssets = (AssetId(RelayLocation::get()), 1_000 * UNITS).into();
		let ticket = Here.into();
		Ok((origin, ticket, assets))
	}

	fn fee_asset() -> Result<Asset, BenchmarkError> {
		Ok((RelayLocation::get(), 1_000_000 * UNITS).into())
	}

	fn unlockable_asset() -> Result<(Location, Location, Asset), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}

	fn export_message_origin_and_destination() -> Result<(Location, NetworkId, InteriorLocation), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}

	fn alias_origin() -> Result<(Location, Location), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}
}
