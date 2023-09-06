use solina_service::{json_rpc_server::run_json_rpc, worker::SolinaWorker};
use std::net::SocketAddr;
use std::path::Path;
use storage_sqlite::SolinaStorage;
use tokio;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let solina_worker = SolinaWorker::new(SolinaStorage::try_open("/Users/jorgeantonio/dev/Solina/infrastructure/solina-service/solina-data").unwrap());
    run_json_rpc(addr, solina_worker).await;
}
