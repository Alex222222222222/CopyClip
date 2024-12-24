mod command;
mod logging;

use tauri::{
    plugin::{Builder, PluginApi, TauriPlugin},
    AppHandle, Manager, Runtime,
};

pub use logging::LogLevelFilter;

pub fn panic_app(msg: &str) {
    log::error!("{}", msg);
    std::process::exit(1);
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("logging")
        .setup(|app_handle: &AppHandle<R>, _plugin_api: PluginApi<R, ()>| {
            // set up the logger
            match logging::setup_logger(app_handle.path()) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string().into()),
            }
        })
        .invoke_handler(tauri::generate_handler![
            command::trace,
            command::debug,
            command::info,
            command::warn,
            command::error,
        ])
        .build()
}
