use crate::{
    auth_challenge::generate_challenge,
    mempool::SolinaMempool,
    types::{
        GetAuthCredentialsRequest, GetAuthCredentialsResponse, GetBatchIntentsRequest,
        GetBatchIntentsResponse, GetIntentRequest, GetIntentResponse, RegisterSolverRequest,
        RegisterSolverResponse, StoreIntentRequest, StoreIntentResponse,
    },
};
use crate::{
    config::SolinaConfig,
    error::{Error, Result},
};
use ethers::prelude::*;
use hex::encode;
use log::{error, info};
use solina::{intent::Intent, structured_hash::StructuredHashInterface};
use std::str::FromStr;
use storage_sqlite::{AuthCredentials, SolinaStorage};

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

    pub fn storage_connection(&mut self) -> &mut SolinaStorage {
        &mut self.storage_connection
    }
}

impl SolinaWorker {
    pub fn rollback(&mut self) -> Option<Intent> {
        self.current_intent_id = self.current_intent_id.checked_sub(1).unwrap_or(0);
        self.mempool.rollback().map(|(_, i)| i)
    }

    pub fn handle_post_store_intent_request(
        &mut self,
        store_intent_request: StoreIntentRequest,
    ) -> Result<StoreIntentResponse> {
        let result = self.process_store_intent_request(store_intent_request);
        match result {
            Ok(response) => Ok(response),
            Err(e) => {
                // if we get an error, we need to rollback the internal state
                self.rollback();
                Err(e)
            }
        }
    }

    fn process_store_intent_request(
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
        info!("Current intent id is: {}", intent_id);
        let batch = self.mempool.insert(intent_id, intent);

        info!("Current batch is: {:?}", batch);

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
            Error::InternalError
        })?;

        tx.store_intents(&batch).map_err(|e| {
            error!(
                "Failed to store intent batch to database, with error: {}",
                e
            );
            Error::InternalError
        })?;

        Ok(StoreIntentResponse {
            intent_id: Some(intent_id),
            is_success: true,
            message: String::from("Intent has been successfully submitted"),
        })
    }

    pub fn handle_get_intent_request(
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
            let intent_json = serde_json::to_value(intent).map_err(|e| {
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

        let intent_json = serde_json::to_value(intent).map_err(|e| {
            error!("Failed to serialize intent data to JSON, with error: {}", e);
            Error::InternalError
        })?;

        Ok(GetIntentResponse {
            intent_json,
            message: String::from("GET intent successfully"),
            is_success: true,
        })
    }

    pub fn handle_get_batch_intents_request(
        &self,
        get_intent_request: GetBatchIntentsRequest,
    ) -> Result<GetBatchIntentsResponse> {
        let mut intent_ids = get_intent_request.ids;
        // we first verify if the intent is still in the mempool
        let batch_intents = self
            .mempool
            .mempool_data
            .iter()
            .filter_map(|(id, intent)| {
                if intent_ids.contains(&(*id as i32)) {
                    intent_ids.retain(|&i| i != *id as i32);
                    Some(intent.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<Intent>>();

        if intent_ids.is_empty() {
            return Ok(GetBatchIntentsResponse {
                batch_intents_json: batch_intents
                    .iter()
                    .map(|intent| {
                        serde_json::to_value(intent).expect("Failed to deserialize intent")
                    })
                    .collect(),
                message: String::from("GET batch intents successfully"),
                is_success: true,
            });
        }

        // let missing_intent_ids = intent_ids.iter().filter(|id| );

        let mut tx = self.storage_connection.create_transaction().map_err(|e| {
            error!(
                "Failed to store intent batch to database, with error: {}",
                e
            );
            Error::InternalError
        })?;

        let batch_intents = tx.get_intents_batch(&intent_ids).map_err(|e| {
            error!("Failed to store batch of intents, with error: {:?}", e);
            Error::InternalError
        })?;
        let batch_intents_json = batch_intents
            .iter()
            .map(|intent| {
                let intent = intent.to_intent().expect("Failed to convert intent");
                serde_json::to_value(intent).expect("Failed to deserialize intent")
            })
            .collect();

        Ok(GetBatchIntentsResponse {
            batch_intents_json,
            message: String::from("GET batch intents successfully"),
            is_success: true,
        })
    }

    pub fn handle_get_auth_credentials_request(
        &mut self,
        request: GetAuthCredentialsRequest,
    ) -> Result<GetAuthCredentialsResponse> {
        let address = request.address;
        let challenge = generate_challenge();

        {
            let mut tx = self
                .storage_connection()
                .create_transaction()
                .map_err(|e| {
                    error!(
                        "Failed to store intent batch to database, with error: {}",
                        e
                    );
                    Error::InternalError
                })?;

            tx.insert_new_credential(address, challenge.clone())
                .map_err(|e| {
                    error!("Failed to insert new credential to DB, with error: {}", e);
                    Error::InternalError
                })?;

            info!("New challenge {}, stored in the database", challenge);
        }

        Ok(GetAuthCredentialsResponse {
            challenge,
            is_success: true,
            message: "New challenge successfully generated".to_string(),
        })
    }

    pub(crate) fn handle_solver_registration(
        &mut self,
        request: RegisterSolverRequest,
    ) -> Result<RegisterSolverResponse> {
        let RegisterSolverRequest {
            solver_address,
            address_signature,
        } = request;

        let address: Address = Address::from_str(&solver_address).map_err(|e| {
            error!(
                "Failed to extract Address from public key, address = {}, error = {}",
                solver_address, e
            );
            Error::InternalError
        })?;
        match Signature::from_str(&address_signature) {
            Ok(sig) => sig.verify(solver_address.clone(), address).map_err(|e| {
                error!(
                    "Failed to recover solver address hash from signature, with error: {}",
                    e
                );
                Error::InvalidRequest
            })?,
            Err(e) => {
                error!("Failed to obtain signature from request, with error: {}", e);
                return Err(Error::InvalidRequest);
            }
        };

        {
            let mut tx = self
                .storage_connection()
                .create_transaction()
                .map_err(|e| {
                    error!("Failed to retrieve database transaction, with error: {}", e);
                    Error::InternalError
                })?;

            tx.register_solver(solver_address.clone()).map_err(|e| {
                error!("Failed to store new solver data to DB, with error: {}", e);
                Error::InternalError
            })?;

            info!(
                "New solver with address={}, registered in the database",
                solver_address
            );
        }

        Ok(RegisterSolverResponse {
            is_success: true,
            message: "New solver registration successfully submitted".to_string(),
        })
    }
}

impl SolinaWorker {
    pub(crate) fn get_current_credential(&mut self, address: &String) -> Result<AuthCredentials> {
        let mut tx = self
            .storage_connection()
            .create_transaction()
            .map_err(|e| {
                error!("Failed to connect to the database, with error: {}", e);
                Error::InternalError
            })?;

        tx.get_current_auth_credential(address).map_err(|e| {
            error!("Failed to insert new credential to DB, with error: {}", e);
            Error::InternalError
        })
    }

    pub(crate) fn update_is_valid_credential(&mut self, id: i32) -> Result<()> {
        let mut tx = self
            .storage_connection()
            .create_transaction()
            .map_err(|e| {
                error!(
                    "Failed to update is_valid credential to database, with error: {}",
                    e
                );
                Error::InternalError
            })?;
        tx.update_is_valid_credential(id).map_err(|e| {
            error!("Failed to update is_auth credential, with error: {}", e);
            Error::InternalError
        })
    }

    pub(crate) fn update_is_auth_credential(&mut self, id: i32) -> Result<()> {
        let mut tx = self
            .storage_connection()
            .create_transaction()
            .map_err(|e| {
                error!(
                    "Failed to update is_auth credential to database, with error: {}",
                    e
                );
                Error::InternalError
            })?;
        tx.update_is_auth_credential(id).map_err(|e| {
            error!("Failed to update is_valid credential, with error: {}", e);
            Error::InternalError
        })
    }
}
