// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Foo Precompile Interface
/// @notice A foo interface
interface IFoo {
    // event Transfer(address indexed from, address indexed to, uint128 value);
    ///  function transfer(address to, uint256 value) external returns (bool);
    function fortytwo() external view returns (uint128);
}
