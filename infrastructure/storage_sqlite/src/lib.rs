#[macro_use]
extern crate diesel;

mod models;
mod reader_writer;
mod schema;

use crate::reader_writer::ReadWriterTransaction;
use diesel::{sql_query, Connection, RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use solina_service::errors::SolinaError;
use std::{
    fs::create_dir_all,
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct SolinaStorage {
    connection: Arc<Mutex<SqliteConnection>>,
}

impl SolinaStorage {
    pub fn try_open<P: AsRef<Path>>(path: P) -> Result<Self, SolinaError> {
        create_dir_all(path.as_ref().parent().unwrap()).expect("Failed to create DB path");

        let database_url = path
            .as_ref()
            .to_str()
            .expect("database_url utf-8 error")
            .to_string();
        let mut connection = SqliteConnection::establish(&database_url)
            .map_err(|e| SolinaError::SolinaStorageError(e.to_string()))?;

        sql_query("PRAGMA foreign_keys = ON;")
            .execute(&mut connection)
            .map_err(|e| SolinaError::SolinaStorageError(e.to_string()))?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn run_migrations(&self) -> Result<(), SolinaError> {
        let mut connection = self.connection.lock().unwrap();
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| SolinaError::SolinaStorageError(e.to_string()))?;
        Ok(())
    }
}

impl SolinaStorage {
    pub fn create_read_transaction(&self) -> Result<ReadWriterTransaction<'_>, SolinaError> {
        let mut lock = self.connection.lock().unwrap();
        sql_query("BEGIN")
            .execute(&mut *lock)
            .map_err(|e| SolinaError::SolinaStorageError(e.to_string()))?;
        Ok(ReadWriterTransaction::new(lock))
    }
}
