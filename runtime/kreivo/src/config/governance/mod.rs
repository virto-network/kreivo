use super::*;

pub mod origins;

use crate::{Balances, Runtime, RuntimeEvent};
use frame_support::parameter_types;
use frame_support::traits::tokens::{PayFromAccount, UnityAssetBalanceConversion};
pub use origins::*;
use parachains_common::Balance;
use sp_runtime::{traits::IdentityLookup, Permill};

// #[runtime::pallet_index(50)]
// pub type Treasury
parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 2000 * CENTS;
	pub const ProposalBondMaximum: Balance = GRAND;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 100 * CENTS;
	pub const DataDepositPerByte: Balance = CENTS;
	pub const MaxApprovals: u32 = 100;
	pub const MaxAuthorities: u32 = 100_000;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
	pub TreasuryAccount: AccountId = Treasury::account_id();
	pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;
}

impl pallet_treasury::Config for Runtime {
	type Currency = Balances;
	type RejectOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type PalletId = TreasuryPalletId;
	type BurnDestination = ();
	type WeightInfo = weights::pallet_treasury::WeightInfo<Self>;
	type SpendFunds = ();
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
	type AssetKind = ();
	type Beneficiary = Self::AccountId;
	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = PayoutSpendPeriod;
	#[cfg(feature = "runtime-benchmarks")]
	/// TODO: fix this benchmark helper in next release. We can proceed with the
	/// empty implementation. type BenchmarkHelper =
	/// polkadot_runtime_common::impls::benchmarks::TreasuryArguments;
	type BenchmarkHelper = ();
}
