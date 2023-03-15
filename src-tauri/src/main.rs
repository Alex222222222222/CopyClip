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

// TODO add way to monitor system key board

use std::{sync::Mutex, thread};

use app::{systray::{self, create_tray}, config, config::{ConfigMutex,Config}, clip::{ClipDataMutex, ClipData, init_database_connection, self}};
use tauri::{Manager, ClipboardManager};

fn main() {
    let num = 10;

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
            let app_handle = app.handle();
            let res = init_database_connection(&app_handle);
            if res.is_err() {
                return Err(res.err().unwrap().into());
            }

            // set up the database connection and create the table
            let app_handle = app.handle();
            
            tauri::async_runtime::spawn(async move {
                let mut last_clip = String::new();
                loop {
                    let clipboard_manager = app_handle.clipboard_manager();
                    let clip = clipboard_manager.read_text();
                    if clip.is_err() {
                        continue;
                    }
                    let clip = clip.unwrap();
                    if clip.is_none() {
                        continue;
                    }
                    let clip = clip.unwrap();
                    if clip == last_clip {
                        continue;
                    }
                    last_clip = clip.clone();
                    let clips = app_handle.state::<ClipDataMutex>();
                    let mut clip_data = clips.clip_data.lock().unwrap();
                    let res = clip_data.new_clip(clip);
                    if res.is_err() {
                        // TODO log error
                        println!("error: {}", res.err().unwrap());
                    }
                }
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
