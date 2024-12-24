/*
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
*/

pub mod backward;
pub mod clip_frontend;
pub mod config;
pub mod database;
pub mod error;
pub mod event;
pub mod export;
pub mod systray;

#[macro_use]
extern crate rust_i18n;
i18n!("../locales");

use crate::{
    clip_frontend::clip_data::ClipStateMutex,
    config::{Config, ConfigMutex},
    database::{init_database_connection, DatabaseStateMutex},
    event::{event_daemon, event_sender, CopyClipEvent, EventSender},
};
use log::{error, info};
use rust_i18n::set_locale;
use systray::SystemTrayMenuMutex;
use tauri::{async_runtime::Mutex, path::BaseDirectory, Manager};
use tauri_plugin_logging::panic_app;

const EVENT_CHANNEL_SIZE: usize = 1000;

/// Get the translation for the given key.
#[tauri::command]
async fn i18n_t(key: String) -> Result<String, String> {
    let res = t!(key);
    Ok(res.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_logging::init())
        // .invoke_handler(tauri::generate_handler![on_button_clicked])
        .manage(ConfigMutex {
            config: Mutex::<Config>::default(),
        })
        .manage(ClipStateMutex::default())
        .manage(DatabaseStateMutex::default())
        .manage(SystemTrayMenuMutex::default())
        .setup(|app| {
            // init the clip search to enable the search feature
            let detection_model_path = match app
                .path()
                .resolve("../ocr_models/text-detection.rten", BaseDirectory::Resource)
            {
                Ok(path) => path,
                Err(err) => {
                    error!("failed to resolve text-detection.rten: {err}", err = err);
                    return Err(err.into());
                }
            };
            let recognition_model_path = match app.path().resolve(
                "../ocr_models/text-recognition.rten",
                BaseDirectory::Resource,
            ) {
                Ok(path) => path,
                Err(err) => {
                    error!("failed to resolve text-recognition.rten: {err}", err = err);
                    return Err(err.into());
                }
            };
            match clip::init_search(detection_model_path, recognition_model_path) {
                Ok(_) => (),
                Err(err) => {
                    error!("failed to init search: {err}", err = err);
                    return Err(err.into());
                }
            }

            // set up the database connection and create the table
            // will also init the clip data state
            let app_handle = app.handle().clone();
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
            let app_handle = app.handle().clone();
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
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let config_mutex = app_handle.state::<ConfigMutex>();
                let config_mutex = config_mutex.config.lock().await;
                let language = config_mutex.language.clone();
                drop(config_mutex);
                set_locale(&language);
            });

            // set up event sender and receiver
            let app_handle = app.handle().clone();
            let (event_tx, event_rx) =
                tauri::async_runtime::channel::<CopyClipEvent>(EVENT_CHANNEL_SIZE);
            app.manage(EventSender::new(event_tx));
            // set up the event receiver daemon
            tauri::async_runtime::spawn(async move {
                event_daemon::<tauri::Wry>(event_rx, &app_handle).await;
            });

            // set up the clip board monitor daemon
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // the daemon to monitor the system clip board change and trigger the tray update
                crate::clip_frontend::monitor::monitor_clip_board(&app_handle).await;
            });

            // init the system tray
            let app_handle = app.handle().clone();
            let icon_resource = app_handle
                .path()
                .resolve("../public/icons/icon.ico", BaseDirectory::Resource)
                .unwrap();
            let tray_icon = tauri::tray::TrayIconBuilder::new()
                .icon(tauri::image::Image::from_path(icon_resource).unwrap())
                .icon_as_template(true)
                .menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    let id = event.id().as_ref();
                    event_sender(app, CopyClipEvent::TrayMenuItemClickEvent(id.to_string()));
                })
                .build(&app_handle)
                .unwrap();
            tauri::async_runtime::spawn(async move {
                // set the tray icon
                let menu = app_handle.state::<SystemTrayMenuMutex>();
                menu.set_tray_icon(tray_icon).await.unwrap();
                menu.init_once_cell(&app_handle).await.unwrap();

                // update the tray icon
                event_sender(&app_handle, CopyClipEvent::RebuildTrayMenuEvent);
            });

            info!("app setup finished");

            Ok(())
        })
        // tauri setup the system tray before the app.setup
        // TODO check if the system tray is set up correctly
        // .system_tray(SystemTray::new())
        // .on_system_tray_event(handle_tray_event)
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
            crate::clip_frontend::switch_pinned_status,
            crate::clip_frontend::copy_clip_to_clipboard,
            crate::clip_frontend::delete_clip_from_database,
            crate::clip_frontend::change_favourite_clip,
            crate::clip_frontend::id_is_pinned,
            crate::clip_frontend::get_all_labels,
            database::search_clips,
            i18n_t,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let windows = app_handle.webview_windows();
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
