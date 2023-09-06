use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
};

use axum::{
    extract::{Extension, Json, State},
    response::IntoResponse,
    routing::post,
    Router,
    extract::FromRef
};
use solina::intent::Intent;

use crate::{
    types::{IntentJrpcRequest, IntentJrpcResponse},
    worker::SolinaWorker,
};

#[derive(Clone, FromRef)]
pub struct AppState {
    solina_worker: Arc<RwLock<SolinaWorker>>,
}

pub fn routes(solina_worker: SolinaWorker) -> Router {
    let app_state = AppState { solina_worker: Arc::new(RwLock::new(solina_worker)) };
    Router::new()
    .route("/", post(json_rpc_handler))
    .route("/intents", post(json_rpc_handler))
    .with_state(app_state)
}

pub async fn run_json_rpc(
    socket_address: SocketAddr,
    solina_worker: SolinaWorker,
) -> Result<(), anyhow::Error> {
    let server = axum::Server::try_bind(&socket_address).or_else(|_| {
        eprintln!("Failed to bind to socket address: {}", socket_address);
        axum::Server::try_bind(&"127.0.0.1:0".parse().unwrap())
    })?;
    let server = server.serve(routes(solina_worker).into_make_service());
    println!("Server is set up !");

    server.await?;

    Ok(())
}

async fn json_rpc_handler(
    State(solina_worker): State<Arc<RwLock<SolinaWorker>>>,
    Json(request): Json<IntentJrpcRequest>,
) -> Json<IntentJrpcResponse> {
    println!("New received request: {:?}", request);
    match request.method.as_str() {
        "store" => {
            let response = solina_worker
                .write()
                .expect("Failed to acquire lock")
                .process_intent_request(request);
            return Json(response);
        }
        _ => {
            return Json(IntentJrpcResponse {
                error: Some(String::from("Invalid request method")),
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
            });
        }
    }
}
