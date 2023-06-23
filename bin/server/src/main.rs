use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{StatusCode, Method};
use serde_json::json;
use std::convert::Infallible;
use futures::TryStreamExt as _;

async fn handle_hello_world(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    match (request.method(), request.uri().path()) {
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Try POSTing data to /echo");
        },
        (&Method::POST, "/echo") => {
            *response.body_mut() = request.into_body();
        },
        (&Method::POST, "/echo/uppercase") => {
            // future stream, make each byte uppercase
            let mapping = request
                .into_body()
                .map_ok(|chunk| {
                    chunk.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Vec<u8>>()
                });
            
            // set the response body
            *response.body_mut() = Body::wrap_stream(mapping);
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        },
    };

    Ok(response)
}

#[tokio::main]
async fn main() {
    // bind to 127.0.0.1:3000
    let addr = ([127, 0, 0, 1], 3000).into();

    // create boiler plate handler service for hello world request
    let make_service = make_service_fn(|_conn| async {
        // convert handler into a service
        Ok::<_, Infallible>(service_fn(handle_hello_world))
    });

    // create server
    let server = Server::bind(&addr).serve(make_service);

    println!("Server is running on http://127.0.0.1:3000");

    let graceful_halt = 
        server.with_graceful_shutdown(shutdown_signal());

    // run forever
    if let Err(e) = graceful_halt.await {
        eprintln!("server error: {}", e);
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}


