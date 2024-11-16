// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.2 <0.9.0;

contract Example {
    function retrieve() public view returns (uint256){
        return heavy_computation(...);
    }

    @free
    function heavy_computation(bytes memory mem, uint256 stack) internal pure returns(bytes memory mem_res, uint256 stack_res) {
        // some logic
        while(false) {
            stack += stack;
        }
        return (mem, stack);
    }
}

function heavy_computation(bytes memory mem, uint256 stack) internal pure returns(bytes memory mem_res, uint256 stack_res) {
    // load needed witnesses from the scratch space after calldata
    bytes calldata output;
    bytes32[] calldata proof;
    assembly {
        output := add(calldatasize(), 32)
        let outSize := calldataload(output)
        proof := add(output, outSize)
    }

    // verify execution
    bytes32 inputHash = keccak256(abi.encode(/*input names*/));
    bytes32 outputHash = keccak256(output);
    bytes memory calldata_buffer = new bytes(100 + proof.length);
    assembly {
        mstore(add(calldata_buffer, 32), /*selector*/)
        mstore(add(calldata_buffer, 36), /*codeHash*/)
        mstore(add(calldata_buffer, 68), inputHash)
        mstore(add(calldata_buffer, 100), outputHash)
        calldatacopy(add(calldata_buffer, 132), add(proof, 32), calldataload(proof))
        let success := call(gas(), /*oracle address*/, 0, add(calldata_buffer, 32), mload(calldata_buffer), 0, 0)
        if iszero(success) {
            revert(0, 0)
        }
    }
    return abi.decode(output, (/*output types*/))
}
}

}


contract heavy_computation_wrapper {
    fallback(bytes memory input) returns(bytes memory output) {
        (bytes memory mem, uint256 stack) = abi.decode(input, (bytes memory, uint256));
        output = heavy_computation(input);
        return abi.encode(mem_res, stack_res);
    }

    function heavy_computation(bytes memory mem, uint256 stack) internal pure returns(bytes memory mem_res, uint256 stack_res) {
        // some logic
        while(false) {
            stack += stack;
        }
        return (mem, stack);
    }
}

pi = keccak(keccak(deployed_bytecode), keccak(input), keccak(output));

