#[macro_use]
extern crate log;
use actix_cors::Cors;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::web::post;
use actix_web::{HttpResponse, Result};
use risc0_zkvm::serde::to_vec;
use risc0_zkvm::Prover;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::time::Instant;

#[derive(MultipartForm)]
struct LambdaUpload {
    code: TempFile,
    input: Text<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ProvedResult {
    outputs: String,
    method_id: Vec<u8>,
    receipt: String,
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/form.html")))
}

async fn prove(mut form: MultipartForm<LambdaUpload>) -> Result<String> {
    debug!("making prover with {}", form.code.size);
    let mut buffer: Vec<u8> = vec![0; form.code.size];
    form.code.file.read_exact(buffer.as_mut())?;

    debug!("making prover with {}", form.input.0);

    let now = Instant::now();
    let method_id_container = risc0_zkvm::MethodId::compute(&buffer).unwrap();
    let method_id = method_id_container.as_slice();
    debug!("method id computed {:.2?}", now.elapsed());

    let mut prover = Prover::new(&buffer, method_id)
        .expect("Prover should be constructed from matching method code & ID");
    debug!("done making prover");

    let now = Instant::now();
    prover.add_input_u32_slice(&to_vec(&form.input.0).expect("should be serializable"));
    let receipt = prover.run().expect("should succeed");
    debug!("done proving, elapsed: {:.2?}", now.elapsed());

    receipt
        .verify(method_id)
        .expect("Proven code should verify");
    debug!("verified");

    let result = std::str::from_utf8(prover.get_output())?;

    debug!("result: {}", result,);

    Ok(serde_json::to_string(&ProvedResult {
        outputs: result.to_string(),
        method_id: method_id.to_vec(),
        receipt: serde_json::to_string(&receipt)?,
    })?)
}

async fn prove_handler(form: MultipartForm<LambdaUpload>) -> Result<HttpResponse> {
    match prove(form).await {
        Ok(r) => Ok(HttpResponse::Ok().body(r)),
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    use actix_web::{web, App, HttpServer};

    let http_address = "localhost:8090";
    debug!("starting server at {}", http_address);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default().allow_any_origin())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/prove").route(post().to(prove_handler)))
    })
    .bind(http_address)?
    .run()
    .await
}
