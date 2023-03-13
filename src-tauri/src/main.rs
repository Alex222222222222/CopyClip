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

use std::sync::Mutex;

use app::{systray, config, config::{ConfigMutex,Config}, clip::{ClipDataMutex, ClipData, init_database_connection}};
use tauri::Manager;

fn main() {
    let tray = systray::create_tray();

    tauri::Builder::default()
        // .invoke_handler(tauri::generate_handler![on_button_clicked])
        .system_tray(tray)
        .on_system_tray_event(systray::handle_tray_event)
        .manage(ConfigMutex{config:Mutex::<Config>::default()})
        .setup(|app| {
            // load the config info from the config file
            let config = config::load_config(app);
            let app_handle = app.handle();
            let config_mutex = app_handle.state::<ConfigMutex>();
            let mut config_mutex = config_mutex.config.lock().unwrap();
            *config_mutex = config;
            Ok(())
        })
        .manage(ClipDataMutex{clip_data:Mutex::<ClipData>::default()})
        .setup(|app| {
            // set up the database connection and create the table
            let app_handle = app.handle();
            let res = init_database_connection(&app_handle);
            if res.is_err() {
                return Err(res.err().unwrap().into());
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
