mod app;
mod components;
mod demo;
pub mod routes;

pub mod api;
pub mod models;

#[cfg(all(feature = "desktop", not(feature = "server")))]
pub mod platform;

#[cfg(feature = "server")]
pub mod server;

pub use app::App;
