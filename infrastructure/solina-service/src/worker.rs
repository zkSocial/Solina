use crate::error::{Error, Result};
use crate::{
    mempool::SolinaMempool,
    types::{IntentRequest, IntentResponse},
};
use hex::encode;
use log::{error, info};
use solina::{intent::Intent, structured_hash::StructuredHashInterface};
use storage_sqlite::SolinaStorage;

pub struct SolinaWorker {
    mempool: SolinaMempool,
    storage_connection: SolinaStorage,
    current_intent_id: i64,
}

// TODO: add logic for shutdown signal
impl SolinaWorker {
    pub fn new(storage_connection: SolinaStorage) -> Self {
        Self {
            mempool: SolinaMempool::new(),
            storage_connection,
            current_intent_id: 0,
        }
    }

    pub fn process_intent_request(
        &mut self,
        intent_request: IntentRequest,
    ) -> Result<IntentResponse> {
        let intent: Intent = serde_json::from_value(intent_request.intent_json).map_err(|e| {
            error!(
                "Failed to deserialize intent request to an Intent, with error: {:?}",
                e
            );
            Error::InvalidRequest
        })?;
        let intent_structured_hash = intent.structured_hash();
        info!(
            "Requested intent has structured hash: {}",
            encode(intent_structured_hash)
        );
        let intent_batch = self.mempool.insert(intent);
        if let Some(batch) = intent_batch {
            let mut tx = self.storage_connection.create_transaction().map_err(|e| {
                error!(
                    "Failed to create transaction on the database, with error: {:?}",
                    e
                );
                Error::InternalError
            })?;
            tx.store_intents(&batch).map_err(|e| {
                error!("Failed to store batch of intents, with error: {:?}", e);
                Error::InternalError
            })?;
        }
        Ok(IntentResponse {
            intent_id: Some(self.update_current_id()),
            is_success: true,
            message: String::from("Intent has been successfully submitted."),
        })
    }

    fn update_current_id(&mut self) -> i64 {
        self.current_intent_id += 1;
        self.current_intent_id
    }
}
