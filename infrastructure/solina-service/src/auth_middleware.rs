use crate::{
    auth_challenge::{generate_challenge, verify_signature},
};
use std::sync::{Arc, Mutex};
use axum::body::{boxed, Body, BoxBody};
use axum::{
    http::{Request, StatusCode},
    response::Response,
    Json,
};
use hyper::body::{to_bytes};
use futures_util::future::BoxFuture;
// use http_body::combinators::box_body::UnsyncBoxBody;
use log::error;
use tower::{layer::Layer, Service};

#[derive(Clone)]
pub struct EthereumAuthMiddleware<S> {
    inner: Arc<Mutex<S>>,
}

impl<S> Service<Request<Body>> for EthereumAuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<BoxBody>> + Send + 'static,
    S::Future: Send + 'static
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
        Box::pin(async move {
            match req.method() {
                // Handle GET request to retrieve a challenge for user
                &http::Method::GET => {
                    let challenge = generate_challenge();
                    let response = Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain")
                        .body(boxed(Body::from(challenge)))
                        .expect("Failed to generate for challenge response");
                    Ok(response)
                }
                &http::Method::POST => {
                    let (parts, body) = req.into_parts();
                    let body_bytes = to_bytes(body)
                        .await
                        .expect("Failed to extract body bytes");
                    let value = serde_json::from_slice(body_bytes.as_ref())
                        .expect("Failed to extract JSON from request bytes");
                    let result = verify_signature(&Json(value));
                    if let Err(e) = result {
                        error!("Failed to verify challenge signature, with error: {}", e);
                        let response = Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(boxed(Body::from("Invalid challenge signature")))
                            .expect("Failed to form body");
                        return Ok(response);
                    }
                    // Reconstruct the request
                    let req = Request::from_parts(parts, Body::from(body_bytes));
                    // if signature verification is successful, forward the req call to the inner service
                    let future = {
                        let mut inner_service = inner.lock().unwrap();
                        inner_service.call(req)
                    };
                    future.await
                }
                // Forward other requests to the inner service
                _ => {
                    let future = {
                        let mut inner_service = inner.lock().unwrap();
                        inner_service.call(req)
                    };
                    future.await
                },
            }
        })
    }
}

#[derive(Clone)]
pub struct EthereumAuthMiddlewareLayer {}

impl<S> Layer<S> for EthereumAuthMiddlewareLayer {
    type Service = EthereumAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EthereumAuthMiddleware { inner: Arc::new(Mutex::new(inner)) }
    }
}
