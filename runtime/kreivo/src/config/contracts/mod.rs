use crate::{
	deposit, AccountId, Balance, Balances, Perbill, Runtime, RuntimeCall, RuntimeEvent, RuntimeHoldReason, Timestamp,
	TreasuryAccount,
};
use frame_support::{
	parameter_types,
	traits::{ConstBool, ConstU32, EitherOf, Randomness},
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRootWithSuccess};
use pallet_balances::Call as BalancesCall;
use pallet_communities::origin::AsSignedByStaticCommunity;
use sp_core::ConstU16;

pub enum AllowBalancesCall {}

impl frame_support::traits::Contains<RuntimeCall> for AllowBalancesCall {
	fn contains(call: &RuntimeCall) -> bool {
		matches!(call, RuntimeCall::Balances(BalancesCall::transfer_allow_death { .. }))
	}
}

fn schedule<T: pallet_contracts::Config>() -> pallet_contracts::Schedule<T> {
	const MB: u32 = 1024 * 1024;
	pallet_contracts::Schedule {
		limits: pallet_contracts::Limits {
			validator_runtime_memory: 1024 * MB,
			// Current `max_storage_size`: 138 MB
			// Constraint: `runtime_memory <= validator_runtime_memory - 2 * max_storage_size`
			runtime_memory: 748 * MB,
			..Default::default()
		},
		..Default::default()
	}
}

// randomness-collective-flip is insecure. Provide dummy randomness as
// placeholder for the deprecated trait. https://github.com/paritytech/polkadot-sdk/blob/9bf1a5e23884921498b381728bfddaae93f83744/substrate/frame/contracts/mock-network/src/parachain/contracts_config.rs#L45
pub struct DummyRandomness<T: pallet_contracts::Config>(sp_std::marker::PhantomData<T>);

impl<T: pallet_contracts::Config> Randomness<T::Hash, BlockNumberFor<T>> for DummyRandomness<T> {
	fn random(_subject: &[u8]) -> (T::Hash, BlockNumberFor<T>) {
		(Default::default(), Default::default())
	}
}

parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub Schedule: pallet_contracts::Schedule<Runtime> = schedule::<Runtime>();
	pub const DefaultDepositLimit: Balance = deposit(1024, 1024 * 1024);
	pub const CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = DummyRandomness<Self>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;

	type UploadOrigin = EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	type InstantiateOrigin = EitherOf<
		EnsureRootWithSuccess<AccountId, TreasuryAccount>,
		EitherOf<
			AsSignedByStaticCommunity<Runtime, ConstU16<1>>, // Virto
			AsSignedByStaticCommunity<Runtime, ConstU16<2>>, // Kippu
		>,
	>;

	/// The safest default is to allow no calls at all.
	///
	/// Runtimes should whitelist dispatchables that are allowed to be called
	/// from contracts and make sure they are stable. Dispatchables exposed to
	/// contracts are not allowed to change because that would break already
	/// deployed contracts. The `RuntimeCall` structure itself is not allowed
	/// to change the indices of existing pallets, too.
	type CallFilter = AllowBalancesCall;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type CallStack = [pallet_contracts::Frame<Self>; 23];
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type Schedule = Schedule;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;

	type ChainExtension = ();
	type ApiVersion = ();

	// ## From Pop Network
	// This node is geared towards development and testing of contracts.
	// We decided to increase the default allowed contract size for this
	// reason (the default is `128 * 1024`).
	//
	// Our reasoning is that the error code `CodeTooLarge` is thrown
	// if a too-large contract is uploaded. We noticed that it poses
	// less friction during development when the requirement here is
	// just more lax.
	type MaxCodeLen = ConstU32<{ 192 * 1024 }>;
	type MaxTransientStorageSize = ConstU32<{ 1024 * 1024 }>;
	type DefaultDepositLimit = DefaultDepositLimit;
	type MaxStorageKeyLen = ConstU32<128>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type UnsafeUnstableInterface = ConstBool<true>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type MaxDelegateDependencies = ConstU32<32>;
	type RuntimeHoldReason = RuntimeHoldReason;

	type Environment = ();
	type Debug = ();
	type Migrations = ();
	type Xcm = pallet_xcm::Pallet<Self>;
}
