// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use copy_clip::{
    clip::{self, clip_data::ClipStateMutex},
    config::{self, Config, ConfigMutex},
    database::{init_database_connection, DatabaseStateMutex},
    event::{event_daemon, event_sender, CopyClipEvent, EventSender},
    export,
    systray::handle_tray_event,
};
use log::{error, info};
use rust_i18n::set_locale;
use tauri::{async_runtime::Mutex, Manager, SystemTray};
use tauri_plugin_logging::panic_app;

const EVENT_CHANNEL_SIZE: usize = 1000;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_logging::init())
        // .invoke_handler(tauri::generate_handler![on_button_clicked])
        .manage(ConfigMutex {
            config: Mutex::<Config>::default(),
        })
        .manage(ClipStateMutex::default())
        .manage(DatabaseStateMutex::default())
        .setup(|app| {
            // set up the database connection and create the table
            // will also init the clip data state
            let app_handle = app.handle();
            let connection = match init_database_connection(&app_handle) {
                Ok(connection) => connection,
                Err(err) => {
                    error!("failed to init database connection");
                    return Err(err.to_string().into());
                }
            };
            // tx and rx is used to wait until the prepare finished
            let (tx, rx) = std::sync::mpsc::channel::<()>();
            tauri::async_runtime::spawn(async move {
                let database_mutex = app_handle.state::<DatabaseStateMutex>();
                let mut database_mutex = database_mutex.database_connection.lock().await;
                *database_mutex = connection;
                tx.send(()).unwrap();
            });
            rx.recv().unwrap();

            // load the config info from the config file
            let app_handle = app.handle();
            // tx and rx is used to wait until the prepare finished
            let (tx, rx) = std::sync::mpsc::channel::<()>();
            tauri::async_runtime::spawn(async move {
                let config_mutex = app_handle.state::<ConfigMutex>();
                let mut config_mutex = config_mutex.config.lock().await;
                config_mutex.load_config(&app_handle);
                drop(config_mutex);

                tx.send(()).unwrap();
            });
            rx.recv().unwrap();

            // set the i18n
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                let config_mutex = app_handle.state::<ConfigMutex>();
                let config_mutex = config_mutex.config.lock().await;
                let language = config_mutex.language.clone();
                drop(config_mutex);
                set_locale(&language);
            });

            // set up event sender and receiver
            let app_handle = app.handle();
            let (event_tx, event_rx) =
                tauri::async_runtime::channel::<CopyClipEvent>(EVENT_CHANNEL_SIZE);
            app.manage(EventSender::new(event_tx));
            // set up the event receiver daemon
            tauri::async_runtime::spawn(async move {
                event_daemon(event_rx, &app_handle).await;
            });

            // set up the clip board monitor daemon
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                // the daemon to monitor the system clip board change and trigger the tray update
                clip::monitor::monitor_clip_board(&app_handle).await;
            });

            // initial the tray
            let app_handle = &app.handle();
            event_sender(app_handle, CopyClipEvent::RebuildTrayMenuEvent);

            info!("app setup finished");

            Ok(())
        })
        // tauri setup the system tray before the app.setup
        .system_tray(SystemTray::new())
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            config::command::get_per_page_data,
            config::command::set_per_page_data,
            config::command::get_max_clip_len,
            config::command::set_max_clip_len,
            config::command::get_log_level_filter,
            config::command::set_log_level_filter,
            config::command::get_dark_mode,
            config::command::set_dark_mode,
            config::command::get_search_clip_per_batch,
            config::command::set_search_clip_per_batch,
            config::command::get_language,
            config::command::set_language,
            config::command::get_auto_delete_duplicate_clip,
            config::command::set_auto_delete_duplicate_clip,
            export::export_data_invoke,
            clip::switch_pinned_status,
            clip::copy_clip_to_clipboard,
            clip::delete_clip_from_database,
            clip::change_favourite_clip,
            clip::search::search_clips,
            clip::search::get_max_id,
            clip::id_is_pinned,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let windows = app_handle.windows();
                for (_, window) in windows {
                    let res = window.close();
                    panic_app(&format!(
                        "failed to close window {err}",
                        err = res.unwrap_err()
                    ));
                }
                api.prevent_exit();
            }
        });
}
