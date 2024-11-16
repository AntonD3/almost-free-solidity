// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";
import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

contract ComputationOracle {
    using MerkleProof for bytes32[];

    mapping(bytes32 merkleRoot => bool proved) public proved;

    address governance;
    address public sp1_verifier;
    bytes32 public program_key;

    constructor() {
        governance = msg.sender;
    }

    function setVerifier(address verifier) external {
        require(governance == msg.sender);
        sp1_verifier = verifier;
    }

    function setProgramKey(bytes32 key) external {
        require(governance == msg.sender);
        program_key = key;
    }

    function verifyComputation(bytes32 codeHash, bytes32 inputHash, bytes32 outputHash, bytes32[] calldata proof) external view {
        bytes memory computationProofPublicInput = bytes.concat(codeHash, inputHash, outputHash);
        bytes32 leaf = keccak256(computationProofPublicInput);
        require(proved[proof.processProof(leaf)]);
    }

    function proveComputation(bytes32 merkleRoot, bytes memory zkp, bool dummyVerifier) external {
        proved[merkleRoot] = true;
        // used for testing
        if (!dummyVerifier) {
            ISP1Verifier(sp1_verifier).verifyProof(program_key, abi.encodePacked(merkleRoot), zkp);
        }
    }
}
