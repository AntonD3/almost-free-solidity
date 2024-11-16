// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

struct PublicValuesStruct {
    bytes32 _merkleRoot;
}

/// @title SolcZkmod.
/// @author Succinct Labs
contract SolcZkmod {
    /// @notice The address of the SP1 verifier contract.
    /// @dev This can either be a specific SP1Verifier for a specific version, or the
    ///      SP1VerifierGateway which can be used to verify proofs for any version of SP1.
    ///      For the list of supported verifiers on each chain, see:
    ///      https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public verifier;

    bytes32 public programVKey;

    constructor(address _verifier, bytes32 _programVKey) {
        verifier = _verifier;
        programVKey = _programVKey;
    }

    /// @param _proofBytes The encoded proof.
    /// @param _publicValues The encoded public values.
    function verifyProof(bytes calldata _publicValues, bytes calldata _proofBytes)
        public
        view
        returns (bytes32)
    {
        ISP1Verifier(verifier).verifyProof(programVKey, _publicValues, _proofBytes);
        PublicValuesStruct memory publicValues = abi.decode(_publicValues, (PublicValuesStruct));
        return (publicValues._merkleRoot);
    }
}
