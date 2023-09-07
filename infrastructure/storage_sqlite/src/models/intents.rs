use chrono::{NaiveDateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use hex::encode;
use num_traits::cast::ToPrimitive;
use solina::intent::Intent as SolinaIntent;
use solina::structured_hash::StructuredHashInterface;

use crate::schema::intents;

#[derive(Debug, Queryable, Identifiable, Insertable)]
#[diesel(table_name = intents)]
pub struct Intent {
    pub id: i32,
    pub structured_hash: String,
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
    pub fn from_intent(intent: &SolinaIntent, id: i32) -> Self {
        let structured_hash = encode(intent.structured_hash());
        let public_key = encode(&intent.public_key);
        let signature = encode(&intent.signature.0);
        let base_token = encode(&intent.inputs.base_token);
        let quote_token = encode(&intent.inputs.quote_token);
        let min_base_token_amount = intent.constraints.min_base_token_amount.to_i64().unwrap(); // TODO: for now we use i64 representations, need refactor
        let quote_amount = intent.inputs.quote_amount.to_i64().unwrap();
        let created_at = Utc::now().naive_utc();
        let direction = intent.inputs.direction.to_bool();

        Self {
            id,
            structured_hash,
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
