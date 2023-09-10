use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // Authentication errors.
    AuthError,
    // -- Request errors.
    InvalidRequest,
    // -- Server errors.
    FailedToStartService,
    InternalError,
    // -- Model errors.
    FailedToStoreIntent,
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        response.extensions_mut().insert(self);
        response
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            // -- Auth errors.
            Self::AuthError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::AUTH_ERROR),
            // -- Request errors.
            Self::InvalidRequest => (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS),
            // -- Server
            Self::FailedToStartService => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::INTERNAL_SERVER_ERROR,
            ),
            Self::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::INTERNAL_SERVER_ERROR,
            ),
            // -- Model
            Self::FailedToStoreIntent => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    AUTH_ERROR,
    INTERNAL_SERVER_ERROR,
    INVALID_PARAMS,
    SERVICE_ERROR,
}
