#[macro_use]
extern crate log;
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, web::post};
use actix_web::{HttpRequest, HttpResponse, Result};
use json::object;
use risc0_zkvm::serde::to_vec;
use risc0_zkvm::Prover;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::RwLock;

static JOB_COUNTER: AtomicUsize = AtomicUsize::new(1);
fn get_job_id() -> usize {
    JOB_COUNTER.fetch_add(1, Ordering::SeqCst)
}
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

struct Job {
    id: usize,
    lambda: Vec<u8>,
    input: String,
}

enum JobStatus {
    Queued,
    Processing,
    Error(String),
    Proved(ProvedResult),
}
#[derive(Clone)]
struct State {
    jobs: Arc<RwLock<HashMap<usize, JobStatus>>>,
    sender: Sender<Job>,
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/form.html")))
}

async fn get_verifier(req: HttpRequest) -> Result<NamedFile> {
    match req.match_info().get("verifier_file") {
        Some(verifier_file) => {
            debug!("verifier_file: {}", verifier_file);
            match verifier_file {
                "verifier.js" => Ok(NamedFile::open("./verifier/pkg/verifier.js")?),
                "verifier_bg.wasm" => Ok(NamedFile::open("./verifier/pkg/verifier_bg.wasm")?),
                _ => Ok(NamedFile::open("./verifier/pkg/_")?),
            }
        }
        None => Err(actix_web::error::ErrorNotFound("file not found")),
    }
}

async fn get_job(req: HttpRequest, state: web::Data<State>) -> Result<HttpResponse> {
    let response = match req.match_info().get("job_id") {
        Some(job_id) => {
            debug!("job_id: {}", job_id);
            // TODO: remove old jobs
            match state
                .jobs
                .read()
                .await
                .get(
                    &job_id
                        .parse::<usize>()
                        .map_err(|_| actix_web::error::ErrorBadRequest("failed to parse job id"))?,
                )
                .map(|job_status| match job_status {
                    JobStatus::Queued => object! {status: "queued", data: ""},
                    JobStatus::Processing => object! {status: "processing", data: ""},
                    JobStatus::Error(e) => object! {status: "error", "data": e.to_string()},
                    JobStatus::Proved(result) => {
                        object! {status: "done", data: serde_json::to_string(&result).unwrap()}
                    }
                }) {
                Some(response) => response,
                None => object! {status: "error", data: "job id not found".to_string()},
            }
        }
        None => object! {status: "error", data: "unknown job id"},
    };
    Ok(HttpResponse::Ok().body(response.to_string()))
}

fn prove(job: Job, method_ids: &mut HashMap<keccak_hash::H256, Vec<u8>>) -> Result<ProvedResult> {

    debug!("prove");
    let hash = keccak_hash::keccak(&job.lambda);
    let method_id = if let Some(id) = method_ids.get(&hash) {
        id
    } else {
        let method_id_container = risc0_zkvm::MethodId::compute(&job.lambda).unwrap();
        let method_id = method_id_container.as_slice();
        method_ids.insert(hash, method_id.to_vec());
        method_ids.get(&hash).unwrap()
    };

    let mut prover = Prover::new(&job.lambda, method_id.as_slice())
        .expect("Prover should be constructed from matching method code & ID");
    debug!("done making prover");

    let now = Instant::now();
    prover.add_input_u32_slice(&to_vec(&job.input).expect("should be serializable"));
    let receipt = prover.run().expect("should succeed");
    debug!("done proving, elapsed: {:.2?}", now.elapsed());

    let result = std::str::from_utf8(prover.get_output())?;
    debug!("result: {}", result,);

    Ok(ProvedResult {
        outputs: result.to_string(),
        method_id: method_id.to_vec(),
        receipt: serde_json::to_string(&receipt)?,
    })
}

async fn post_job(
    mut form: MultipartForm<LambdaUpload>,
    state: web::Data<State>,
) -> Result<HttpResponse> {
    debug!("post_job");
    let mut buffer: Vec<u8> = vec![0; form.code.size];
    form.code.file.read_exact(buffer.as_mut())?;
    let job_id = get_job_id(); // TODO: generate job id
    match state
        .sender
        .send(Job {
            id: job_id,
            lambda: buffer,
            input: form.input.0.clone(),
        })
        .await
    {
        Ok(_) => {
            state.jobs.write().await.insert(job_id, JobStatus::Queued);
            Ok(HttpResponse::Ok().body(job_id.to_string()))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("failed to post job")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    use actix_web::{web, App, HttpServer};
    let (sender, mut receiver) = channel::<Job>(1000);
    let state = State {
        jobs: Arc::new(RwLock::new(HashMap::new())),
        sender,
    };

    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut method_ids: HashMap<keccak_hash::H256, Vec<u8>> = HashMap::new();
        while let Some(job) = receiver.recv().await {
            debug!("got job");
            let job_id = job.id;
            state_clone
                .jobs
                .write()
                .await
                .insert(job_id, JobStatus::Processing);
            let status = match prove(job, &mut method_ids) {
                Ok(result) => {
                    debug!("got result");
                    JobStatus::Proved(result)
                }
                Err(e) => {
                    debug!("got error");
                    JobStatus::Error(format!("Failed to create proof: {}", e))
                }
            };
            state_clone.jobs.write().await.insert(job_id, status);
        }
    });

    let http_address = "localhost:8090";
    debug!("starting server at {}", http_address);
    let shared_state = web::Data::new(state);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default().allow_any_origin())
            .app_data(shared_state.clone())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/pkg/{verifier_file}").route(web::get().to(get_verifier)))
            .service(web::resource("/job").route(post().to(post_job)))
            .service(web::resource("/job/{job_id}").route(web::get().to(get_job)))
    })
    .bind(http_address)?
    .run()
    .await
}
