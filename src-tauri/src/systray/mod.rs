mod system_tray_menu;

pub use system_tray_menu::SystemTrayMenuMutex;

use log::{debug, error, info, warn};
use tauri::{AppHandle, Manager};
use tauri_plugin_logging::panic_app;

use crate::{
    clip_frontend::clip_data::ClipStateMutex,
    config::ConfigMutex,
    event::{CopyClipEvent, EventSender},
};

/// handle the menu item click
/// this function is called when the user clicks on a menu item
/// the id is the id of the menu item
///
/// the id can be:
/// - quit
/// - next_page
/// - prev_page
/// - first_page
/// - tray_clip_num
pub async fn handle_menu_item_click(app: &AppHandle, id: String) {
    match id.as_str() {
        "quit" => {
            // quit the app
            debug!("Quitting the app");
            std::process::exit(0);
        }
        "next_page" => {
            debug!("Next page clicked");
            let clip_data = app.state::<ClipStateMutex>();
            let mut clip_data = clip_data.clip_state.lock().await;
            let res = clip_data.next_page(app).await;
            if let Err(e) = res {
                warn!("Failed to get next page: {}", e);
                return;
            }

            // update the tray
            let event_sender = app.state::<EventSender>();
            let res = event_sender
                .tx
                .send(CopyClipEvent::RebuildTrayMenuEvent)
                .await;
            if let Err(err) = res {
                error!("Failed to send event, error: {}", err);
            }
        }
        "prev_page" => {
            debug!("Prev page clicked");
            let clip_data = app.state::<ClipStateMutex>();
            let mut clip_data = clip_data.clip_state.lock().await;
            let res = clip_data.prev_page(app).await;
            if let Err(e) = res {
                warn!("Failed to get prev page: {}", e);
                return;
            }

            // update the tray
            let event_sender = app.state::<EventSender>();
            let res = event_sender
                .tx
                .send(CopyClipEvent::RebuildTrayMenuEvent)
                .await;
            if let Err(err) = res {
                error!("Failed to send event, error: {}", err);
            }
        }
        "first_page" => {
            debug!("First page clicked");
            let clip_data = app.state::<ClipStateMutex>();
            let mut clip_data = clip_data.clip_state.lock().await;
            clip_data.first_page().await;

            // update the tray
            let event_sender = app.state::<EventSender>();
            let res = event_sender
                .tx
                .send(CopyClipEvent::RebuildTrayMenuEvent)
                .await;
            if let Err(err) = res {
                error!("Failed to send event, error: {}", err);
            }
        }
        "preferences" => {
            debug!("Preferences clicked, Opening preferences window");
            // open the preferences window
            // test if the window is already open
            let windows = app.webview_windows();
            let preferences_window = windows.get("preferences");
            if let Some(preferences_window) = preferences_window {
                let res = preferences_window.show();
                if let Err(e) = res {
                    panic_app(&format!("Failed to show preferences window: {e}"));
                }
            } else {
                let app_handle = app.app_handle().clone();
                std::thread::spawn(move || {
                    // TODO check error
                    let preferences_window =
                        tauri::window::WindowBuilder::new(&app_handle, "preferences")
                            .title("Copy Clip")
                            .focused(true)
                            .build()
                            .unwrap();
                    let preferences_webview = tauri::WebviewBuilder::new(
                        "preferences",
                        tauri::WebviewUrl::App("preferences".into()),
                    );
                    let webview = preferences_window.add_child(
                        preferences_webview,
                        tauri::LogicalPosition::new(0, 0),
                        preferences_window.inner_size().unwrap(),
                    );
                    // .title("Copy Clip")
                    if let Err(e) = webview {
                        panic_app(&format!("Failed to open preferences window: {e}"));
                    }
                });
            }
        }
        "search" => {
            debug!("Search clicked, Opening search window");
            // open the preferences window
            // test if the window is already open
            let windows = app.windows();
            let preferences_window = windows.get("search");
            if let Some(preferences_window) = preferences_window {
                let res = preferences_window.show();
                if let Err(e) = res {
                    panic_app(&format!("Failed to show search window: {e}"));
                }
            } else {
                let app_handle = app.app_handle().clone();
                std::thread::spawn(move || {
                    // TODO check error
                    // TODO fix open window size and resizeable
                    let search_window = tauri::WindowBuilder::new(&app_handle, "search")
                        .title("Copy Clip")
                        .focused(true)
                        .build()
                        .unwrap();
                    let seach_webview = tauri::WebviewBuilder::new(
                        "search",
                        tauri::WebviewUrl::App("search".into()),
                    );
                    let webview = search_window.add_child(
                        seach_webview,
                        tauri::LogicalPosition::new(0, 0),
                        search_window.inner_size().unwrap(),
                    );
                    if let Err(e) = webview {
                        panic_app(&format!("Failed to open search window: {e}"));
                    }
                });
            }
        }
        "pause" => {
            debug!("Pause clicked, Toggling pause monitoring");
            let config = app.state::<ConfigMutex>();
            let mut config = config.config.lock().await;
            config.pause_monitoring = !config.pause_monitoring;
            drop(config);
            let event_sender = app.state::<EventSender>();
            let res = event_sender
                .tx
                .send(CopyClipEvent::RebuildTrayMenuEvent)
                .await;
            if let Err(err) = res {
                warn!("Failed to send event, error: {}", err);
            }
        }
        _ => {
            if id.starts_with("clip_") {
                // test if the id is a tray_clip
                debug!("Tray clip clicked: {}", id);

                // get the index of the clip
                let id = id.replace("clip_", "").parse::<u64>().unwrap();

                // select the index
                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;

                let res = clip_data.select_clip(app, Some(id)).await;
                if res.is_err() {
                    warn!("Failed to select the clip: {}", res.err().unwrap());
                    return;
                }
            } else {
                warn!("Unknown menu item id: {}", id);
            }

            info!("Menu item clicked: {}", id)
        }
    }
}
