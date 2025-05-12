use super::*;

use frame_contrib_traits::memberships::OnMembershipAssigned;
use frame_system::EnsureRootWithSuccess;
use sp_runtime::traits::Verify;

use pallet_nfts::PalletFeatures;
use sp_core::ConstU128;
use virto_common::MembershipId;

parameter_types! {
	pub MembershipsPalletFeatures: PalletFeatures = PalletFeatures::all_enabled();
	pub const MaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
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

impl pallet_nfts::Config<CommunityMembershipsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = CommunityId;
	type ItemId = MembershipId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	// Ensure only root is allowed to issue new membership groups.
	type CreateOrigin = EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	type Locker = ();
	type CollectionDeposit = ();
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ItemDeposit = ();
	#[cfg(feature = "runtime-benchmarks")]
	// When benchmarking, items must have a deposit cost, in order for `redeposit`
	// to work.
	type ItemDeposit = ConstU128<CENTS>;
	type MetadataDepositBase = ();
	type AttributeDepositBase = ();
	type DepositPerByte = ();
	type StringLimit = ConstU32<256>;
	type KeyLimit = ConstU32<64>;
	type ValueLimit = ConstU32<256>;
	type ApprovalsLimit = ConstU32<20>;
	type ItemAttributesApprovalsLimit = ConstU32<30>;
	type MaxTips = ConstU32<10>;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = ConstU32<10>;
	type Features = ();
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;
	type BlockNumberProvider = System;
}
