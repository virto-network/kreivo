// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

/// Fixed-type, opinionated ABI for the Payments pallet precompile.
/// Reverts on error. Returns values on success.

interface IPayments {
    enum Role { Sender, Beneficiary }

    // ---- Events (mapped to pallet events; names may differ slightly) ----
    event PaymentCreated(bytes32 indexed paymentId, uint256 asset, uint256 amount, bytes remark);
    event PaymentReleased(bytes32 indexed paymentId);
    event PaymentCancelled(bytes32 indexed paymentId);
    event PaymentRefundRequested(bytes32 indexed paymentId, uint256 expiryBlock);
    event PaymentRefunded(bytes32 indexed paymentId);
    event PaymentRefundDisputed(bytes32 indexed paymentId);
    event PaymentRequestCreated(bytes32 indexed paymentId);
    event PaymentRequestCompleted(bytes32 indexed paymentId);
    event PaymentDisputeResolved(bytes32 indexed paymentId);

    // ---- Write methods (REVERT on error) ----
    //
    // Origin mapping:
    //   - Functions intended for the payment "sender" use msg.sender as origin.
    //   - Functions intended for the "beneficiary" / "resolver" use msg.sender accordingly.
    // Role gating is enforced inside the precompile/pallet.

    /// Mirror of: pay(origin, beneficiary, asset, amount, remark)
    /// Returns the PaymentId assigned by the pallet.
    function pay(
        address beneficiary,
        uint256 asset,
        uint256 amount,
        bytes calldata remark
    ) external returns (bytes32 paymentId);

    /// Mirror of: release(origin, payment_id)
    function release(bytes32 paymentId) external returns (bool);

    /// Mirror of: request_refund(origin, payment_id)
    /// Returns the block number at which auto-cancel would execute if undisputed.
    function requestRefund(bytes32 paymentId) external returns (uint256 expiryBlock);

    /// Mirror of: accept_and_pay(origin, payment_id)
    function acceptAndPay(bytes32 paymentId) external returns (bool);

    /// Mirror of: cancel(origin, payment_id) — callable by beneficiary
    function cancel(bytes32 paymentId) external returns (bool);

    /// Mirror of: dispute_refund(origin, payment_id) — callable by beneficiary
    function disputeRefund(bytes32 paymentId) external returns (bool);

    /// Mirror of: request_payment(origin (beneficiary), sender, asset, amount)
    /// Returns the PaymentId created by the pallet.
    function requestPayment(
        address sender,
        uint256 asset,
        uint256 amount
    ) external returns (bytes32 paymentId);

    /// Mirror of: resolve_dispute(origin (resolver), payment_id, dispute_result)
    /// percentBeneficiary is 0..100 mapped to Percent.
    function resolveDispute(
        bytes32 paymentId,
        Role inFavorOf,
        uint8 percentBeneficiary
    ) external returns (bool);

    // ---- Optional views (only if the precompile exposes storage reads) ----
    // These are fixed-type projections of your PaymentDetail and PaymentParties.

    /// Returns a snapshot of the payment detail for (sender, paymentId).
    /// NOTE: `state` is an implementation-defined small integer mapping of your PaymentState.
    function paymentOf(address sender, bytes32 paymentId)
        external
        view
        returns (
            uint256 asset,
            uint256 amount,
            address beneficiary,
            uint256 incentiveAmount,
            uint8 state,          // map your PaymentState to small ints
            bytes memory fees     // opaque blob or ABI-stable struct if you define it
        );

    /// Returns the (sender, beneficiary) tuple for a paymentId.
    function paymentParties(bytes32 paymentId)
        external
        view
        returns (address sender, address beneficiary);
}
