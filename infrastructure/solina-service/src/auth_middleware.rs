use crate::{
    auth_challenge::{extract_address, extract_signature, generate_challenge, verify_signature},
    json_rpc_server::AppState,
};
use axum::body::{boxed, Body, BoxBody};
use axum::{
    http::{Request, StatusCode},
    response::Response,
    Json,
};
use futures_util::future::BoxFuture;
use hyper::body::to_bytes;
use std::sync::{Arc, Mutex};
// use http_body::combinators::box_body::UnsyncBoxBody;
use log::{error, info};
use tower::{layer::Layer, Service};

#[derive(Clone)]
pub struct EthereumAuthMiddleware<S> {
    inner: Arc<Mutex<S>>,
    pub(crate) app_state: AppState,
}

impl<S> Service<Request<Body>> for EthereumAuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<BoxBody>> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // Forward the call to the inner service
        self.inner.lock().unwrap().poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let inner = self.inner.clone();
        let app_state = self.app_state.clone();
        Box::pin(async move {
            match req.method() {
                &http::Method::GET => {
                    let future = {
                        let mut inner_service = inner.lock().unwrap();
                        info!("Sending request to inner service");
                        inner_service.call(req)
                    };
                    future.await
                }
                &http::Method::POST => {
                    let (parts, body) = req.into_parts();
                    let body_bytes = to_bytes(body).await.expect("Failed to extract body bytes");
                    let value = serde_json::from_slice(body_bytes.as_ref())
                        .expect("Failed to extract JSON from request bytes");
                    info!("The provided JSON value is: {}", value);
                    let address = extract_address(&value).expect("Failed to extract signature");
                    let signature = extract_signature(&value).expect("Failed to extract signature");
                    let credential = {
                        let mut app_state_write = app_state
                            .solina_worker
                            .write()
                            .expect("Failed to write to worker");
                        let mut tx = app_state_write
                            .storage_connection()
                            .create_transaction()
                            .expect("Failed to store intent batch to database, with error: {}");

                        tx.get_current_auth_credential(&address)
                            .expect("Failed to insert new credential to DB")
                    };
                    info!("The current credential is: {:?}", credential);
                    let result = {
                        let now = chrono::prelude::Utc::now().naive_utc();
                        let config_auth_timeout = app_state
                            .solina_worker
                            .read()
                            .unwrap()
                            .config()
                            .auth_credential_timeout();
                        if credential.is_auth
                            && now
                                .signed_duration_since(credential.created_at)
                                .num_seconds()
                                <= config_auth_timeout as i64
                        {
                            let future = {
                                let mut inner_service = inner.lock().unwrap();
                                info!("Sending request to inner service");
                                // Reconstruct the request
                                let req = Request::from_parts(parts, Body::from(body_bytes));
                                inner_service.call(req)
                            };
                            return future.await;
                        } else if now
                            .signed_duration_since(credential.created_at)
                            .num_seconds()
                            <= config_auth_timeout as i64
                        {
                            verify_signature(address, credential.challenge.clone(), signature)
                        } else {
                            {
                                let mut app_state_write = app_state
                                    .solina_worker
                                    .write()
                                    .expect("Failed to write to worker");
                                let mut tx = app_state_write
                                    .storage_connection()
                                    .create_transaction()
                                    .expect(
                                        "Failed to store intent batch to database, with error: {}",
                                    );
                                tx.update_is_valid_credential(credential.id)
                                    .expect("Failed to update credential");
                            }
                            let response = Response::builder()
                                .status(StatusCode::UNAUTHORIZED)
                                .body(boxed(Body::from(
                                    "User does not have a valid challenge in memory to sign",
                                )))
                                .expect("Failed to form body");
                            return Ok(response);
                        }
                    };
                    if let Err(e) = result {
                        error!("Failed to verify challenge signature, with error: {}", e);
                        let response = Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(boxed(Body::from("Invalid challenge signature")))
                            .expect("Failed to form body");
                        return Ok(response);
                    } else {
                        // otherwise update the authentication in the database
                        let mut app_state_write = app_state
                            .solina_worker
                            .write()
                            .expect("Failed to write to worker");
                        let mut tx = app_state_write
                            .storage_connection()
                            .create_transaction()
                            .expect("Failed to store intent batch to database, with error: {}");

                        tx.update_is_auth_credential(credential.id)
                            .expect("Failed to insert new credential to DB");
                    }
                    // Reconstruct the request
                    let req = Request::from_parts(parts, Body::from(body_bytes));
                    // if signature verification is successful, forward the req call to the inner service
                    let future = {
                        let mut inner_service = inner.lock().unwrap();
                        info!("Sending request to inner service");
                        inner_service.call(req)
                    };
                    future.await
                }
                _ => {
                    let future = {
                        let mut inner_service = inner.lock().unwrap();
                        info!("Sending request to inner service");
                        inner_service.call(req)
                    };
                    future.await
                }
            }
        })
    }
}

#[derive(Clone)]
pub struct EthereumAuthMiddlewareLayer {
    pub(crate) app_state: AppState,
}

impl<S> Layer<S> for EthereumAuthMiddlewareLayer {
    type Service = EthereumAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EthereumAuthMiddleware {
            inner: Arc::new(Mutex::new(inner)),
            app_state: self.app_state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::{prelude::*, utils::keccak256};
    use hex::encode;

    #[tokio::test]
    async fn challenge_auth() {
        let challenge = "nCMWmFSDdHP86tfO2BeqSWWyJ0d5vTGx";
        let wallet = ethers::prelude::Wallet::from_bytes(&[
            11, 176, 40, 212, 120, 121, 98, 44, 125, 5, 140, 11, 173, 250, 133, 5, 23, 126, 153,
            162, 85, 39, 195, 104, 241, 251, 117, 187, 10, 148, 7, 187,
        ])
        .expect("Failed to convert bytes");

        // Print the private key and address
        println!("Address: {:?}", wallet);

        // Sign the message hash
        let signature = wallet.sign_message(challenge).await.unwrap();

        let signature_hex = encode(signature.to_vec());

        // Print the signature
        println!("Signature hex: {:?}", signature_hex);

        assert!(signature.verify(challenge, wallet.address()).is_ok());
    }
}