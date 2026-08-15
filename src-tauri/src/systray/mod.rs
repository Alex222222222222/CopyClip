use log::{debug, error, info, warn};
use tauri::{
    menu::{Menu, MenuEvent, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu},
    AppHandle, Manager, Wry,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_logging::panic_app;

use crate::{
    clip::clip_data::ClipStateMutex,
    config::ConfigMutex,
    event::{event_sender, CopyClipEvent, EventSender},
};

#[derive(Default)]
pub struct TrayMenuState {
    pub menu: std::sync::Mutex<Option<Menu<Wry>>>,
}

fn append_item(
    menu: &Menu<Wry>,
    app: &AppHandle,
    id: impl Into<tauri::menu::MenuId>,
    text: impl AsRef<str>,
    enabled: bool,
    accelerator: Option<&str>,
) -> tauri::Result<()> {
    menu.append(&MenuItem::with_id(app, id, text, enabled, accelerator)?)
}

fn append_separator(menu: &Menu<Wry>, app: &AppHandle) -> tauri::Result<()> {
    menu.append(&PredefinedMenuItem::separator(app)?)
}

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
pub fn create_tray_menu(
    app: &AppHandle,
    page_len: i64,
    pinned_clips_num: i64,
    favourite_clips_num: i64,
    paused: bool,
) -> tauri::Result<Menu<Wry>> {
    let tray_menu = Menu::new(app)?;
    let favourite_menu = Submenu::with_id(app, "favourite", t!("tray_menu.favourite"), true)?;

    #[cfg(not(target_os = "windows"))]
    {
        append_item(
            &tray_menu,
            app,
            "notice_select",
            t!("tray_menu.notice_select"),
            false,
            None,
        )?;
        append_separator(&tray_menu, app)?;
        for i in 0..pinned_clips_num {
            append_item(&tray_menu, app, format!("pinned_clip_{i}"), "", true, None)?;
        }
        append_separator(&tray_menu, app)?;
        for i in 0..favourite_clips_num {
            favourite_menu.append(&MenuItem::with_id(
                app,
                format!("favourite_clip_{i}"),
                "",
                true,
                None::<&str>,
            )?)?;
        }
        tray_menu.append(&favourite_menu)?;
        append_separator(&tray_menu, app)?;
        for i in 0..page_len {
            append_item(&tray_menu, app, format!("tray_clip_{i}"), "", true, None)?;
        }
    }

    append_separator(&tray_menu, app)?;
    append_item(&tray_menu, app, "page_info", "", false, None)?;
    append_item(
        &tray_menu,
        app,
        "prev_page",
        t!("tray_menu.prev_page"),
        true,
        Some("CommandOrControl+A"),
    )?;
    append_item(
        &tray_menu,
        app,
        "next_page",
        t!("tray_menu.next_page"),
        true,
        Some("CommandOrControl+D"),
    )?;
    append_item(
        &tray_menu,
        app,
        "first_page",
        t!("tray_menu.first_page"),
        true,
        None,
    )?;
    append_separator(&tray_menu, app)?;
    append_item(
        &tray_menu,
        app,
        "preferences",
        t!("tray_menu.preferences"),
        true,
        None,
    )?;
    append_item(
        &tray_menu,
        app,
        "search",
        t!("tray_menu.search"),
        true,
        None,
    )?;
    append_item(
        &tray_menu,
        app,
        "pause",
        if paused {
            t!("tray_menu.resume_monitoring")
        } else {
            t!("tray_menu.pause_monitoring")
        },
        true,
        None,
    )?;
    append_item(
        &tray_menu,
        app,
        "clear_history",
        t!("tray_menu.clear_history"),
        true,
        None,
    )?;
    append_separator(&tray_menu, app)?;
    append_item(&tray_menu, app, "quit", t!("tray_menu.quit"), true, None)?;

    #[cfg(target_os = "windows")]
    {
        for i in (0..page_len).rev() {
            append_item(&tray_menu, app, format!("tray_clip_{i}"), "", true, None)?;
        }
        append_separator(&tray_menu, app)?;
        for i in (0..favourite_clips_num).rev() {
            favourite_menu.append(&MenuItem::with_id(
                app,
                format!("favourite_clip_{i}"),
                "",
                true,
                None::<&str>,
            )?)?;
        }
        tray_menu.append(&favourite_menu)?;
        append_separator(&tray_menu, app)?;
        for i in (0..pinned_clips_num).rev() {
            append_item(&tray_menu, app, format!("pinned_clip_{i}"), "", true, None)?;
        }
        append_separator(&tray_menu, app)?;
        append_item(
            &tray_menu,
            app,
            "notice_select",
            t!("tray_menu.notice_select"),
            false,
            None,
        )?;
    }

    Ok(tray_menu)
}

/// handle the tray event
pub fn handle_tray_event(app: &AppHandle, event: MenuEvent) {
    event_sender(
        app,
        CopyClipEvent::TrayMenuItemClickEvent(event.id().as_ref().to_string()),
    );
}

pub fn tray_menu_item(app: &AppHandle, id: &str) -> Option<MenuItemKind<Wry>> {
    app.state::<TrayMenuState>()
        .menu
        .lock()
        .ok()
        .and_then(|menu| menu.as_ref().and_then(|menu| menu.get(id)))
}

pub fn set_tray_menu_item_text(
    app: &AppHandle,
    id: &str,
    text: impl AsRef<str>,
) -> Result<(), String> {
    let item = tray_menu_item(app, id).ok_or_else(|| format!("tray menu item `{id}` not found"))?;
    let item = item
        .as_menuitem()
        .ok_or_else(|| format!("tray menu item `{id}` is not a text item"))?;
    item.set_text(text).map_err(|error| error.to_string())
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
            let preferences_window = app.get_webview_window("preferences");
            if let Some(preferences_window) = preferences_window {
                let res = preferences_window.show();
                if let Err(e) = res {
                    panic_app(&format!("Failed to show preferences window: {e}"));
                }
            } else {
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    let preferences_window = tauri::WebviewWindowBuilder::new(
                        &app_handle,
                        "preferences",
                        tauri::WebviewUrl::App("preferences".into()),
                    )
                    .title("Copy Clip")
                    .build();
                    if let Err(e) = preferences_window {
                        panic_app(&format!("Failed to open preferences window: {e}"));
                    }
                });
            }
        }
        "search" => {
            debug!("Search clicked, Opening search window");
            // open the preferences window
            // test if the window is already open
            let preferences_window = app.get_webview_window("search");
            if let Some(preferences_window) = preferences_window {
                let res = preferences_window.show();
                if let Err(e) = res {
                    panic_app(&format!("Failed to show search window: {e}"));
                }
            } else {
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    let preferences_window = tauri::WebviewWindowBuilder::new(
                        &app_handle,
                        "search",
                        tauri::WebviewUrl::App("search".into()),
                    )
                    .title("Copy Clip")
                    .build();
                    if let Err(e) = preferences_window {
                        panic_app(&format!("Failed to open search window: {e}"));
                    }
                });
            }
        }
        "clear_history" => {
            debug!("Clear history clicked, asking for confirmation");
            let app_handle = app.clone();
            app.dialog()
                .message(t!("tray_menu.clear_history_confirm"))
                .title(t!("tray_menu.clear_history"))
                .buttons(MessageDialogButtons::YesNo)
                .show(move |confirmed| {
                    if !confirmed {
                        return;
                    }
                    tauri::async_runtime::spawn(async move {
                        let clip_data = app_handle.state::<ClipStateMutex>();
                        let mut clip_data = clip_data.clip_state.lock().await;
                        let res = clip_data.clear_clips(&app_handle).await;
                        if let Err(e) = res {
                            warn!("Failed to clear history: {}", e);
                        }
                    });
                });
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

                let item_id = match clip_data
                    .get_label_clip_id_with_pos(app, "pinned", index)
                    .await
                {
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
                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;

                let item_id = match clip_data
                    .get_label_clip_id_with_pos(app, "favourite", index)
                    .await
                {
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

                let res = clip_data.select_clip(app, Some(item_id)).await;
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
