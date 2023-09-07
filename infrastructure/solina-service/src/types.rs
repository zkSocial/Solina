use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentRequest {
    pub(crate) intent_json: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentResponse {
    pub(crate) intent_id: Option<i64>,
    pub(crate) is_success: bool,
    pub(crate) message: String,
}
