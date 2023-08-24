use thiserror::Error;

#[derive(Debug, Error)]
pub enum SolinaStorageError {
    #[error("Storage Error: `{0}`")]
    StorageError(String),
}
