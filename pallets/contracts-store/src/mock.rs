//! Test environment for contracts store pallet.

use crate as pallet_contracts_store;

use frame_contrib_traits::listings::test_utils::{self, MockListings};
use frame_support::traits::Time;
use frame_support::{derive_impl, pallet_prelude::ConstU32, traits::EnsureOrigin};
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::EnsureSigned;
use pallet_contracts::{AddressGenerator, Frame, Schedule};
use sp_core::parameter_types;
use sp_runtime::{traits::IdentityLookup, BuildStorage, SaturatedConversion};

use mock_helpers::ExtHelper;
pub use sp_io::TestExternalities;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u128;
pub type AssetId = u32;
pub type Balance = <Test as pallet_balances::Config>::Balance;

pub type Listings = MockListings<Test>;

// Configure a mock runtime to test the pallet.
#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeTask,
		RuntimeHoldReason,
		RuntimeFreezeReason
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;
	#[runtime::pallet_index(10)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(30)]
	pub type Contracts = pallet_contracts;
	#[runtime::pallet_index(31)]
	pub type ContractStore = pallet_contracts_store;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = IdentityLookup<Self::AccountId>;
	type AccountData = pallet_balances::AccountData<Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
}

parameter_types! {
	pub MySchedule: Schedule<Test> = <Schedule<Test>>::default();
}

type MerchantId = u32;
pub type AppId = u64;
pub type LicenseId = u16;

impl test_utils::Config for Test {
	type MerchantId = MerchantId;
	type InventoryId = AppId;
	type ItemId = LicenseId;
	type AssetId = AssetId;
	type Balance = Balance;
	type MaxMetadataLen = ConstU32<64>;
	type MaxKeyLen = ConstU32<32>;
	type MaxValueLen = ConstU32<64>;
}

pub struct SimpleAddressGenerator;

impl AddressGenerator<Test> for SimpleAddressGenerator {
	fn contract_address(
		deploying_address: &AccountId,
		code_hash: &<Test as frame_system::Config>::Hash,
		input_data: &[u8],
		salt: &[u8],
	) -> AccountId {
		let deploying_address = deploying_address << 64;
		let code_hash = (code_hash.0.into_iter().filter(|b| *b == 0).count() as AccountId) << 32;
		let input_data = (input_data.len() as AccountId) << 16;
		let salt = salt.len() as AccountId;

		deploying_address + code_hash + input_data + salt
	}
}

impl Time for Test {
	type Moment = BlockNumberFor<Self>;

	fn now() -> Self::Moment {
		System::block_number()
	}
}

#[derive_impl(pallet_contracts::config_preludes::TestDefaultConfig)]
impl pallet_contracts::Config for Test {
	type Time = Self;
	type Currency = Balances;
	type UploadOrigin = EnsureSigned<AccountId>;
	type InstantiateOrigin = EnsureSigned<AccountId>;
	type AddressGenerator = SimpleAddressGenerator;
	type Schedule = MySchedule;
	type CallStack = [Frame<Self>; 5];
}

pub struct EnsureSignedMerchant;

impl EnsureOrigin<RuntimeOrigin> for EnsureSignedMerchant {
	type Success = (AccountId, u32);

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		match o.clone().caller {
			OriginCaller::system(frame_system::RawOrigin::Signed(who)) => Ok((who, who.saturated_into::<u32>())),
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::signed(ALICE))
	}
}

impl pallet_contracts_store::Config for Test {
	type WeightInfo = ();
	type InstantiateOrigin = EnsureSignedMerchant;
	type AppId = AppId;
	type LicenseId = LicenseId;
	type Listings = Listings;
	type ContractsStoreMerchantId = ConstU32<0>;
}

#[derive(Default)]
pub struct ExtBuilder {
	accounts: mock_helpers::BalancesExtBuilder<Test>,
}

impl ExtBuilder {
	fn with_account(mut self, who: AccountId, amount: Balance) -> Self {
		self.accounts = self.accounts.with_account(who, amount);
		self
	}

	fn build(self) -> TestExternalities {
		let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		self.accounts.as_storage().assimilate_storage(&mut storage).unwrap();

		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn new_test_ext() -> TestExternalities {
	ExtBuilder::default().with_account(ALICE, Balance::MAX / 2).build()
}
