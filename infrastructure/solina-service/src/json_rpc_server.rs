use log::{error, info};
use std::sync::{Arc, RwLock};

use crate::error::{Error, Result};

use axum::{
    extract::FromRef,
    extract::{Json, State},
    routing::{get},
    Router,
};

use crate::{
    auth_middleware::EthereumAuthMiddlewareLayer,
    types::{
        GetAuthCredentialsRequest, GetAuthCredentialsResponse, GetBatchIntentsRequest,
        GetBatchIntentsResponse, GetIntentRequest, GetIntentResponse, RegisterSolverRequest,
        RegisterSolverResponse, StoreIntentRequest, StoreIntentResponse,
    },
    worker::SolinaWorker,
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub(crate) solina_worker: Arc<RwLock<SolinaWorker>>,
}

pub fn routes(solina_worker: SolinaWorker) -> Router {
    let app_state = AppState {
        solina_worker: Arc::new(RwLock::new(solina_worker)),
    };

    Router::new()
        .route(
            "/store_intent",
            get(get_auth_credentials_handler).post(store_intent_handler),
        )
        .route("/register_solver", get(get_auth_credentials_handler).post(register_solver_handler))
        .route("/deregister_solver", get(get_auth_credentials_handler).dele)
        .layer(EthereumAuthMiddlewareLayer {
            app_state: app_state.clone(),
        })
        .route("/get_intent", get(get_intent_handler))
        .route("/get_batch_intents", get(get_batch_intents_handler))
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

async fn store_intent_handler(
    State(solina_worker): State<Arc<RwLock<SolinaWorker>>>,
    Json(request): Json<StoreIntentRequest>,
) -> Json<Result<StoreIntentResponse>> {
    info!("New POST request to submit intent: {:?}", request);
    let response = solina_worker
        .write()
        .expect("Failed to acquire lock")
        .handle_post_store_intent_request(request);
    Json(response)
}

async fn get_intent_handler(
    State(solina_worker): State<Arc<RwLock<SolinaWorker>>>,
    Json(request): Json<GetIntentRequest>,
) -> Json<Result<GetIntentResponse>> {
    info!("New GET request for intent with id: {}", request.id);
    let response = solina_worker
        .write()
        .expect("Failed to acquire lock")
        .handle_get_intent_request(request);
    Json(response)
}

async fn get_batch_intents_handler(
    State(solina_worker): State<Arc<RwLock<SolinaWorker>>>,
    Json(request): Json<GetBatchIntentsRequest>,
) -> Json<Result<GetBatchIntentsResponse>> {
    info!(
        "New GET request for batch intents with ids: {:?}",
        request.ids
    );
    let response = solina_worker
        .write()
        .expect("Failed to acquire lock")
        .handle_get_batch_intents_request(request);
    Json(response)
}

async fn get_auth_credentials_handler(
    State(solina_worker): State<Arc<RwLock<SolinaWorker>>>,
    Json(request): Json<GetAuthCredentialsRequest>,
) -> Json<Result<GetAuthCredentialsResponse>> {
    info!(
        "New GET request for authentication credentials, for address: {}",
        request.address
    );
    let write_lock = solina_worker.write();
    if let Err(e) = write_lock {
        error!("Failed to acquire worker lock, with error: {}", e);
        return Json(Err(Error::InternalError));
    } else {
        let response = write_lock
            .unwrap()
            .handle_get_auth_credentials_request(request);
        Json(response)
    }
}

async fn register_solver_handler(
    State(solina_worker): State<Arc<RwLock<SolinaWorker>>>,
    Json(request): Json<RegisterSolverRequest>,
) -> Json<Result<RegisterSolverResponse>> {
    info!(
        "New POST request for solver registration, {}",
        request.solver_address
    );
    let write_lock = solina_worker.write();
    if let Err(e) = write_lock {
        error!("Failed to acquire worker lock, with error: {}", e);
        return Json(Err(Error::InternalError));
    } else {
        let response = write_lock.unwrap().handle_solver_registration(request);
        Json(response)
    }
}
