use super::*;

use frame_system::EnsureRootWithSuccess;
use sp_runtime::traits::Verify;

use pallet_nfts::PalletFeatures;
use virto_common::MembershipId;

parameter_types! {
	pub MembershipsPalletFeatures: PalletFeatures = PalletFeatures::all_enabled();
	pub const MaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
	pub const MetadataDepositBase: Balance = 0;
	pub const AttributeDepositBase: Balance = 0;
	pub const DepositPerByte: Balance = 0;
}

pub type CommunityMembershipsInstance = pallet_nfts::Instance2;

// From https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/asset-hubs/asset-hub-kusama/src/lib.rs#L810
impl pallet_nfts::Config<CommunityMembershipsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type CollectionId = CommunityId;
	type ItemId = MembershipId;

	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	// Ensure only root is allowed to executing `create` calls
	type CreateOrigin = EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	type Locker = ();

	type CollectionDeposit = ();
	type ItemDeposit = ();
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;

	type StringLimit = ConstU32<256>;
	type KeyLimit = ConstU32<64>;
	type ValueLimit = ConstU32<256>;
	type ApprovalsLimit = ConstU32<20>;
	type ItemAttributesApprovalsLimit = ConstU32<30>;
	type MaxTips = ConstU32<10>;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = ConstU32<10>;
	type Features = MembershipsPalletFeatures;

	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = NftsBenchmarksHelper;

	type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct NftsBenchmarksHelper;

#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::traits::IdentifyAccount;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_nfts::BenchmarkHelper<CommunityId, MembershipId, <Signature as Verify>::Signer, AccountId, Signature>
	for NftsBenchmarksHelper
{
	fn collection(_: u16) -> CommunityId {
		<Runtime as pallet_communities::Config>::BenchmarkHelper::community_id()
	}
	fn item(i: u16) -> MembershipId {
		i.into()
	}
	fn signer() -> (sp_runtime::MultiSigner, AccountId) {
		let public = sp_io::crypto::sr25519_generate(0.into(), None);
		let account = sp_runtime::MultiSigner::Sr25519(public).into_account();
		(public.into(), account)
	}
	fn sign(signer: &sp_runtime::MultiSigner, message: &[u8]) -> Signature {
		sp_runtime::MultiSignature::Sr25519(
			sp_io::crypto::sr25519_sign(0.into(), &signer.clone().try_into().unwrap(), message).unwrap(),
		)
	}
}
