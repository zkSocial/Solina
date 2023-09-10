#[macro_use]
extern crate diesel;

mod error;
mod models;
mod reader_writer;
mod schema;

use crate::{error::SolinaStorageError, reader_writer::ReadWriterTransaction};
use diesel::{sql_query, Connection, RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{
    fs::create_dir_all,
    path::Path,
    sync::{Arc, Mutex},
};

pub use models::AuthCredentials;

#[derive(Clone)]
pub struct SolinaStorage {
    connection: Arc<Mutex<SqliteConnection>>,
}

impl SolinaStorage {
    pub fn try_open<P: AsRef<Path>>(path: P) -> Result<Self, SolinaStorageError> {
        create_dir_all(path.as_ref().parent().unwrap()).expect("Failed to create DB path");

        let database_url = path
            .as_ref()
            .to_str()
            .expect("database_url utf-8 error")
            .to_string();
        let mut connection = SqliteConnection::establish(&database_url)
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        sql_query("PRAGMA foreign_keys = ON;")
            .execute(&mut connection)
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn run_migrations(&self) -> Result<(), SolinaStorageError> {
        let mut connection = self.connection.lock().unwrap();
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| SolinaStorageError::StorageError(e.to_string()))?;
        Ok(())
    }
}

impl SolinaStorage {
    pub fn create_transaction(&self) -> Result<ReadWriterTransaction<'_>, SolinaStorageError> {
        let lock = self.connection.lock().unwrap();
        Ok(ReadWriterTransaction::new(lock))
    }
}
