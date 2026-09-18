use crate::models::ValidationError;
use dioxus::fullstack::{AsStatusCode, ServerFnError, StatusCode};
use serde::{Deserialize, Serialize};

/// Transport-facing errors. Messages are safe to show to users: internal details such as
/// SQL, file paths, or hashes are logged on the server and never serialized.
#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Validation(ValidationError),
    #[error("Sign in to continue.")]
    Unauthorized,
    #[error("Email or password is incorrect.")]
    InvalidCredentials,
    #[error("New accounts cannot be created on this server.")]
    RegistrationClosed,
    #[error("Too many attempts. Wait a few minutes and try again.")]
    RateLimited,
    #[error("That item could not be found.")]
    NotFound,
    #[error("Something went wrong on the server. Please try again.")]
    Internal,
    #[error("Could not reach the server. Check the connection and try again.")]
    Transport(#[from] ServerFnError),
}

impl AsStatusCode for ApiError {
    fn as_status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized | Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::RegistrationClosed => StatusCode::FORBIDDEN,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Transport(error) => error.as_status_code(),
        }
    }
}

#[cfg(feature = "server")]
impl ApiError {
    /// Logs an unexpected failure with its operation and returns the generic error.
    pub(crate) fn internal(operation: &'static str) -> impl FnOnce(&dyn std::fmt::Display) -> Self {
        move |error| {
            dioxus::logger::tracing::error!(operation, %error, "request failed");
            Self::Internal
        }
    }
}

#[cfg(feature = "server")]
impl From<crate::server::auth::AuthError> for ApiError {
    fn from(error: crate::server::auth::AuthError) -> Self {
        use crate::server::auth::AuthError;
        match error {
            AuthError::Validation(error) => Self::Validation(error),
            AuthError::InvalidCredentials => Self::InvalidCredentials,
            AuthError::RegistrationClosed => Self::RegistrationClosed,
            AuthError::RateLimited => Self::RateLimited,
            AuthError::Internal(detail) => Self::internal("auth")(&detail),
        }
    }
}
