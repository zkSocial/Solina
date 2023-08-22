use crate::intents::Intent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentRequest {
    intent: Intent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentResponse {
    is_sucess: bool,
}
