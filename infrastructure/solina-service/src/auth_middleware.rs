use crate::auth_challenge::{generate_challenge, verify_signature};
use axum::{
    http::{Request, Response, StatusCode},
    Json,
};
use hyper::Body;
use serde_json::{json, Value};
use std::{future::Future, pin::Pin};
use tower::{layer::Layer, Service};
use log::error;

pub struct EthereumAuthMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Json<serde_json::Value>>> for EthereumAuthMiddleware<S>
where
    S: Service<Request<Json<Value>>, Response = Response<Json<Value>>> + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // Forward the call to the inner service
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Json<serde_json::Value>>) -> Self::Future {
        match req.method() {
            // Handle GET request to retrieve a challenge for user
            &http::Method::GET => {
                let challenge = generate_challenge();
                let response = Response::new(Json(json!({ "challenge": challenge })));
                Box::pin(async move { Ok(response) })
            }
            &http::Method::POST => {
                let result = verify_signature(req.body());
                if let Err(e) = result {
                    error!("Failed to verify challenge signature, with error: {}", e);
                    let response = Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Json(json!({"message": "Invalid signature for challenge"})))
                        .expect("Failed to form body");
                    Box::pin(async move { Ok(response) })
                } else {
                    // if signature verification is successful, forward the req call to the inner service
                    Box::pin(self.inner.call(req))
                }
            }
            // Forward other requests to the inner service
            _ => Box::pin(self.inner.call(req)),
        }
    }
}

pub struct EthereumAuthMiddlewareLayer {}

impl<S> Layer<S> for EthereumAuthMiddlewareLayer {
    type Service = EthereumAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EthereumAuthMiddleware { inner }
    }
}
