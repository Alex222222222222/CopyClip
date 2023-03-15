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

use std::sync::Mutex;

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
            let res = init_database_connection(&app_handle);
            if res.is_err() {
                return Err(res.err().unwrap().into());
            }

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                let mut last_clip = String::new(); // TODO get last clip from database
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
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            });

            let app_handle = app.handle();

            tauri::async_runtime::spawn(async move {
                // monitor the whole list of ids len change
                let mut last_len = 0;

                // also monitor the current clip change
                let mut current_clip = 0;
                loop{
                    let clips = app_handle.state::<ClipDataMutex>();
                    let mut clip_data = clips.clip_data.lock().unwrap();
                    if clip_data.clips.whole_list_of_ids.len() != last_len || clip_data.clips.current_clip != current_clip {
                        last_len = clip_data.clips.whole_list_of_ids.len();
                        current_clip = clip_data.clips.current_clip;
                        let res = clip_data.update_tray(&app_handle);
                        if res.is_err() {
                            // TODO log error
                            println!("error: {}", res.err().unwrap());
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1000));
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
