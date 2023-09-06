use crate::{
    error::SolinaError,
    mempool::SolinaMempool,
    types::{IntentJrpcRequest, IntentJrpcResponse, IntentRequest, IntentResponse},
};
use bincode::deserialize;
use core::pin::Pin;
use futures::{stream::FuturesUnordered, Future};
use hex::encode;
use solina::{intent::Intent, structured_hash::StructuredHashInterface, Uuid};
use storage_sqlite::SolinaStorage;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub struct SolinaWorker {
    mempool: SolinaMempool,
    storage_connection: SolinaStorage,
}

// TODO: add logic for shutdown signal
impl SolinaWorker {
    pub fn new(storage_connection: SolinaStorage) -> Self {
        Self {
            mempool: SolinaMempool::new(),
            storage_connection,
        }
    }

    pub fn process_intent_request(
        &mut self,
        intent_request: IntentJrpcRequest,
    ) -> IntentJrpcResponse {
        let intent: Intent = match intent_request.params {
            Some(params) => {
                match serde_json::from_value(params) {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!(
                            "Failed to deserialize intent data, with error: {}",
                            e.to_string()
                        ); // TODO: add proper logging
                        return IntentJrpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: intent_request.id,
                            error: Some(e.to_string()),
                            result: None,
                        };
                    }
                }
            }
            None => {
                return IntentJrpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: intent_request.id,
                    error: Some(String::from("Invalid intent request params")),
                    result: None,
                }
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
                            tx.commit()
                                .expect("Failed to commit changes to the database");
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
                return IntentJrpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: intent_request.id,
                    error: Some(e.to_string()),
                    result: None,
                };
            }
        }
        IntentJrpcResponse {
            jsonrpc: "2.0".to_string(),
            id: intent_request.id,
            error: None,
            result: Some(serde_json::json!({
                "intent_hash": encode(intent_structured_hash)
            })),
        }
    }
}
