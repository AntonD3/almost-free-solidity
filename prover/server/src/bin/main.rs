use axum::{
    extract::{Path, Json, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use sha2::{Digest, Sha256};
use solc_zkmod_lib::{prover::prover};
use solc_zkmod_lib::prover::prover::ProvingInput;

// Shared state to keep track of submitted proofs
struct AppState {
    submitted_proofs: Mutex<HashSet<String>>,
    prover: prover::Prover,
}

// Request payload for /request-proof
#[derive(Deserialize)]
struct ProofRequest {
    requests: Vec<ProvingInput>,
}

// Response payload for /request-proof
#[derive(Serialize)]
struct ProofResponse {
    id: String,
}

// Handler for /request-proof
async fn request_proof(
    Json(payload): Json<ProofRequest>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut req_hash = Sha256::digest([]);
    let mut state_guard = state.submitted_proofs.lock().unwrap();
    for req in payload.requests {
        let concatenated = format!("{:?}{:?}", req.bytecode, req.calldata);
        let hash = format!("{:x}", Sha256::digest(concatenated));
        state_guard.insert(hash.clone());
        req_hash = Sha256::digest(hash.as_bytes().to_vec().append(&mut req_hash.to_vec()))
    }
    drop(state_guard);

    let req_id = format!("{:x}", req_hash);
    state.prover.prove(req_id.clone(), payload.requests);

    Json(ProofResponse { id: req_id })
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
    // Shared state to store proof IDs
    let state = Arc::new(AppState {
        submitted_proofs: Mutex::new(HashSet::new()),
        prover,
    });


    // Build the app with routes
    let app = Router::new()
        .route("/request-proof", post(request_proof))
        .route("/check-proof/:id", get(check_proof))
        .with_state(state.clone());

    let prover = prover::Prover::new();


    // Start the server
    println!("Running the server at 127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
