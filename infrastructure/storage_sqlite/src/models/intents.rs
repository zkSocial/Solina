use crate::error::SolinaStorageError;
use crate::schema::intents;
use chrono::{NaiveDateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use hex::{decode, encode};
use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use solina::structured_hash::StructuredHashInterface;
use solina::{
    intent::{Intent as SolinaIntent, IntentConstraints, IntentInputs, TradeDirection},
    Signature,
};

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
        let public_key = encode(intent.public_key);
        let signature = encode(intent.signature.0);
        let base_token = encode(intent.inputs.base_token);
        let quote_token = encode(intent.inputs.quote_token);
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

    pub fn to_intent(&self) -> Result<SolinaIntent, SolinaStorageError> {
        let mut public_key = [0_u8; 32];
        let public_key_buffer = decode(&self.public_key)
            .map_err(|e| SolinaStorageError::ConversionError(e.to_string()))?;
        public_key.copy_from_slice(&public_key_buffer);
        let mut signature = [0_u8; 64];
        let signature_buffer = decode(&self.signature)
            .map_err(|e| SolinaStorageError::ConversionError(e.to_string()))?;
        signature.copy_from_slice(&signature_buffer);
        let mut base_token = [0_u8; 32];
        let base_token_buffer = decode(&self.base_token)
            .map_err(|e| SolinaStorageError::ConversionError(e.to_string()))?;
        base_token.copy_from_slice(&base_token_buffer);
        let mut quote_token = [0_u8; 32];
        let quote_token_buffer = decode(&self.quote_token)
            .map_err(|e| SolinaStorageError::ConversionError(e.to_string()))?;
        quote_token.copy_from_slice(&quote_token_buffer);
        let min_base_token_amount = BigUint::from(self.min_base_token_amount as u64);
        let quote_amount = BigUint::from(self.quote_amount as u64);
        let direction = TradeDirection::from_bool(self.direction);

        let intent_constraints = IntentConstraints::new(min_base_token_amount);
        let intent_inputs = IntentInputs::new(base_token, quote_token, quote_amount, direction);
        Ok(SolinaIntent::new(
            public_key,
            intent_inputs,
            intent_constraints,
            Signature(signature),
        ))
    }
}
