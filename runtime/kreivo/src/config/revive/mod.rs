use super::*;

#[cfg(not(feature = "zombienet"))]
use frame_system::EnsureRootWithSuccess;
#[cfg(feature = "zombienet")]
use frame_system::EnsureSigned;

parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub const DepositPerChildTrieItem: Balance = deposit(1, 0) / 100;
	pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(30);
	pub const ChainId: u64 = 2281;
}

impl pallet_revive::Config for Runtime {
	type Time = Timestamp;
	type Balance = Balance;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = pallet_revive::weights::SubstrateWeight<Self>;
	type Precompiles = ();
	// 10^(18 - 12) Eth is 10^18, Native is 10^12.
	type FindAuthor = <Runtime as pallet_authorship::Config>::FindAuthor;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type DepositPerChildTrieItem = DepositPerChildTrieItem;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type AddressMapper = pallet_revive::AccountId32Mapper<Self>;
	type UnsafeUnstableInterface = ConstBool<true>;
	type AllowEVMBytecode = ConstBool<true>;
	#[cfg(not(feature = "zombienet"))]
	type UploadOrigin = EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	#[cfg(feature = "zombienet")]
	type UploadOrigin = EnsureSigned<AccountId>;
	#[cfg(not(feature = "zombienet"))]
	type InstantiateOrigin = EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	#[cfg(feature = "zombienet")]
	type InstantiateOrigin = EnsureSigned<AccountId>;
	type RuntimeMemory = ConstU32<{ 128 * 1024 * 1024 }>;
	type PVFMemory = ConstU32<{ 512 * 1024 * 1024 }>;
	type ChainId = ChainId;
	type NativeToEthRatio = ConstU32<1_000_000>;
	type FeeInfo = pallet_revive::evm::fees::Info<Address, Signature, EthExtraImpl>;
	type MaxEthExtrinsicWeight = ();
	type DebugEnabled = ConstBool<false>;
	type GasScale = ConstU32<1000>;
}
