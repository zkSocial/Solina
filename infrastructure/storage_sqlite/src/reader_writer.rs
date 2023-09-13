use crate::{
    error::SolinaStorageError,
    models::{AuthCredentials, Intent, NewAuthCredentials, NewSolver},
};
use chrono::Utc;
use diesel::{
    sql_query, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection,
};
use solina::intent;
use std::sync::MutexGuard;

// Sqlite does not make a distinction between read and write transactions.
// Therefore, any transaction is writable. We will need to refactor this,
// once we get past Sqlite.
pub struct ReadWriterTransaction<'a> {
    connection: MutexGuard<'a, SqliteConnection>,
    is_done: bool,
}

impl<'a> ReadWriterTransaction<'a> {
    pub fn new(connection: MutexGuard<'a, SqliteConnection>) -> Self {
        Self {
            connection,
            is_done: false,
        }
    }

    pub fn connection(&mut self) -> &mut SqliteConnection {
        &mut self.connection
    }

    pub fn commit(&mut self) -> Result<(), SolinaStorageError> {
        sql_query("COMMIT")
            .execute(self.connection())
            .map_err(|e| {
                SolinaStorageError::StorageError(format!(
                    "Failed to commit transaction, with error: {}",
                    e
                ))
            })?;
        self.is_done = true;
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), SolinaStorageError> {
        sql_query("ROLLBACK")
            .execute(self.connection())
            .map_err(|e| {
                SolinaStorageError::StorageError(format!(
                    "Failed to rollback transaction, with error: {}",
                    e
                ))
            })?;
        self.is_done = true;
        Ok(())
    }
}

impl<'a> ReadWriterTransaction<'a> {
    // ----------------------------------------------- Read methods -----------------------------------------------
    pub fn get_intent(&mut self, id: i32) -> Result<Intent, SolinaStorageError> {
        use crate::schema::intents;

        let result = intents::table
            .filter(intents::id.eq(id))
            .first(self.connection())
            .optional()
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        match result {
            Some(output) => Ok(output),
            None => Err(SolinaStorageError::StorageError(format!(
                "Could not find stored intent with id: {}",
                id,
            ))),
        }
    }

    pub fn get_intents_batch(&mut self, ids: &[i32]) -> Result<Vec<Intent>, SolinaStorageError> {
        use crate::schema::intents;

        let results = intents::table
            .filter(intents::id.eq_any(ids))
            .load::<Intent>(self.connection())
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        if results.is_empty() {
            Err(SolinaStorageError::StorageError(
                "Could not find stored intents for the provided ids".to_string(),
            ))
        } else {
            Ok(results)
        }
    }

    pub fn get_current_auth_credential(
        &mut self,
        address: &String,
    ) -> Result<AuthCredentials, SolinaStorageError> {
        use crate::schema::auth_credentials;

        let credential = auth_credentials::table
            .filter(auth_credentials::address.eq(address))
            .order(auth_credentials::id.desc())
            .first(self.connection())
            .optional()
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        match credential {
            Some(output) => Ok(output),
            None => Err(SolinaStorageError::StorageError(format!(
                "Could not find credential for address: {}",
                address,
            ))),
        }
    }

    // ----------------------------------------------- Write methods -----------------------------------------------
    pub fn store_intents(
        &mut self,
        intents: &[(i64, intent::Intent)],
    ) -> Result<(), SolinaStorageError> {
        use crate::schema::current_batch_id;
        use crate::schema::intents;

        let current_batch_id = current_batch_id::table
            .order(current_batch_id::id.desc())
            .first(self.connection())
            .optional()
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?
            .unwrap_or(0);
        let intents = intents
            .iter()
            .map(|(id, intent)| Intent::from_intent(intent, *id as i32, current_batch_id))
            .collect::<Vec<_>>();
        diesel::insert_into(intents::table)
            .values(intents)
            .execute(self.connection())
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        Ok(())
    }

    pub fn insert_new_credential(
        &mut self,
        address: String,
        challenge: String,
    ) -> Result<(), SolinaStorageError> {
        use crate::schema::auth_credentials;

        diesel::insert_into(auth_credentials::table)
            .values(NewAuthCredentials {
                address,
                challenge,
                is_auth: false,
                is_valid: true,
                created_at: Utc::now().naive_utc(),
            })
            .execute(self.connection())
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        Ok(())
    }

    pub fn update_is_auth_credential(&mut self, id: i32) -> Result<(), SolinaStorageError> {
        use crate::schema::auth_credentials;

        diesel::update(auth_credentials::table.filter(auth_credentials::id.eq(id)))
            .set(auth_credentials::is_auth.eq(true))
            .execute(self.connection())
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        Ok(())
    }

    pub fn update_is_valid_credential(&mut self, id: i32) -> Result<(), SolinaStorageError> {
        use crate::schema::auth_credentials;

        diesel::update(auth_credentials::table.filter(auth_credentials::id.eq(id)))
            .set(auth_credentials::is_valid.eq(false))
            .execute(self.connection())
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        Ok(())
    }

    pub fn register_solver(&mut self, address: String) -> Result<(), SolinaStorageError> {
        use crate::schema::solvers;

        diesel::insert_into(solvers::table)
            .values(NewSolver { address })
            .execute(self.connection())
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        Ok(())
    }
}
