//! Kreivo, as an emulated parachain.

pub mod genesis;

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain, impl_xcm_helpers_for_parachain,
	impls::Parachain, xcm_emulator::decl_test_parachains,
};
use frame_support::traits::OnInitialize;

decl_test_parachains! {
	pub struct Kreivo {
		genesis = genesis::genesis(),
		on_init = {
			kreivo_runtime::AuraExt::on_initialize(1);
		},
		runtime = kreivo_runtime,
		core = {
			XcmpMessageHandler: kreivo_runtime::XcmpQueue,
			LocationToAccountId: kreivo_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: kreivo_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: kreivo_runtime::PolkadotXcm,
			Balances: kreivo_runtime::Balances,
			Assets: kreivo_runtime::Assets,
		}
	},
}

// Kreivo's asset ids are `FungibleAssetLocation`s, so the `u32`-keyed asset helpers don't apply.
impl_accounts_helpers_for_parachain!(Kreivo);
impl_assert_events_helpers_for_parachain!(Kreivo);
impl_xcm_helpers_for_parachain!(Kreivo);
