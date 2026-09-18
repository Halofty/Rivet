use super::ApiError;
use crate::models::{ClientKind, User};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use {
    crate::server::{AppState, auth, sessions::SessionToken, users::UserRecord},
    dioxus::fullstack::FullstackContext,
    dioxus::server::axum::{
        Extension,
        http::{HeaderMap, header},
    },
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthStatus {
    pub user: Option<User>,
    /// Whether this request may create an account (first-user setup or open registration).
    pub registration_open: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
    pub client: ClientKind,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("email", &self.email)
            .field("password", &"..")
            .field("client", &self.client)
            .finish()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedIn {
    pub user: User,
    /// Returned only to desktop clients, which keep it in the OS credential store.
    /// Web sessions travel in an HttpOnly cookie that page scripts cannot read.
    pub token: Option<String>,
}

impl std::fmt::Debug for SignedIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignedIn")
            .field("user", &self.user)
            .field("token", &self.token.as_ref().map(|_| ".."))
            .finish()
    }
}

#[get("/api/auth/status", state: Extension<AppState>, headers: HeaderMap)]
pub async fn auth_status() -> std::result::Result<AuthStatus, ApiError> {
    let user = state
        .current_user(&headers)
        .await
        .map_err(|e| ApiError::internal("auth_status")(&e))?
        .map(|record| record.user);
    let registration_open = match state.registration_policy(&headers) {
        Some(policy) => crate::server::users::registration_open(&state.pool, policy)
            .await
            .map_err(|e| ApiError::internal("auth_status")(&e))?,
        None => false,
    };
    Ok(AuthStatus {
        user,
        registration_open,
    })
}

#[post("/api/auth/register", state: Extension<AppState>, headers: HeaderMap)]
pub async fn register(credentials: Credentials) -> std::result::Result<SignedIn, ApiError> {
    let policy = state
        .registration_policy(&headers)
        .ok_or(ApiError::RegistrationClosed)?;
    let (record, token) = state
        .auth
        .register(
            &state.pool,
            policy,
            &credentials.email,
            &credentials.password,
            credentials.client,
        )
        .await?;
    Ok(signed_in(&state, record, token, credentials.client))
}

#[post("/api/auth/login", state: Extension<AppState>)]
pub async fn login(credentials: Credentials) -> std::result::Result<SignedIn, ApiError> {
    let (record, token) = state
        .auth
        .login(
            &state.pool,
            &credentials.email,
            &credentials.password,
            credentials.client,
        )
        .await?;
    Ok(signed_in(&state, record, token, credentials.client))
}

/// Ends the current session. Succeeds even when no session is present.
#[post("/api/auth/logout", state: Extension<AppState>, headers: HeaderMap)]
pub async fn logout() -> std::result::Result<(), ApiError> {
    if let Some(token) = auth::token_from_headers(&headers) {
        crate::server::sessions::delete_session(&state.pool, &token)
            .await
            .map_err(|e| ApiError::internal("logout")(&e))?;
    }
    set_cookie(auth::clear_session_cookie(state.config.secure_cookies()));
    Ok(())
}

/// The signed-in account for a request, or `Unauthorized`. Every owned server function
/// starts here and passes the returned [`UserRecord::id`] to the data layer.
#[cfg(feature = "server")]
pub(crate) async fn require_user(
    state: &AppState,
    headers: &HeaderMap,
) -> std::result::Result<UserRecord, ApiError> {
    state
        .current_user(headers)
        .await
        .map_err(|e| ApiError::internal("require_user")(&e))?
        .ok_or(ApiError::Unauthorized)
}

#[cfg(feature = "server")]
fn signed_in(
    state: &AppState,
    record: UserRecord,
    token: SessionToken,
    client: ClientKind,
) -> SignedIn {
    match client {
        ClientKind::Web => {
            set_cookie(auth::session_cookie(&token, state.config.secure_cookies()));
            SignedIn {
                user: record.user,
                token: None,
            }
        }
        ClientKind::Desktop => SignedIn {
            user: record.user,
            token: Some(token.as_str().to_owned()),
        },
    }
}

#[cfg(feature = "server")]
fn set_cookie(value: dioxus::server::axum::http::HeaderValue) {
    if let Some(context) = FullstackContext::current() {
        context.add_response_header(header::SET_COOKIE, value);
    }
}
