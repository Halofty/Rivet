use dioxus::fullstack::{AsStatusCode, ServerFnError, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Validation(String),
    #[error("Could not reach the server. Check the connection and try again.")]
    Transport(#[from] ServerFnError),
}

impl AsStatusCode for ApiError {
    fn as_status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Transport(error) => error.as_status_code(),
        }
    }
}
