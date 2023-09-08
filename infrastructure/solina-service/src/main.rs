use solina_service::{config::SolinaConfig, error::Result};
use solina_service::{json_rpc_server::run_json_rpc, worker::SolinaWorker};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let solina_config = SolinaConfig::default();

    let solina_worker = SolinaWorker::new(solina_config).expect("Failed to start a Solina worker");
    run_json_rpc(addr, solina_worker).await?;

    Ok(())
}
