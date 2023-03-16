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

// TODO add way to change the theme and the icon

use std::sync::Mutex;

use app::{systray::{self, create_tray}, config, config::{ConfigMutex,Config}, clip::{ClipDataMutex, ClipData, database::init_database_connection, self}};
use tauri::Manager;

fn main() {
    let num = Config::default().clips_to_show;

    tauri::Builder::default()
        // .invoke_handler(tauri::generate_handler![on_button_clicked])
        .manage(ConfigMutex{config:Mutex::<Config>::default()})
        .manage(ClipDataMutex{clip_data:Mutex::<ClipData>::default()})
        .setup(|app| {
            // load the config info from the config file
            let config = config::load_config(app);
            let app_handle = app.handle();
            let config_mutex = app_handle.state::<ConfigMutex>();
            let mut config_mutex = config_mutex.config.lock().unwrap();
            *config_mutex = config;

            // set up the database connection and create the table
            let res = init_database_connection(&app_handle);
            if res.is_err() {
                return Err(res.err().unwrap().into());
            }

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                // the daemon to monitor the system clip board change                
                clip::monitor::monitor(&app_handle);
            });

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                // the daemon to monitor the app clips data change and trigger the tray update
                clip::monitor::clips_data_monitor(&app_handle);
            });

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                clip::cache::cache_daemon(&app_handle);
            });

            Ok(())
        })
        .system_tray(create_tray(num))
        .on_system_tray_event(systray::handle_tray_event)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
