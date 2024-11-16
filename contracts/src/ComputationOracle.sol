// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";

contract ComputationOracle {
    using MerkleProof for bytes32[];

    mapping(bytes32 merkleRoot => bool proved) proved;

    function verifyComputation(bytes32 codeHash, bytes32 inputHash, bytes32 outputHash, bytes32[] calldata proof) external view {
        bytes memory computationProofPublicInput = bytes.concat(codeHash, inputHash, outputHash);
        bytes32 leaf = keccak256(computationProofPublicInput);
        require(proved[proof.processProof(leaf)]);
    }

    function proveComputation(bytes32 merkleRoot, bytes memory zkp) external {
        proved[merkleRoot] = true;
        // TODO: verify zkp proof
    }
}
