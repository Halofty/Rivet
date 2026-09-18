pub mod auth;
pub mod config;
pub mod db;
mod guard;
pub mod issues;
pub mod projects;
pub mod sessions;
pub mod users;

use auth::Auth;
use config::ServerConfig;
use dioxus::server::axum::{
    Extension, Router,
    extract::DefaultBodyLimit,
    http::{HeaderMap, header},
    middleware,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use users::{Registration, UserRecord};

/// Largest accepted request body; the biggest valid input is a 10,000-character description.
const BODY_LIMIT_BYTES: usize = 256 * 1024;
const PROXY_HEADERS: [&str; 4] = [
    "forwarded",
    "x-forwarded-for",
    "x-forwarded-host",
    "x-real-ip",
];

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Database(#[from] db::DatabaseError),
}

/// Request-scoped access to backend resources, provided as an Axum extension so server
/// functions receive it during both API calls and SSR.
#[derive(Clone)]
pub struct AppState {
    pub application: &'static str,
    pub pool: SqlitePool,
    pub config: Arc<ServerConfig>,
    pub auth: Arc<Auth>,
}

impl AppState {
    pub fn new(pool: SqlitePool, config: ServerConfig, auth: Arc<Auth>) -> Self {
        Self {
            application: "Rivet",
            pool,
            config: Arc::new(config),
            auth,
        }
    }

    /// Reads configuration, opens and migrates the database. Fails before serving on error.
    pub async fn from_env() -> Result<Self, StartupError> {
        let config = ServerConfig::from_env()?;
        let path = db::database_path_from_env()?;
        let pool = db::open(&path).await?;
        Ok(Self::new(pool, config, Auth::new()))
    }

    pub async fn current_user(&self, headers: &HeaderMap) -> sqlx::Result<Option<UserRecord>> {
        Ok(auth::authenticate(&self.pool, headers)
            .await?
            .map(|(record, _)| record))
    }

    /// First-user setup is accepted only for direct local requests, so a freshly exposed
    /// server cannot be claimed through its public address.
    ///
    /// Some reverse proxies rewrite `Host` to the loopback upstream, so a loopback host alone
    /// is not proof of a local request: any proxy forwarding header also closes setup.
    pub fn registration_policy(&self, headers: &HeaderMap) -> Option<Registration> {
        match self.config.registration {
            Registration::Open => Some(Registration::Open),
            Registration::FirstUserOnly => {
                let loopback_host = headers
                    .get(header::HOST)
                    .and_then(|host| host.to_str().ok())
                    .is_some_and(|host| self.config.is_loopback_host(host));
                let proxied = PROXY_HEADERS.iter().any(|name| headers.contains_key(*name));
                (loopback_host && !proxied).then_some(Registration::FirstUserOnly)
            }
        }
    }
}

pub fn router(state: AppState) -> Router {
    with_layers(dioxus::server::router(crate::App), state)
}

/// Shared by the production router and transport tests. The guard is the outermost layer.
pub fn with_layers(router: Router, state: AppState) -> Router {
    let config = state.config.clone();
    router
        .layer(Extension(state))
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES))
        .layer(middleware::from_fn_with_state(config, guard::request_guard))
}
