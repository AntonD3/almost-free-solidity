contract Provable {
    fallback(bytes calldata input) external payable returns(bytes memory output) {
        (uint256 a, uint256 b) = abi.decode(input, (uint256, uint256));
        return abi.encode(add(a, b));
    }

    function add(uint256 a, uint256 b) internal returns(uint256) {
        return a + b;
    }
}