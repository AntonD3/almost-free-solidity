// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract ComputationOracle {
    mapping(bytes32 merkleRoot -> bool proved) proofs;

    function verifyComputation(bytes32 codeHash, bytes32 input, bytes32 output) {

    }
}
