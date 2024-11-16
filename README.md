# free-computation

## solc-zkmod
solc extension to replace "@free" functions calculation with ZKP verification.

### implementation
There are 4 steps:
- We are looking for functions marked as "@free" in the source code and replacing their implementation with ZKP verification
- Compiling functions implementation to EVM bytecode
- Compiling preprocessed contract source code to EVM bytecode
- Replacing `calldatasize()` with `calldataload(caldatasize() - 32)` in the EVM bytecode.
   It's needed because we are using calldata memory to pass additional witnesses needed for ZKP verification.
   But we don't want to affect contracts behavior, so we are pushing "logical calldata size" at the end of the celldata.
