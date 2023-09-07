use solina_service::error::Result;
use solina_service::{json_rpc_server::run_json_rpc, worker::SolinaWorker};
use std::net::SocketAddr;
use storage_sqlite::SolinaStorage;

#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

    let storage = SolinaStorage::try_open(
        "/Users/jorgeantonio/dev/Solina/infrastructure/solina-service/solina-data.sqlite",
    )
    .expect("Failed to start Solina Storage");
    storage.run_migrations().expect("Failed to run migrations");

    let solina_worker = SolinaWorker::new(storage);
    run_json_rpc(addr, solina_worker).await?;

    Ok(())
}
