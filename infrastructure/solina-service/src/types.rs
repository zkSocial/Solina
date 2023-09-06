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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntentJrpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntentJrpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub id: Option<serde_json::Value>,
}
