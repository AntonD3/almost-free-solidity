// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.2 <0.9.0;

contract Example {
    function example_func() external returns (uint256){
        return heavy_computation(100);
    }

    // This function will not be computed on chain! The output will be verified using ZK!
    @free
    function heavy_computation(uint256 n) internal returns(uint256 result) {
        result = 0;
        for (uint256 i = 1; i <= n; i++) {
            result += i;
        }
    }
}
