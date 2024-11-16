// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.2 <0.9.0;

contract Example {
    function func() public returns (uint256){
        return sum(100);
    }

    @free
    function sum(uint256 n) internal returns(uint256 result) {
        result = 0;
        for (uint256 i = 1; i <= n; i++) {
            result += i;
        }
    }
}
