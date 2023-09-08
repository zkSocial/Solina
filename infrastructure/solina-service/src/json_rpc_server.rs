use log::{error, info};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use crate::error::{Error, Result};

use axum::{
    extract::FromRef,
    extract::{Json, State},
    routing::post,
    Router,
};

use crate::{
    types::{IntentRequest, IntentResponse},
    worker::SolinaWorker,
};

#[derive(Clone, FromRef)]
pub struct AppState {
    solina_worker: Arc<RwLock<SolinaWorker>>,
}

pub fn routes(solina_worker: SolinaWorker) -> Router {
    let app_state = AppState {
        solina_worker: Arc::new(RwLock::new(solina_worker)),
    };
    Router::new()
        .route("/", post(json_rpc_handler))
        .route("/intents", post(json_rpc_handler))
        .with_state(app_state)
}

pub async fn run_json_rpc(solina_worker: SolinaWorker) -> Result<()> {
    let mut bind = true;
    let socket_address = solina_worker.config().socket_address();
    let server = axum::Server::try_bind(&socket_address)
        .or_else(|_| {
            error!("Failed to bind to socket address: {}", socket_address);
            bind = false;
            axum::Server::try_bind(&"127.0.0.1:0".parse().unwrap())
        })
        .map_err(|_| Error::FailedToStartService)?;
    let server = server.serve(routes(solina_worker).into_make_service());

    let bind_addr = if bind {
        socket_address
    } else {
        "127.0.0.1:0".parse().unwrap()
    };
    info!("Started JSON RPC service at {:?}", bind_addr);

    server.await.map_err(|_| Error::FailedToStartService)?;

    Ok(())
}

async fn json_rpc_handler(
    State(solina_worker): State<Arc<RwLock<SolinaWorker>>>,
    Json(request): Json<IntentRequest>,
) -> Json<Result<IntentResponse>> {
    info!("New received request: {:?}", request);
    let response = solina_worker
        .write()
        .expect("Failed to acquire lock")
        .process_intent_request(request);
    Json(response)
}
