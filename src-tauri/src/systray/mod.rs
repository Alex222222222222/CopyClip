mod system_tray_menu;

pub use system_tray_menu::SystemTrayMenuMutex;

use log::{debug, error, info, warn};
use tauri::{async_runtime::Mutex, AppHandle, Manager};
use tauri_plugin_logging::panic_app;

use tauri::menu::MenuItemBuilder;

use crate::{
    clip_frontend::clip_data::{ClipState, ClipStateMutex},
    config::ConfigMutex,
    event::{event_sender, CopyClipEvent, EventSender},
};



/// create the tray menu
/// the menu is created with the given number of clips
/// the menu is created with the following items:
/// - notice select
/// - pinned clips slot
/// - clips slot
/// - page info
/// - prev page
/// - next page
/// - first page
/// - preferences
/// - search
/// - quit
#[cfg(not(target_os = "windows"))]
pub fn create_tray_menu(
    page_len: i64,
    pinned_clips_num: i64,
    favourite_clips_num: i64,
    paused: bool,
    app: &AppHandle,
) -> anyhow::Result<tauri::menu::Menu<tauri::Wry>> {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.

    let notice_select =
        MenuItemBuilder::with_id("notice_select".to_string(), t!("tray_menu.notice_select"))
            .enabled(false)
            .build(app)?;

    let page_info = MenuItemBuilder::with_id("page_info".to_string(), "")
        .enabled(false)
        .build(app)?; // Total clips: 0, Current page: 0/0
    let prev_page = MenuItemBuilder::with_id("prev_page".to_string(), t!("tray_menu.prev_page"))
        .accelerator("CommandOrControl+A")
        .build(app)?;
    let next_page = MenuItemBuilder::with_id("next_page".to_string(), t!("tray_menu.next_page"))
        .accelerator("CommandOrControl+D")
        .build(app)?;
    let first_page = MenuItemBuilder::with_id("first_page".to_string(), t!("tray_menu.first_page"))
        .build(app)?;

    let preferences =
        MenuItemBuilder::with_id("preferences".to_string(), t!("tray_menu.preferences"))
            .build(app)?;
    let search =
        MenuItemBuilder::with_id("search".to_string(), t!("tray_menu.search")).build(app)?;
    let text = if paused {
        t!("tray_menu.resume_monitoring")
    } else {
        t!("tray_menu.pause_monitoring")
    };
    let pause = MenuItemBuilder::with_id("pause".to_string(), text).build(app)?;

    let quit = MenuItemBuilder::with_id("quit".to_string(), t!("tray_menu.quit")).build(app)?;

    let mut tray_menu = tauri::menu::MenuBuilder::new(app)
        .item(&notice_select)
        .separator();

    // add the pinned clips slot
    for i in 0..pinned_clips_num {
        let clip =
            MenuItemBuilder::with_id("pinned_clip_".to_string() + &i.to_string(), "").build(app)?;
        tray_menu = tray_menu.item(&clip);
    }
    tray_menu = tray_menu.separator();

    // add the label submenus
    //    -default label: favourites
    let mut favourite = tauri::menu::SubmenuBuilder::new(app, t!("tray_menu.favourite"));
    for i in 0..favourite_clips_num {
        let clip = MenuItemBuilder::with_id("favourite_clip_".to_string() + &i.to_string(), "")
            .build(app)?;
        favourite = favourite.item(&clip);
    }
    let favourite = favourite.build()?;
    tray_menu = tray_menu.item(&favourite).separator();

    // add the clips slot
    for i in 0..page_len {
        let clip =
            MenuItemBuilder::with_id("tray_clip_".to_string() + &i.to_string(), "").build(app)?;
        tray_menu = tray_menu.item(&clip);
    }

    Ok(tray_menu
        .separator()
        .items(&[&page_info, &prev_page, &next_page, &first_page])
        .separator()
        .items(&[&preferences, &search, &pause])
        .separator()
        .item(&quit)
        .build()?)
}

/// handle the tray event
pub fn handle_tray_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    event_sender(app, CopyClipEvent::TrayMenuItemClickEvent(id.to_string()));
}

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
            if id.starts_with("tray_clip_") {
                // test if the id is a tray_clip
                debug!("Tray clip clicked: {}", id);

                // get the index of the clip
                let index = id.replace("tray_clip_", "").parse::<u64>().unwrap();

                // select the index
                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;

                // try calculate the pos of the clip using current page and page_len
                // and total number of clips
                let item_id = match clip_data
                    .get_id_with_pos_in_current_page(app, Some(index))
                    .await
                {
                    Ok(res) => match res {
                        Some(res) => res,
                        None => {
                            error!("Failed to get the item id for the tray id: {}", index);
                            return;
                        }
                    },
                    Err(_) => {
                        error!("Failed to get the item id for the tray id: {}", index);
                        return;
                    }
                };

                let res = clip_data.select_clip(app, Some(item_id)).await;
                if res.is_err() {
                    warn!("Failed to select the clip: {}", res.err().unwrap());
                    return;
                }
            } else if id.starts_with("pinned_clip_") {
                // test if the id is a pinned_clip

                // get the index of the clip
                let index = id.replace("pinned_clip_", "").parse::<u64>().unwrap();

                // select the index
                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;

                let item_id =
                    match ClipState::get_label_clip_id_with_pos(app, "pinned", index).await {
                        Ok(res) => match res {
                            Some(res) => res,
                            None => {
                                error!(
                                    "Failed to get the item id for the pinned clip id: {}",
                                    index
                                );
                                return;
                            }
                        },
                        Err(_) => {
                            error!(
                                "Failed to get the item id for the pinned clip id: {}",
                                index
                            );
                            return;
                        }
                    };

                let res = clip_data.select_clip(app, Some(item_id)).await;
                if res.is_err() {
                    warn!("Failed to select the clip: {}", res.err().unwrap());
                    return;
                }
            } else if id.starts_with("favourite_clip_") {
                // test if the id is a favourite_clip

                // get the index of the clip
                let index = id.replace("favourite_clip_", "").parse::<u64>().unwrap();

                // select the index
                let item_id =
                    match ClipState::get_label_clip_id_with_pos(app, "favourite", index).await {
                        Ok(res) => match res {
                            Some(res) => res,
                            None => {
                                error!(
                                    "Failed to get the item id for the favourite clip id: {}",
                                    index
                                );
                                return;
                            }
                        },
                        Err(_) => {
                            error!(
                                "Failed to get the item id for the favourite clip id: {}",
                                index
                            );
                            return;
                        }
                    };

                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;
                let res = clip_data.select_clip(app, Some(item_id)).await;
                drop(clip_data);

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
