use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StoreIntentRequest {
    pub(crate) intent_json: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StoreIntentResponse {
    pub(crate) intent_id: Option<i64>,
    pub(crate) is_success: bool,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetIntentRequest {
    pub(crate) id: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetIntentResponse {
    pub(crate) intent_json: serde_json::Value,
    pub(crate) is_success: bool,
    pub(crate) message: String,
}
