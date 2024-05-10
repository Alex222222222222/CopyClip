#[cfg(feature = "tauri")]
mod tauri;

#[cfg(feature = "tauri")]
pub use tauri::init;

#[cfg(feature = "tauri")]
pub use tauri::LogLevelFilter;

#[cfg(feature = "tauri")]
pub use tauri::panic_app;

#[cfg(feature = "api")]
mod api;

#[cfg(feature = "api")]
pub use api::debug;
#[cfg(feature = "api")]
pub use api::error;
#[cfg(feature = "api")]
pub use api::info;
#[cfg(feature = "api")]
pub use api::trace;
#[cfg(feature = "api")]
pub use api::warn;
