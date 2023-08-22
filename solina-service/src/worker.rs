use crate::{
    errors::SolinaError,
    intents::Intent,
    types::{IntentRequest, IntentResponse},
};
use bincode::deserialize;
use core::pin::Pin;
use futures::{stream::FuturesUnordered, Future};
use solina::structured_hash::StructuredHashInterface;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct SolinaWorker {
    rx_intent_request: Receiver<IntentRequest>,
    tx_intent_response: Sender<IntentResponse>,
}

// TODO: add logic for shutdown signal
impl SolinaWorker {
    pub fn new(
        rx_intent_request: Receiver<IntentRequest>,
        tx_intent_response: Sender<IntentResponse>,
    ) -> Self {
        Self {
            rx_intent_request,
            tx_intent_response,
        }
    }

    pub async fn spawn(&self) {}

    pub async fn run(&mut self) -> Result<(), SolinaError> {
        while let Some(intent_request) = self.rx_intent_request.recv().await {
            let bytes_data = intent_request.intent_bytes.clone();
            let intent = match Err(e) = deserialize::<Intent>(&bytes_data) {
                Ok(inner) => inner,
                Err(e) => self.tx_intent_response.send(IntentResponse {
                    intent_id: None,
                    is_success: false,
                    message: e.to_string(),
                }),
            };
            let intent_structured_hash = intent.structured_hash();
            let response = IntentResponse {
                intent_id: Some(Uuid {
                    id: intent_structured_hash,
                }),
                is_success: true,
                message: "Success".to_string(),
            };
            self.tx_intent_response.send(response);
        }
        Ok(())
    }
}
