//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system plonk
//! ```

use std::collections::HashMap;
use alloy_sol_types::SolType;
use clap::{Parser, ValueEnum};

use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin, SP1VerifyingKey};
use std::path::PathBuf;
use std::sync::Arc;
use clap::builder::Str;
use crate::PublicValuesStruct;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const SOLC_ZKMOD_ELF: &[u8] = include_elf!("solc-zkmod-program");

/// The arguments for the prover input.
#[derive(Parser, Debug, Serialize, Deserialize)]
pub struct ProvingInput {
    pub bytecode: Vec<u8>,
    pub calldata: Vec<u8>,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1ProofFixture {
    merkle_root: [u8; 32],
    vkey: String,
    public_values: String,
    proof: String,
}

#[derive(Clone)]
pub struct Prover {
    pk: SP1ProvingKey,
    vk: SP1VerifyingKey,
    client: Arc<ProverClient>,
    proofs: HashMap<String, Vec<u8>>,
}

impl Prover {
    fn new() -> Self {
        sp1_sdk::utils::setup_logger();

        let client = ProverClient::new();

        let (pk, vk) = client.setup(SOLC_ZKMOD_ELF);

        Self {
            pk,
            vk,
            client: Arc::new(client),
            proofs: HashMap::new(),
        }
    }

    pub fn prove(&mut self, req_id: String, requests: Vec<ProvingInput>) {
        let client = self.client.clone(); // Assume `client` is `Clone`
        let pk = self.pk.clone();
        let vk = self.vk.clone();
        let mut proofs = self.proofs.clone(); // Clo

        std::thread::spawn(move || {
            let mut stdin = SP1Stdin::new();
            stdin.write(&requests);
            let proof = client.prove(&pk, stdin).compressed().run().unwrap();

            // Update the cloned proofs map
            proofs.insert(req_id.clone(), proof.proof.clone());

            // Generate proof fixture
            create_proof_fixture(&proof, &vk, ProofSystem::Groth16);

            // If needed, pass the updated proofs back to the original context
        });
    }

    pub fn get_proof(&self, id: String) -> Option<&SP1Proof> {
        self.proofs.get(id.as_str())
    }
}

/// Create a fixture for the given proof.
fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
) {
    // Deserialize the public values.
    let bytes = proof.public_values.as_slice();
    let PublicValuesStruct { _merkleRoot } = PublicValuesStruct::abi_decode(bytes, false).unwrap();

    // Create the testing fixture so we can test things end-to-end.
    let fixture = SP1ProofFixture {
        merkle_root: _merkleRoot.0,
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
        .expect("failed to write fixture");
}
