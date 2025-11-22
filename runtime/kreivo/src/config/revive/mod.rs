use super::*;

#[cfg(not(feature = "zombienet"))]
use frame_system::EnsureRootWithSuccess;
#[cfg(feature = "zombienet")]
use frame_system::EnsureSigned;

// Precompiles
use pallet_assets_precompiles::{InlineIdConfig, ERC20};
use pallet_foo_precompiles::Foo;
use pallet_xcm::precompiles::XcmPrecompile;

parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(30);
	pub const ChainId: u64 = 2281;
}

impl pallet_revive::Config for Runtime {
	type Time = Timestamp;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = pallet_revive::weights::SubstrateWeight<Self>;
	type Precompiles = (
		ERC20<Self, InlineIdConfig<0x120>, KreivoAssetsInstance>,
		Foo<Self, pallet_foo_precompiles::InlineIdConfig<0x0F>>,
		XcmPrecompile<Self>,
	);
	type AddressMapper = pallet_revive::AccountId32Mapper<Self>;
	type RuntimeMemory = ConstU32<{ 128 * 1024 * 1024 }>;
	type PVFMemory = ConstU32<{ 512 * 1024 * 1024 }>;
	type UnsafeUnstableInterface = ConstBool<true>;
	#[cfg(not(feature = "zombienet"))]
	type UploadOrigin = EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	#[cfg(not(feature = "zombienet"))]
	type InstantiateOrigin = EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	#[cfg(feature = "zombienet")]
	type UploadOrigin = EnsureSigned<AccountId>;
	#[cfg(feature = "zombienet")]
	type InstantiateOrigin = EnsureSigned<AccountId>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type ChainId = ChainId;
	type NativeToEthRatio = ConstU32<1_000_000>; // 10^(18 - 12) Eth is 10^18, Native is 10^12.
	type EthGasEncoder = ();
	type FindAuthor = <Runtime as pallet_authorship::Config>::FindAuthor;
	type AllowEVMBytecode = ConstBool<true>; // Virto accepts EVM Bytecode (?)
}
