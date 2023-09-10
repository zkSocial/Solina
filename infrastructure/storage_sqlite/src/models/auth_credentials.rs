use crate::schema::auth_credentials;
use chrono::{NaiveDateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name=auth_credentials)]
pub struct AuthCredentials {
    pub id: i32,
    pub address: String,
    pub challenge: String,
    pub is_auth: bool,
    pub is_valid: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name=auth_credentials)]
pub struct NewAuthCredentials {
    pub address: String,
    pub challenge: String,
    pub is_auth: bool,
    pub is_valid: bool,
    pub created_at: NaiveDateTime,
}
