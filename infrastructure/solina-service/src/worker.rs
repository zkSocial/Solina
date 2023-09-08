use crate::{
    config::SolinaConfig,
    error::{Error, Result},
};
use crate::{
    mempool::SolinaMempool,
    types::{GetIntentRequest, GetIntentResponse, StoreIntentRequest, StoreIntentResponse},
};
use hex::encode;
use log::{error, info};
use solina::{intent::Intent, structured_hash::StructuredHashInterface};
use storage_sqlite::SolinaStorage;

pub struct SolinaWorker {
    mempool: SolinaMempool,
    storage_connection: SolinaStorage,
    current_intent_id: i64,
    config: SolinaConfig,
}

impl SolinaWorker {
    pub fn new(config: SolinaConfig) -> Result<Self> {
        let storage_connection =
            SolinaStorage::try_open(config.storage_file_path()).map_err(|e| {
                error!("Failed to start a storage connection, with error: {}", e);
                Error::InternalError
            })?;
        storage_connection.run_migrations().map_err(|e| {
            error!("Failed to run migrations, with error: {}", e);
            Error::InternalError
        })?;
        Ok(Self {
            mempool: SolinaMempool::new(config.mempool_capacity()),
            storage_connection,
            current_intent_id: 0,
            config,
        })
    }

    fn update_current_id(&mut self) -> i64 {
        self.current_intent_id += 1;
        self.current_intent_id
    }
}

impl SolinaWorker {
    pub fn config(&self) -> &SolinaConfig {
        &self.config
    }
}

impl SolinaWorker {
    pub fn rollback(&mut self) -> Option<Intent> {
        self.current_intent_id = self.current_intent_id.checked_sub(1).unwrap_or(0);
        self.mempool.rollback().map(|(_, i)| i)
    }

    pub fn process_store_intent_request(
        &mut self,
        store_intent_request: StoreIntentRequest,
    ) -> Result<StoreIntentResponse> {
        let intent: Intent =
            serde_json::from_value(store_intent_request.intent_json).map_err(|e| {
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
        let intent_id = self.update_current_id();
        let batch = self.mempool.insert(intent_id, intent);

        if batch.is_none() {
            return Ok(StoreIntentResponse {
                intent_id: Some(intent_id),
                is_success: true,
                message: String::from("Intent has been successfully submitted"),
            });
        }

        let batch = batch.unwrap();
        let mut tx = self.storage_connection.create_transaction().map_err(|e| {
            error!(
                "Failed to store intent batch to database, with error: {}",
                e
            );
            // update current inner state of self.
            self.mempool.rollback();
            self.current_intent_id -= 1;
            Error::InternalError
        })?;

        tx.store_intents(&batch).map_err(|e| {
            error!(
                "Failed to store intent batch to database, with error: {}",
                e
            );
            // update current inner state of self.
            self.mempool.rollback();
            self.current_intent_id -= 1;
            Error::InternalError
        })?;

        Ok(StoreIntentResponse {
            intent_id: Some(intent_id),
            is_success: true,
            message: String::from("Intent has been successfully submitted"),
        })
    }

    pub fn process_get_intent_request(
        &self,
        get_intent_request: GetIntentRequest,
    ) -> Result<GetIntentResponse> {
        let intent_id = get_intent_request.id;
        // we first verify if the intent is still in the mempool
        if let Some(intent) = self
            .mempool
            .mempool_data
            .iter()
            .find(|(id, _)| *id == intent_id as i64)
            .map(|(_, int)| int)
        {
            let intent_json = serde_json::to_value(&intent).map_err(|e| {
                error!("Failed to serialize intent data to JSON, with error: {}", e);
                Error::InternalError
            })?;
            return Ok(GetIntentResponse {
                intent_json,
                message: String::from("GET intent successfully"),
                is_success: true,
            });
        }
        // Otherwise, we need to query the database
        let mut tx = self.storage_connection.create_transaction().map_err(|e| {
            error!(
                "Failed to create transaction on the database, with error: {:?}",
                e
            );
            Error::InternalError
        })?;
        let intent = tx
            .get_intent(intent_id)
            .map_err(|e| {
                error!("Failed to store batch of intents, with error: {:?}", e);
                Error::InternalError
            })?
            .to_intent()
            .map_err(|e| {
                error!("Failed to convert intent, with error: {}", e);
                Error::InternalError
            })?;

        let intent_json = serde_json::to_value(&intent).map_err(|e| {
            error!("Failed to serialize intent data to JSON, with error: {}", e);
            Error::InternalError
        })?;

        Ok(GetIntentResponse {
            intent_json,
            message: String::from("GET intent successfully"),
            is_success: true,
        })
    }
}
