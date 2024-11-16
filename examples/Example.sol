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

contract heavy_computation_wrapper {
    fallback(bytes memory input) returns(bytes memory output) {
        (bytes memory mem, uint256 stack) = abi.decode(input, (bytes memory, uint256));
        // logic
        return abi.encode(mem_res, stack_res);
    }
}

pi = keccak(keccak(deployed_bytecode), keccak(input), keccak(output));

