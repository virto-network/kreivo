//! XCM integration tests between Kreivo and the system chains, on `xcm-emulator`.
//!
//! The system chains are Westend's (the Kusama runtimes lag one SDK release behind Kreivo).
//! Kreivo only ever sees them through relative locations — the relay chain is `(1, Here)` and
//! Asset Hub is `(1, [Parachain(1000)])` — so the relay token plays KSM here.

pub mod chains;

#[cfg(test)]
mod tests;

pub use chains::{asset_hub_westend::AssetHubWestend, kreivo::Kreivo, westend::Westend};

use emulated_integration_tests_common::{
	accounts::{ALICE, BOB},
	xcm_emulator::{decl_test_networks, decl_test_sender_receiver_accounts_parameter_types},
};

decl_test_networks! {
	pub struct KreivoMockNet {
		relay_chain = Westend,
		parachains = vec![
			AssetHubWestend,
			Kreivo,
		],
		bridge = ()
	},
}

decl_test_sender_receiver_accounts_parameter_types! {
	WestendRelay { sender: ALICE, receiver: BOB },
	AssetHubWestendPara { sender: ALICE, receiver: BOB },
	KreivoPara { sender: ALICE, receiver: BOB }
}
