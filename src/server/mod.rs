use dioxus::server::axum::{Extension, Router};

/// Request-scoped access to backend resources. The database pool arrives in milestone 2.
#[derive(Clone)]
pub struct AppState {
    pub application: &'static str,
}

pub fn router() -> Router {
    dioxus::server::router(crate::App).layer(Extension(AppState {
        application: "Rivet",
    }))
}
