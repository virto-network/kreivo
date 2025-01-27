use super::*;

use fc_traits_memberships::OnMembershipAssigned;
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

const WELL_KNOWN_ATTR_KEYS: [&[u8]; 3] = [b"membership_member_rank", b"membership_gas", b"membership_expiration"];

parameter_types! {
	pub CopySystemAttributesOnAssign: Box<dyn OnMembershipAssigned<AccountId, CommunityId, MembershipId>> =
		Box::new(|_, group, m| {
			use frame_support::traits::nonfungibles_v2::{Inspect as NonFunsInspect, Mutate};
			for key in WELL_KNOWN_ATTR_KEYS.into_iter() {
				if let Some(value) = CommunityMemberships::system_attribute(&group, Some(&m), key) {
					<CommunityMemberships as Mutate<_, _>>::set_attribute(&group, &m, key, &value)?;
				}
			}

			Ok(())
		});
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
	fn collection(id: u16) -> CommunityId {
		id.into()
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
