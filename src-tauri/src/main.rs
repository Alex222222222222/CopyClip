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

use app::{
    clip::{self, database::init_database_connection, ClipData, ClipDataMutex},
    config,
    config::{Config, ConfigMutex},
    event::{event_daemon, CopyClipEvent, EventSender},
    systray::{self, create_tray_menu, send_tray_update_event},
};
use tauri::{Manager, SystemTray};

fn main() {
    tauri::Builder::default()
        // .invoke_handler(tauri::generate_handler![on_button_clicked])
        .manage(ConfigMutex {
            config: Mutex::<Config>::default(),
        })
        .manage(ClipDataMutex {
            clip_data: Mutex::<ClipData>::default(),
        })
        .setup(|app| {
            // load the config info from the config file
            let config = config::load_config(app);
            let app_handle = app.handle();
            let config_mutex = app_handle.state::<ConfigMutex>();
            let mut config_mutex = config_mutex.config.lock().unwrap();
            *config_mutex = config;
            drop(config_mutex);

            // set up the database connection and create the table
            let res = init_database_connection(&app_handle);
            if res.is_err() {
                println!("failed to init database connection");
                panic!("{}", res.err().unwrap().message());
            }

            // set up event sender and receiver
            let app_handle = app.handle();
            let (event_tx, event_rx) = std::sync::mpsc::channel::<CopyClipEvent>();
            app.manage(EventSender::new(event_tx));
            // set up the event receiver daemon
            tauri::async_runtime::spawn(async move {
                event_daemon(event_rx, &app_handle);
            });

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                // the daemon to monitor the system clip board change and trigger the tray update
                clip::monitor::monitor_clip_board(&app_handle);
            });

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                clip::cache::cache_daemon(&app_handle);
            });

            // get number of clips to show from config
            let app_handle = app.app_handle();
            let num = app_handle.state::<ConfigMutex>();
            let num = num.config.lock().unwrap().clips_to_show;
            let res = app.tray_handle().set_menu(create_tray_menu(num));
            if res.is_err() {
                println!("failed to set tray menu");
                panic!("{}", res.err().unwrap().to_string());
            }
            // initial the tray
            send_tray_update_event(&app_handle);

            Ok(())
        })
        // tauri setup the system tray before the app.setup
        .system_tray(SystemTray::new())
        .on_system_tray_event(systray::handle_tray_event)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
