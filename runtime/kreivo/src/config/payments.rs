use super::*;

use frame_support::traits::EitherOf;
use frame_system::EnsureSigned;
use pallet_communities::origin::AsSignedByCommunity;
use parity_scale_codec::Encode;
use sp_runtime::traits::AccountIdConversion;

parameter_types! {
	pub const MaxRemarkLength: u8 = 50;
	pub const IncentivePercentage: Percent = Percent::from_percent(INCENTIVE_PERCENTAGE);
	pub const PaymentPalletId: PalletId = PalletId(*b"payments");
}

#[cfg(feature = "runtime-benchmarks")]
pub struct PaymentsBenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl pallet_payments::BenchmarkHelper<AccountId, FungibleAssetLocation, Balance> for PaymentsBenchmarkHelper {
	fn create_asset(id: FungibleAssetLocation, admin: AccountId, is_sufficient: bool, min_balance: Balance) {
		<Assets as frame_support::traits::tokens::fungibles::Create<AccountId>>::create(
			id,
			admin,
			is_sufficient,
			min_balance,
		)
		.unwrap();
	}
}

pub struct KreivoFeeHandler;

const MANDATORY_FEE: bool = true;
pub const SENDER_FEE: Percent = Percent::from_percent(1);
pub const BENEFICIARY_FEE: Percent = Percent::from_percent(3);
pub const INCENTIVE_PERCENTAGE: u8 = 10;

impl FeeHandler<Runtime> for KreivoFeeHandler {
	fn apply_fees(
		asset: &AssetIdOf<Runtime>,
		sender: &AccountId,
		beneficiary: &AccountId,
		amount: &Balance,
		_remark: Option<&[u8]>,
	) -> Fees<Runtime> {
		let min = <Assets as fungibles::Inspect<AccountId>>::minimum_balance(*asset);
		let pallet_id = crate::config::communities::CommunityPalletId::get();
		let default_fee = |fee: Percent| (TreasuryAccount::get(), min.max(fee.mul_floor(*amount)), MANDATORY_FEE);
		let is_community =
			|who| matches!(PalletId::try_from_sub_account::<CommunityId>(who), Some((pid, _)) if pallet_id == pid );

		let mut sender_fees = vec![];
		let mut beneficiary_fees = vec![];

		if !is_community(sender) {
			sender_fees.push(default_fee(SENDER_FEE))
		}
		if !is_community(beneficiary) {
			beneficiary_fees.push(default_fee(BENEFICIARY_FEE))
		}
		Fees {
			sender_pays: BoundedVec::try_from(sender_fees).unwrap(),
			beneficiary_pays: BoundedVec::try_from(beneficiary_fees).unwrap(),
		}
	}
}

impl pallet_payments::PaymentId<Runtime> for virto_common::PaymentId {
	fn next(_: &AccountId, beneficiary: &AccountId) -> Option<Self> {
		let block: u32 = System::block_number();
		let idx = System::extrinsic_index()?;
		Some((block, idx, beneficiary.encode().as_slice()).into())
	}
}

impl pallet_payments::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Assets = Assets;
	type AssetsBalance = Balance;
	type PaymentId = virto_common::PaymentId;
	type FeeHandler = KreivoFeeHandler;
	type IncentivePercentage = IncentivePercentage;
	type MaxRemarkLength = MaxRemarkLength;
	type SenderOrigin = EitherOf<AsSignedByCommunity<Self>, EnsureSigned<AccountId>>;
	type BeneficiaryOrigin = EnsureSigned<AccountId>;
	type DisputeResolver = frame_system::EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	type PalletId = PaymentPalletId;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxDiscounts = ConstU32<10>;
	type MaxFees = ConstU32<50>;
	type RuntimeCall = RuntimeCall;
	type Scheduler = Scheduler;
	type Preimages = Preimage;
	type CancelBufferBlockLength = ConstU32<14400>; // 2 days
	type PalletsOrigin = OriginCaller;
	type WeightInfo = crate::weights::pallet_payments::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = PaymentsBenchmarkHelper;
}
