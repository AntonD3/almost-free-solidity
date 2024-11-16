#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use solc_zkmod_lib::{run_evm, PublicValuesStruct, keccak256, prover::prover};

fn hash_pair(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    keccak256(
        [
            a[..],
            b[..]
        ].concat()
    )
}

pub fn main() {
    let requests = sp1_zkvm::io::read::<Vec<prover::ProvingInput>>();

    let mut leafs = vec![];
    for request in requests {
        let result = run_evm(request.bytecode.clone(), request.calldata.clone()).unwrap();
        let leaf = keccak256(
            [
                keccak256(request.bytecode.as_slice())[..],
                keccak256(request.calldata.as_slice())[..],
                keccak256(hex::encode(result).as_bytes())[..],
            ].concat()
        );
        leafs.push(leaf);
    }

    leafs.sort_by(|x, y| y.cmp(&x));

    let mut current_level = leafs;

    while current_level.len() > 1 {
        let mut next_level = Vec::new();

        // Process pairs of nodes
        for i in (0..current_level.len()).step_by(2) {
            if i + 1 < current_level.len() {
                // Hash the pair
                next_level.push(hash_pair(current_level[i], current_level[i + 1]));
            } else {
                // Odd number of nodes, duplicate the last node
                next_level.push(hash_pair(current_level[i], current_level[i]));
            }
        }

        current_level = next_level;
    }

    let root = current_level[0].clone();

    // Encode the public values of the program.
    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct {
        _merkleRoot: root.into(),
    });

    sp1_zkvm::io::commit_slice(&bytes);
}
