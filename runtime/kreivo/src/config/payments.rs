use super::*;

mod indices;

use frame_support::traits::EitherOf;
use frame_system::EnsureSigned;
use pallet_communities::origin::AsSignedByCommunity;
use parity_scale_codec::Encode;
use sp_runtime::traits::AccountIdConversion;

pub use indices::pallet_payment_indices;

parameter_types! {
	pub const MaxRemarkLength: u8 = 50;
	pub const IncentivePercentage: Percent = Percent::from_percent(INCENTIVE_PERCENTAGE);
	pub const PaymentPalletId: PalletId = PalletId(*b"payments");
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
		let pallet_id = communities::CommunityPalletId::get();
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

impl pallet_payment_indices::Config for Runtime {}
impl pallet_payments::PaymentId<Runtime> for virto_common::PaymentId {
	fn next(_: &AccountId, beneficiary: &AccountId) -> Option<Self> {
		let block: u32 = System::block_number();
		let idx = PaymentIndices::next_index();
		Some((block, idx, beneficiary.encode().as_slice()).into())
	}
}

impl pallet_payments::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = weights::pallet_payments::WeightInfo<Self>;
	type SenderOrigin = EitherOf<AsSignedByCommunity<Self>, EnsureSigned<AccountId>>;
	type BeneficiaryOrigin = EnsureSigned<AccountId>;
	type DisputeResolver = frame_system::EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	type PaymentId = virto_common::PaymentId;
	type Assets = Assets;
	type AssetsHold = AssetsHolder;
	type BlockNumberProvider = RelaychainData;
	type FeeHandler = KreivoFeeHandler;
	type Scheduler = Scheduler;
	type Preimages = Preimage;
	type OnPaymentStatusChanged = ();
	type PalletId = PaymentPalletId;
	type IncentivePercentage = IncentivePercentage;
	type MaxRemarkLength = MaxRemarkLength;
	type MaxFees = ConstU32<50>;
	type MaxDiscounts = ConstU32<10>;
	type CancelBufferBlockLength = ConstU32<{ 2 * DAYS }>;
}
