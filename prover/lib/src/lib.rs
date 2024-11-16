pub mod evm;
pub mod prover;

use alloy_sol_types::sol;
use crate::evm::context::Context;
use crate::evm::evm;

sol! {
    struct PublicValuesStruct {
        bytes32 _merkleRoot;
    }
}

pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};

    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}

pub fn run_evm(bytecode: Vec<u8>, calldata: Vec<u8>) -> Result<Vec<u8>, String> {
    let result = evm(
        bytecode.as_slice(),
        Context::new(calldata.as_slice()),
    );

    if !result.success {
        return Err(format!("Unsuccessful execution, {:?}", result.error));
    }

    Ok(result.return_val.ok_or("missing return value")?)
}
