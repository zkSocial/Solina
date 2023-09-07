use crate::{error::SolinaStorageError, models::Intent};
use diesel::{
    dsl::max, sql_query, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    SqliteConnection,
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

    pub fn get_intents_batch() -> Option<intent::Intent> {
        None
    }

    // ----------------------------------------------- Write methods -----------------------------------------------
    pub fn store_intents(&mut self, intents: &[intent::Intent]) -> Result<(), SolinaStorageError> {
        use crate::schema::intents;

        let nullable_id: Option<Option<i32>> = intents::table
            .select(max(intents::id))
            .first(self.connection())
            .optional()
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;
        let id = nullable_id
            .ok_or_else(|| {
                SolinaStorageError::StorageError(String::from("Failed to retrieve table max id"))
            })?
            .unwrap_or(0);

        let intents = (0..intents.len())
            .map(|i| Intent::from_intent(&intents[i], (id as usize + i + 1) as i32))
            .collect::<Vec<_>>();
        diesel::insert_into(intents::table)
            .values(intents)
            .execute(self.connection())
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        Ok(())
    }
}
