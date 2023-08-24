use crate::{
    error::SolinaError,
    mempool::SolinaMempool,
    types::{IntentRequest, IntentResponse},
};
use bincode::deserialize;
use core::pin::Pin;
use futures::{stream::FuturesUnordered, Future};
use solina::intent::Intent;
use solina::{structured_hash::StructuredHashInterface, Uuid};
use storage_sqlite::SolinaStorage;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub struct SolinaWorker {
    rx_intent_request: Receiver<IntentRequest>,
    tx_intent_response: Sender<IntentResponse>,
    mempool: SolinaMempool,
    storage_connection: SolinaStorage,
}

// TODO: add logic for shutdown signal
impl SolinaWorker {
    pub fn new(
        rx_intent_request: Receiver<IntentRequest>,
        tx_intent_response: Sender<IntentResponse>,
        storage_connection: SolinaStorage,
    ) -> Self {
        Self {
            rx_intent_request,
            tx_intent_response,
            mempool: SolinaMempool::new(),
            storage_connection,
        }
    }

    pub async fn spawn(
        &self,
        rx_intent_request: Receiver<IntentRequest>,
        tx_intent_response: Sender<IntentResponse>,
        storage_connection: SolinaStorage,
    ) -> JoinHandle<Result<(), SolinaError>> {
        tokio::spawn(async move {
            Self::new(rx_intent_request, tx_intent_response, storage_connection)
                .run()
                .await
        })
    }

    pub async fn run(&mut self) -> Result<(), SolinaError> {
        while let Some(intent_request) = self.rx_intent_request.recv().await {
            let bytes_data = intent_request.intent_bytes.clone();
            let intent = match deserialize::<Intent>(&bytes_data) {
                Ok(inner) => inner,
                Err(e) => {
                    eprintln!(
                        "Failed to deserialize intent data, with error: {}",
                        e.to_string()
                    ); // TODO: add proper logging
                    self.tx_intent_response
                        .send(IntentResponse {
                            intent_id: None,
                            is_success: false,
                            message: e.to_string(),
                        })
                        .await
                        .map_err(|e| SolinaError::SolinaWorkerError(e.to_string()))?;
                    continue;
                }
            };
            let intent_structured_hash = intent.structured_hash();
            let intent_batch = self.mempool.insert(intent.clone());
            if let Some(batch) = intent_batch {
                let result: Result<(), String> = {
                    match self.storage_connection.create_read_transaction() {
                        Ok(mut tx) => {
                            if let Err(e) = tx.store_intents(&batch) {
                                eprintln!(
                                    "Failed to store intents to the database, with error: {}",
                                    e.to_string()
                                );
                                Err(e.to_string())
                            } else {
                                Ok(())
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get read_write transaction from database");
                            Err(e.to_string())
                        }
                    };
                    Ok(())
                };
                if let Err(e) = result {
                    self.tx_intent_response
                        .send(IntentResponse {
                            intent_id: Some(Uuid {
                                id: intent_structured_hash,
                            }),
                            is_success: true,
                            message: e.to_string(),
                        })
                        .await
                        .map_err(|e| SolinaError::SolinaWorkerError(e.to_string()))?;
                }
            }
            let response = IntentResponse {
                intent_id: Some(Uuid {
                    id: intent_structured_hash,
                }),
                is_success: true,
                message: "Success".to_string(),
            };
            self.tx_intent_response
                .send(response)
                .await
                .map_err(|e| SolinaError::SolinaWorkerError(e.to_string()))?;
        }
        Ok(())
    }
}
