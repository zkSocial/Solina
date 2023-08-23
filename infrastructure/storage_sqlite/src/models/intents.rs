use chrono::NaiveDateTime;
use diesel::{Identifiable, Queryable};

use crate::schema::intents;

#[derive(Clone, Debug, Queryable, Identifiable)]
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
