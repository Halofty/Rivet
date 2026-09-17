//! Temporary milestone 0 endpoints: replace the echo probe with product flows later.

mod error;

pub use error::ApiError;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use {crate::server::AppState, dioxus::server::axum::Extension};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackendStatus {
    pub application: String,
}

#[get("/api/health", state: Extension<AppState>)]
pub async fn get_status() -> std::result::Result<BackendStatus, ApiError> {
    Ok(BackendStatus {
        application: state.application.to_owned(),
    })
}

#[post("/api/echo", state: Extension<AppState>)]
pub async fn echo_message(message: String) -> std::result::Result<String, ApiError> {
    let message = message.trim();
    if message.is_empty() || message.chars().count() > 200 {
        return Err(ApiError::Validation(
            "Enter a message between 1 and 200 characters.".to_owned(),
        ));
    }

    Ok(format!("{} received: {message}", state.application))
}
