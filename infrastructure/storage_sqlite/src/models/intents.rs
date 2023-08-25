use chrono::{NaiveDateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use hex::encode;
use solina::intent::Intent as SolinaIntent;
use solina::structured_hash::StructuredHashInterface;

use crate::schema::intents;

#[derive(Clone, Debug, Queryable, Identifiable, Insertable)]
#[diesel(table_name = intents)]
pub struct Intent {
    pub id: String,
    pub public_key: String,
    pub signature: String,
    pub base_token: String,
    pub quote_token: String,
    pub quote_amount: i64,
    pub direction: bool,
    pub min_base_token_amount: i64,
    pub created_at: NaiveDateTime,
}

impl Intent {
    pub fn from_intent(intent: &SolinaIntent) -> Self {
        let id = encode(intent.structured_hash());
        let public_key = encode(&intent.public_key);
        let signature = encode(&intent.signature.0);
        let base_token = encode(&intent.inputs.base_token);
        let quote_token = encode(&intent.inputs.quote_token);
        let mut min_base_token_amount_bytes = [0u8; 8];
        min_base_token_amount_bytes
            .copy_from_slice(&intent.constraints.min_base_token_amount.to_bytes_be());
        let min_base_token_amount = i64::from_be_bytes(min_base_token_amount_bytes); // TODO: for now we use i64 representations, need refactor
        let mut quote_amount_bytes = [0u8; 8];
        quote_amount_bytes.copy_from_slice(&intent.inputs.quote_amount.to_bytes_be());
        let quote_amount = i64::from_be_bytes(quote_amount_bytes);
        let created_at = Utc::now().naive_utc();
        let direction = intent.inputs.direction.to_bool();

        Self {
            id,
            public_key,
            signature,
            base_token,
            quote_token,
            min_base_token_amount,
            quote_amount,
            created_at,
            direction,
        }
    }
}
