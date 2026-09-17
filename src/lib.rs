mod app;

pub mod api;

#[cfg(all(feature = "desktop", not(feature = "server")))]
pub mod platform;

#[cfg(feature = "server")]
pub mod server;

pub use app::App;
