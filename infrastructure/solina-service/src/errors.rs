use thiserror::Error;

#[derive(Debug, Error)]
pub enum SolinaError {
    #[error("Solina Worker error: {0}")]
    SolinaWorkerError(String),
}
