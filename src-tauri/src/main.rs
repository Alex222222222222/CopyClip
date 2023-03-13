#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

/*
#[tauri::command]
fn on_button_clicked() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    format!("on_button_clicked called from Rust! (timestamp: {since_the_epoch}ms)")
}
*/

use std::{ sync::Mutex};

use app::{systray, config, config::{ConfigMutex,Config}};
use tauri::Manager;

fn main() {
    let tray = systray::create_tray();

    tauri::Builder::default()
        .manage(ConfigMutex{config:Mutex::<Config>::default()})
        // .invoke_handler(tauri::generate_handler![on_button_clicked])
        .system_tray(tray)
        .on_system_tray_event(systray::handle_tray_event)
        .setup(|app| {
            // TODO load the config info from the config file
            let config = config::load_config(app);
            let app_handle = app.handle();
            let config_mutex = app_handle.state::<ConfigMutex>();
            let mut config_mutex = config_mutex.config.lock().unwrap();
            *config_mutex = config;
            Ok(())
        })
        // TODO create the sqlite3 connection to the database.
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
