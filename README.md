# [almost] free soldity
_Powered by ZKPs_

This project introduces a `@free` annotation that allows you to offload any heavy computation offchain. 
It allows solidity developers to bring the cost of compute-heavy functions almost to zero without sacrificing the DevEx. 

## Implementation
1. Preprocessing
   1. Use a custom Solidity preprocessor to find all the functions with `@free` annotation in the source code.
   2. Separate the implementations and compile them to EVM bytecode.
   3. Replace their implementations with a call to the verification oracle.
   4. Replace `calldatasize()` with `calldataload(caldatasize() - 32)` in the EVM bytecode. It allows us to pass additional witnesses needed for ZKP verification without affecting the function selectors and contract logic.
2. Proving
   1. Receive the bytecodes from step 1.2 along with the function inputs.
   2. Use a specialized server with a custom EVM implementation to ensure that the function execution can be performed without interactions with the state. 
   3. Use a zkVM to prove the EVM execution of the function's bytecode with specific calldata.
3. Aggregation
   1. Proofs from different users of the protocol are batched to share the verification cost.
   2. Final proof is sent to the blockchain, all the nested proofs are merklized, and the root is used as a part of the public input.
   3. Each of the nested proofs within the final one gets a Merkle proof to verify that it belongs to the verified batch.
4. Execution
   1. When calling the initial contract, the Merkle proof is provided in the "buffer" that was allocated in the calldata during preprocessing.
   2. Merkle proof is used to call the oracle and ensure that the function with the specific input will return specific output.

At the current level of ZK proving/verification costs, this can be mostly applied to a heavy computations like complex math, verification of bulks of signatures, cryptography.
But with the upcoming optimizations to the proving and verification, this approach can be applied to almost any function.

This also allows for a wide range of customizations, as the code within `@free` function is executed in zkVM - the alternative execution environments can be used, e.g. this allows us to write Rust code in Solidiy contracts.

**Note: the verification of the contracts using this annotation is impossible because the resulting bytecode is modified quite significantly.**

### Potential cost estimations(assuming that our proof is one out of 5 in a batch):

| Scenario                             | Normal cost (gas) | [almost] free soldity cost (gas) |
|--------------------------------------|-------------------|----------------------------------|
| Verification of 100 Merkle proofs    | ~250k             | ~60k                             |
| Verification of 2 Ed25519 signatures | ~1M               | ~60k                             |
| Calculating high Fibonacci numbers   | ∞                 | ~60k                             |

### Oracles' addresses on different networks:
