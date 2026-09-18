use super::{
    sessions::{self, SessionError, SessionToken},
    users::{self, CreateUser, Registration, UserRecord},
};
use crate::models::{ClientKind, ValidationError, normalize_email, validate_password};
use argon2::{
    Argon2, Params,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use dioxus::server::axum::http::{HeaderMap, HeaderValue, header};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

pub const SESSION_COOKIE: &str = "rivet_session";
const MAX_FAILURES: usize = 5;
const FAILURE_WINDOW: Duration = Duration::from_secs(15 * 60);
/// Argon2id with the default parameters uses about 19 MiB per hash; cap concurrent work.
const CONCURRENT_HASHES: usize = 2;
const TRACKED_KEYS_BEFORE_PRUNE: usize = 1024;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Validation(ValidationError),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("registration is closed")]
    RegistrationClosed,
    #[error("too many failed sign-in attempts")]
    RateLimited,
    #[error("internal authentication failure: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for AuthError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(error.to_string())
    }
}

impl From<SessionError> for AuthError {
    fn from(error: SessionError) -> Self {
        Self::Internal(error.to_string())
    }
}

/// Password hashing and sign-in throttling shared by every request.
pub struct Auth {
    params: Params,
    hashing: Semaphore,
    failures: Mutex<HashMap<String, Vec<Instant>>>,
    /// Verified when an email is unknown, so response time does not reveal accounts.
    dummy_hash: OnceLock<String>,
}

impl Auth {
    /// Production settings: Argon2id with the crate's OWASP-aligned defaults.
    pub fn new() -> Arc<Self> {
        Self::with_params(Params::default())
    }

    /// Tests pass cheap parameters; stored hashes record their own parameters.
    pub fn with_params(params: Params) -> Arc<Self> {
        Arc::new(Self {
            params,
            hashing: Semaphore::new(CONCURRENT_HASHES),
            failures: Mutex::new(HashMap::new()),
            dummy_hash: OnceLock::new(),
        })
    }

    pub async fn register(
        self: &Arc<Self>,
        pool: &SqlitePool,
        policy: Registration,
        email: &str,
        password: &str,
        client: ClientKind,
    ) -> Result<(UserRecord, SessionToken), AuthError> {
        // Cheap checks first, so a closed server never spends time hashing.
        if !users::registration_open(pool, policy).await? {
            return Err(AuthError::RegistrationClosed);
        }
        let email = normalize_email(email).map_err(AuthError::Validation)?;
        validate_password(password).map_err(AuthError::Validation)?;
        let hash = self.hash(password.to_owned()).await?;
        let record = match users::create_user(pool, &email, &hash, policy).await? {
            CreateUser::Created(record) => record,
            CreateUser::EmailTaken => {
                return Err(AuthError::Validation(ValidationError::new(
                    crate::models::Field::Email,
                    "An account with this email already exists.",
                )));
            }
            CreateUser::Closed => return Err(AuthError::RegistrationClosed),
        };
        let token = sessions::create_session(pool, record.id, client).await?;
        Ok((record, token))
    }

    pub async fn login(
        self: &Arc<Self>,
        pool: &SqlitePool,
        email: &str,
        password: &str,
        client: ClientKind,
    ) -> Result<(UserRecord, SessionToken), AuthError> {
        // Malformed input gets the same answer as a wrong password.
        let key = email.trim().to_lowercase();
        if !self.allow_attempt(&key) {
            return Err(AuthError::RateLimited);
        }
        let valid_shape = normalize_email(email).is_ok() && validate_password(password).is_ok();
        let record = if valid_shape {
            users::find_user_by_email(pool, &key).await?
        } else {
            None
        };
        let stored = match &record {
            Some(record) => record.password_hash.clone(),
            None => self.dummy_hash().await?,
        };
        let verified = self.verify(password.to_owned(), stored).await?;
        match record {
            Some(record) if verified && valid_shape => {
                self.clear_failures(&key);
                sessions::delete_expired_sessions(pool).await?;
                let token = sessions::create_session(pool, record.id, client).await?;
                Ok((record, token))
            }
            _ => {
                self.record_failure(key);
                Err(AuthError::InvalidCredentials)
            }
        }
    }

    async fn hash(self: &Arc<Self>, password: String) -> Result<String, AuthError> {
        let _permit = self.hashing.acquire().await.map_err(internal)?;
        let params = self.params.clone();
        tokio::task::spawn_blocking(move || {
            argon2(params)
                .hash_password(password.as_bytes())
                .map(|hash| hash.to_string())
                .map_err(internal)
        })
        .await
        .map_err(internal)?
    }

    async fn verify(self: &Arc<Self>, password: String, stored: String) -> Result<bool, AuthError> {
        let _permit = self.hashing.acquire().await.map_err(internal)?;
        let params = self.params.clone();
        tokio::task::spawn_blocking(move || {
            let parsed = PasswordHash::new(&stored).map_err(internal)?;
            Ok(argon2(params)
                .verify_password(password.as_bytes(), &parsed)
                .is_ok())
        })
        .await
        .map_err(internal)?
    }

    async fn dummy_hash(self: &Arc<Self>) -> Result<String, AuthError> {
        if let Some(hash) = self.dummy_hash.get() {
            return Ok(hash.clone());
        }
        let hash = self.hash("rivet-dummy-password".to_owned()).await?;
        Ok(self.dummy_hash.get_or_init(|| hash).clone())
    }

    fn allow_attempt(&self, key: &str) -> bool {
        let mut failures = self.failures.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        failures.get_mut(key).is_none_or(|times| {
            times.retain(|time| now.duration_since(*time) < FAILURE_WINDOW);
            times.len() < MAX_FAILURES
        })
    }

    fn record_failure(&self, key: String) {
        let mut failures = self.failures.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        if failures.len() >= TRACKED_KEYS_BEFORE_PRUNE {
            failures.retain(|_, times| {
                times.retain(|time| now.duration_since(*time) < FAILURE_WINDOW);
                !times.is_empty()
            });
        }
        failures.entry(key).or_default().push(now);
    }

    fn clear_failures(&self, key: &str) {
        let mut failures = self.failures.lock().unwrap_or_else(|e| e.into_inner());
        failures.remove(key);
    }
}

fn argon2(params: Params) -> Argon2<'static> {
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
}

fn internal(error: impl std::fmt::Display) -> AuthError {
    AuthError::Internal(error.to_string())
}

/// The session token from `Authorization: Bearer` (desktop) or the session cookie (web).
pub fn token_from_headers(headers: &HeaderMap) -> Option<SessionToken> {
    if let Some(value) = headers.get(header::AUTHORIZATION) {
        // A present but malformed Authorization header never falls back to the cookie.
        return value
            .to_str()
            .ok()
            .and_then(|value| value.strip_prefix("Bearer "))
            .and_then(|token| SessionToken::parse(token.trim()));
    }
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == SESSION_COOKIE)
        .and_then(|(_, token)| SessionToken::parse(token))
}

pub async fn authenticate(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> sqlx::Result<Option<(UserRecord, SessionToken)>> {
    let Some(token) = token_from_headers(headers) else {
        return Ok(None);
    };
    Ok(sessions::find_session_user(pool, &token)
        .await?
        .map(|record| (record, token)))
}

/// `HttpOnly` keeps the token away from page scripts; `SameSite=Lax` stops other sites
/// from sending it with cross-site form posts.
pub fn session_cookie(token: &SessionToken, secure: bool) -> HeaderValue {
    cookie(token.as_str(), sessions::SESSION_LIFETIME_SECS, secure)
}

pub fn clear_session_cookie(secure: bool) -> HeaderValue {
    cookie("", 0, secure)
}

fn cookie(value: &str, max_age: i64, secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}"
    ))
    .expect("session cookies contain only visible ASCII")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token() -> SessionToken {
        SessionToken::parse(&"ab".repeat(32)).unwrap()
    }

    #[test]
    fn bearer_header_takes_precedence_and_malformed_values_are_ignored() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("theme=dark; {SESSION_COOKIE}={}", "cd".repeat(32)))
                .unwrap(),
        );
        assert_eq!(
            token_from_headers(&headers).unwrap().as_str(),
            "cd".repeat(32)
        );

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token().as_str())).unwrap(),
        );
        assert_eq!(token_from_headers(&headers), Some(token()));

        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Basic abc"));
        assert_eq!(token_from_headers(&headers), None);
    }

    #[test]
    fn cookies_are_http_only_and_same_site() {
        let value = session_cookie(&token(), true);
        let value = value.to_str().unwrap();
        assert!(value.starts_with(&format!("{SESSION_COOKIE}={}", token().as_str())));
        for attribute in ["HttpOnly", "SameSite=Lax", "Path=/", "Secure"] {
            assert!(value.contains(attribute), "{attribute}");
        }
        let cleared = clear_session_cookie(false);
        assert!(cleared.to_str().unwrap().contains("Max-Age=0"));
        assert!(!cleared.to_str().unwrap().contains("Secure"));
    }

    #[test]
    fn failures_are_limited_per_account_key() {
        let auth = Auth::with_params(Params::default());
        for _ in 0..MAX_FAILURES {
            assert!(auth.allow_attempt("me@example.com"));
            auth.record_failure("me@example.com".into());
        }
        assert!(!auth.allow_attempt("me@example.com"));
        assert!(auth.allow_attempt("other@example.com"));
        auth.clear_failures("me@example.com");
        assert!(auth.allow_attempt("me@example.com"));
    }
}
