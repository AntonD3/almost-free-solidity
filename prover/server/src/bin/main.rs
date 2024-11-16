use axum::{
    extract::{Path, Json, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, env, sync::{Arc, Mutex}};
use std::collections::HashMap;
use std::hash::Hash;
use sha2::{Digest, Sha256};
use web3::types::{CallRequest, H160};
use solc_zkmod_lib::{keccak256, prover::prover, run_evm};
use solc_zkmod_lib::prover::prover::ProvingInput;

// Shared state to keep track of submitted proofs
struct AppState {
    submitted_proofs: Mutex<HashSet<String>>,
    known_bytecodes: HashMap<[u8; 32], Vec<u8>>,
    known_req_data: HashMap<String, ReqData>,
    prover: prover::Prover,
}

struct ReqData {
    bytecode: Vec<u8>,
    calldata: Vec<u8>,
    merkle_proof: Vec<u8>,
}

#[derive(Deserialize)]
struct AddBytecodeRequest {
    bytecode: Vec<u8>,
}

struct ProofRequestData {
    pub address: [u8; 20],
    pub calldata: Vec<u8>,
    pub value: u64,
    pub sender: [u8; 20],
}

#[derive(Deserialize)]
struct ProofRequest {
    requests: Vec<ProofRequestData>,
}


#[derive(Serialize)]
struct ProofResponse {
    id: String,
}

#[derive(Serialize)]
struct WitnessOutput {
    output: Vec<u8>,
}

// Handler for /request-proof
async fn request_proof(
    Json(payload): Json<ProofRequest>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut req_hash = Sha256::digest([]);
    let mut proving_inputs = vec![];

    for req in payload.requests {
        let (code_hash, calldata) = make_eth_call(req);
        let bytecode = state.known_bytecodes.get(code_hash).unwrap();

        let concatenated = format!("{:?}{:?}", bytecode.clone(), calldata);
        let hash = format!("{:x}", Sha256::digest(concatenated));
        req_hash = Sha256::digest(hash.as_bytes().to_vec().append(&mut req_hash.to_vec()));
        proving_inputs.push(ProvingInput {
            bytecode: bytecode.clone(),
            calldata,
        })
    }

    let req_id = format!("{:x}", req_hash);
    let mut state_guard = state.submitted_proofs.lock().unwrap();
    state_guard.insert(req_id.clone());
    drop(state_guard);

    state.prover.prove(req_id.clone(), proving_inputs);

    Json(ProofResponse { id: req_id })
}

async fn make_eth_call(data: ProofRequestData) -> Result<([u8], [u8]), String> {
    let rpc_url = env::var("RPC_URL")?;
    let transport = web3::transports::Http::new(rpc_url.as_str());
    let web3 = web3::Web3::new(transport);

    // Smart contract address (replace with your contract's address)
    let contract_address: H160 = data.address
        .parse()
        .expect("Invalid contract address");

    // Sender address (can be zero address for read-only calls)
    let sender_address: H160 = data.sender
        .parse()
        .expect("Invalid sender address");

    // Create the CallRequest
    let call_request = CallRequest {
        from: Some(sender_address),
        to: Some(contract_address),
        gas: None,
        gas_price: None,
        value: Some(data.value),
        data: Some(data.calldata),
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    // Make the eth_call
    let result: Vec<u8> = web3.eth().call(call_request, None).await?;
    if hex::decode("deadbeef")? != result[..32].to_vec() {
        return Err("missing prefix");
    }

    let code_hash = result[32..64];
    let input_data = result[64..];

    Ok((code_hash, input_data))
}

async fn get_witness(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let proof = state.prover.get_proof(id.clone()).unwrap();
    let witness_data = state.known_req_data.get(id.clone().as_str()).unwrap().clone();
    let result = run_evm(witness_data.bytecode.clone(), witness_data.calldata.clone()).unwrap();

    let output = [
        *[result.len() as u8],
        result[..],
        *[witness_data.merkle_proof.len() as u8],
        witness_data.merkle_proof[..],
    ].concat();

    Json(WitnessOutput { output })
}

async fn add_bytecode(
    Json(payload): Json<AddBytecodeRequest>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.known_bytecodes.insert(keccak256(payload.bytecode.clone().as_slice()), payload.bytecode);
}

// Handler for /check-proof/{id}
async fn check_proof(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let proof = state.prover.get_proof(id);

    Json(serde_json::json!({ "exists": proof.is_some() }))
}

#[tokio::main]
async fn main() {
    let prover = prover::Prover::new();

    // Shared state to store proof IDs
    let state = Arc::new(AppState {
        submitted_proofs: Mutex::new(HashSet::new()),
        known_bytecodes: HashMap::new(),
        known_req_data: HashMap::new(),
        prover,
    });


    // Build the app with routes
    let app = Router::new()
        .route("/request-proof", post(request_proof))
        .route("/check-proof/:id", get(check_proof))
        .route("/add-bytecode", post(add_bytecode))
        .route("/get-witness/:id", get(get_witness))
        .with_state(state.clone());


    // Start the server
    println!("Running the server at 127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
