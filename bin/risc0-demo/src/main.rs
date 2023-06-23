#[macro_use]
extern crate log;
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, web::post, HttpRequest, HttpResponse, Result};
use json::object;
use risc0_zkvm::{serde::to_vec, Prover};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Read,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};
use tokio::sync::{
    mpsc::{channel, Sender},
    RwLock,
};

// TODO: generate a unique job id
type JobId = usize;

static JOB_COUNTER: AtomicUsize = AtomicUsize::new(1);
fn get_job_id() -> JobId {
    JOB_COUNTER.fetch_add(1, Ordering::SeqCst)
}
#[derive(MultipartForm)]
struct LambdaUpload {
    code: TempFile,
    input: Text<String>,
}

struct Job {
    id: JobId,
    code: Vec<u8>,
    input: String,
}

enum JobStatus {
    Queued,
    Processing,
    Error(String),
    Proved(ProvedResult),
}

#[derive(Serialize, Deserialize)]
pub struct ProvedResult {
    outputs: String,
    method_id: Vec<u8>,
    receipt: String,
}

#[derive(Clone)]
struct Context {
    jobs: Arc<RwLock<HashMap<JobId, JobStatus>>>,
    sender: Sender<Job>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    use actix_web::{web, App, HttpServer};
    let (sender, mut receiver) = channel::<Job>(1000);
    let context = Context {
        jobs: Arc::new(RwLock::new(HashMap::new())),
        sender,
    };

    let prover_context = context.clone();
    tokio::spawn(async move {
        let mut method_id_cache: HashMap<keccak_hash::H256, Vec<u8>> = HashMap::new();
        while let Some(job) = receiver.recv().await {
            debug!("got job");
            let job_id = job.id;
            prover_context
                .jobs
                .write()
                .await
                .insert(job_id, JobStatus::Processing);
            let status = match prove(job, &mut method_id_cache) {
                Ok(result) => JobStatus::Proved(result),
                Err(e) => {
                    debug!("Failed to create proof: {}", e);
                    JobStatus::Error(format!("Failed to create proof: {}", e))
                }
            };
            prover_context.jobs.write().await.insert(job_id, status);
        }
    });

    let http_address = "localhost:8090";
    let web_context = web::Data::new(context);

    info!("starting server at {}", http_address);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default().allow_any_origin())
            .app_data(web_context.clone())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/pkg/{verifier_file}").route(web::get().to(get_verifier)))
            .service(web::resource("/job").route(post().to(post_job)))
            .service(web::resource("/job/{job_id}").route(web::get().to(get_job)))
    })
    .bind(http_address)?
    .run()
    .await
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/form.html")))
}

async fn post_job(
    mut form: MultipartForm<LambdaUpload>,
    context: web::Data<Context>,
) -> Result<HttpResponse> {
    debug!("post_job");
    let mut buffer: Vec<u8> = vec![0; form.code.size];
    form.code.file.read_exact(buffer.as_mut())?;
    let job_id = get_job_id();
    match context
        .sender
        .send(Job {
            id: job_id,
            code: buffer,
            input: form.input.0.clone(),
        })
        .await
    {
        Ok(_) => {
            context.jobs.write().await.insert(job_id, JobStatus::Queued);
            Ok(HttpResponse::Ok().body(job_id.to_string()))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("failed to post job")),
    }
}

async fn get_job(req: HttpRequest, context: web::Data<Context>) -> Result<HttpResponse> {
    let response = match req.match_info().get("job_id") {
        Some(job_id) => {
            debug!("job_id: {}", job_id);
            // TODO: remove old jobs
            match context
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
                    JobStatus::Proved(result) => match serde_json::to_string(&result) {
                        Ok(result) => object! {status: "done", data: result},
                        Err(e) => object! {status: "error", data: e.to_string()},
                    },
                }) {
                Some(response) => response,
                None => object! {status: "error", data: "job id not found".to_string()},
            }
        }
        None => object! {status: "error", data: "unknown job id"},
    };
    Ok(HttpResponse::Ok().body(response.to_string()))
}

async fn get_verifier(req: HttpRequest) -> Result<NamedFile> {
    match req.match_info().get("verifier_file") {
        Some(verifier_file) => {
            debug!("verifier_file: {}", verifier_file);
            match verifier_file {
                "verifier.js" => Ok(NamedFile::open("./verifier/pkg/verifier.js")?),
                "verifier_bg.wasm" => Ok(NamedFile::open("./verifier/pkg/verifier_bg.wasm")?),
                _ => Err(actix_web::error::ErrorForbidden("access denied")),
            }
        }
        None => Err(actix_web::error::ErrorNotFound("file not found")),
    }
}

fn prove(
    job: Job,
    method_id_cache: &mut HashMap<keccak_hash::H256, Vec<u8>>,
) -> anyhow::Result<ProvedResult> {
    debug!("prove");
    let hash = keccak_hash::keccak(&job.code);
    let method_id = if let Some(id) = method_id_cache.get(&hash) {
        id
    } else {
        let method_id_container = risc0_zkvm::MethodId::compute(&job.code)
            .map_err(|e| anyhow::anyhow!("Failed to compute method id {}", e))?;
        let method_id = method_id_container.as_slice();
        method_id_cache.insert(hash, method_id.to_vec());
        method_id_cache
            .get(&hash)
            .ok_or(anyhow::anyhow!("Failed to get method id"))?
    };

    let mut prover = Prover::new(&job.code, method_id.as_slice())
        .map_err(|e| anyhow::anyhow!("Failed to construct prover {}", e))?;

    let now = Instant::now();
    prover.add_input_u32_slice(
        &to_vec(&job.input).map_err(|e| anyhow::anyhow!("Failed to serialize inputs: {}", e))?,
    );
    let receipt = prover
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to run prover {}", e))?;
    debug!("done proving, elapsed: {:.2?}", now.elapsed());

    let result = std::str::from_utf8(prover.get_output())?;
    debug!("result: {}", result,);

    Ok(ProvedResult {
        outputs: result.to_string(),
        method_id: method_id.to_vec(),
        receipt: serde_json::to_string(&receipt)?,
    })
}
