//! Server-function declarations visible to every build. Bodies run only on the server.

mod auth;
mod error;

pub use auth::{AuthStatus, Credentials, SignedIn, auth_status, login, logout, register};
pub use error::ApiError;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use {
    crate::models::{Field, ValidationError},
    crate::server::AppState,
    dioxus::server::axum::{Extension, http::HeaderMap},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackendStatus {
    pub application: String,
}

/// Public connectivity probe, usable before signing in.
#[get("/api/health", state: Extension<AppState>)]
pub async fn get_status() -> std::result::Result<BackendStatus, ApiError> {
    Ok(BackendStatus {
        application: state.application.to_owned(),
    })
}

/// Temporary milestone 0 probe; now also exercises the sign-in requirement.
#[post("/api/echo", state: Extension<AppState>, headers: HeaderMap)]
pub async fn echo_message(message: String) -> std::result::Result<String, ApiError> {
    auth::require_user(&state, &headers).await?;
    let message = message.trim();
    if message.is_empty() || message.chars().count() > 200 {
        return Err(ApiError::Validation(ValidationError::new(
            Field::Input,
            "Enter a message between 1 and 200 characters.",
        )));
    }

    Ok(format!("{} received: {message}", state.application))
}
