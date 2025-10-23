// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev The on-chain address of the Foo (Cross-Consensus Messaging) precompile.
address constant FOO_PRECOMPILE_ADDRESS = address(0xF0000);

/// @title Foo Precompile Interface
/// @notice A foo interface
interface IFoo {
    function transfer(address to, uint256 value) external returns (bool);
    function fortytwo() external view returns (uint128);
}
