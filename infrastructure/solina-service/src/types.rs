use serde::{Deserialize, Serialize};
use solina::{intent::Intent, Uuid};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentRequest {
    pub(crate) intent_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentResponse {
    pub(crate) intent_id: Option<Uuid>,
    pub(crate) is_success: bool,
    pub(crate) message: String,
}
