use anyhow::{anyhow, Error};
use diesel::{sql_query, SqliteConnection};

pub struct ReadTransaction<'a> {
    connection: MutexGuard<'a, SqliteConnection>,
    is_done: bool,
}

impl<'a> ReadTransaction<'a> {
    pub fn new(connection: MutexGuard<'a, SqliteConnection>) -> Self {
        Self {
            connection,
            is_done: false,
        }
    }

    pub(super) fn is_done(&self) -> bool {
        self.is_done
    }

    pub(super) fn connection(&mut self) -> &mut SqliteConnection {
        &mut self.connection
    }

    pub(super) fn commit(&mut self) -> Result<(), Error> {
        sql_query("COMMIT")
            .execute(self.connection())
            .map_err(|e| {
                anyhow!(
                    "Failed to commit transaction, with error: {}",
                    e.to_string()
                )
            })?;
        self.is_done = true;
        Ok(())
    }

    pub(super) fn rollback(&mut self) -> Result<(), Error> {
        sql_query("ROLLBACK")
            .execute(self.connection())
            .map_err(|e| {
                anyhow!(
                    "Failed to rollback transaction, with error: {}",
                    e.to_string()
                )
            })?;
        self.is_done = true;
        Ok(())
    }
}

impl<'a> ReadTransaction<'a> {
    pub(super) fn get_intent(id: Uuid) -> Option<Intent> {
        None
    }
}
