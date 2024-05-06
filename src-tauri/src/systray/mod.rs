use log::{info, warn};
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, SystemTraySubmenu,
};

use crate::{
    clip::clip_data::ClipStateMutex,
    config::ConfigMutex,
    event::{event_sender, CopyClipEvent, EventSender},
    log::panic_app,
};

/// create the tray
pub fn create_tray(
    page_len: i64,
    pinned_clips_num: i64,
    favourite_clips_num: i64,
    paused: bool,
) -> SystemTray {
    let tray_menu = create_tray_menu(page_len, pinned_clips_num, favourite_clips_num, paused);

    SystemTray::new().with_menu(tray_menu)
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
#[cfg(not(target_os = "windows"))]
pub fn create_tray_menu(
    page_len: i64,
    pinned_clips_num: i64,
    favourite_clips_num: i64,
    paused: bool,
) -> SystemTrayMenu {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.

    let notice_select =
        CustomMenuItem::new("notice_select".to_string(), t!("tray_menu.notice_select")).disabled();

    let page_info = CustomMenuItem::new("page_info".to_string(), "").disabled(); // Total clips: 0, Current page: 0/0
    let prev_page = CustomMenuItem::new("prev_page".to_string(), t!("tray_menu.prev_page"))
        .accelerator("CommandOrControl+A");
    let next_page = CustomMenuItem::new("next_page".to_string(), t!("tray_menu.next_page"))
        .accelerator("CommandOrControl+D");
    let first_page = CustomMenuItem::new("first_page".to_string(), t!("tray_menu.first_page"));

    let preferences = CustomMenuItem::new("preferences".to_string(), t!("tray_menu.preferences"));
    let search = CustomMenuItem::new("search".to_string(), t!("tray_menu.search"));
    let text = if paused {
        t!("tray_menu.resume_monitoring")
    } else {
        t!("tray_menu.pause_monitoring")
    };
    let pause = CustomMenuItem::new("pause".to_string(), text);

    let quit = CustomMenuItem::new("quit".to_string(), t!("tray_menu.quit"));
    let mut tray_menu = SystemTrayMenu::new()
        .add_item(notice_select)
        .add_native_item(SystemTrayMenuItem::Separator);

    // add the pinned clips slot
    for i in 0..pinned_clips_num {
        let clip = CustomMenuItem::new("pinned_clip_".to_string() + &i.to_string(), "");
        tray_menu = tray_menu.add_item(clip);
    }
    tray_menu = tray_menu.add_native_item(SystemTrayMenuItem::Separator);

    // add the label submenus
    //    -default label: favourites
    let mut favourite_menu = SystemTrayMenu::new();
    for i in 0..favourite_clips_num {
        let clip = CustomMenuItem::new("favourite_clip_".to_string() + &i.to_string(), "");
        favourite_menu = favourite_menu.add_item(clip);
    }
    let favourite = SystemTraySubmenu::new(t!("tray_menu.favourite"), favourite_menu);
    tray_menu = tray_menu.add_submenu(favourite);
    tray_menu = tray_menu.add_native_item(SystemTrayMenuItem::Separator);

    // add the clips slot
    for i in 0..page_len {
        let clip = CustomMenuItem::new("tray_clip_".to_string() + &i.to_string(), "");
        tray_menu = tray_menu.add_item(clip);
    }

    tray_menu
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(page_info)
        .add_item(prev_page)
        .add_item(next_page)
        .add_item(first_page)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(preferences)
        .add_item(search)
        .add_item(pause)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit)
}

#[cfg(target_os = "windows")]
pub fn create_tray_menu(
    page_len: i64,
    pinned_clips_num: i64,
    favourite_clips_num: i64,
    paused: bool,
) -> SystemTrayMenu {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.

    let notice_select =
        CustomMenuItem::new("notice_select".to_string(), t!("tray_menu.notice_select")).disabled();

    let page_info = CustomMenuItem::new("page_info".to_string(), "").disabled(); // Total clips: 0, Current page: 0/0
    let prev_page = CustomMenuItem::new("prev_page".to_string(), t!("tray_menu.prev_page"))
        .accelerator("CommandOrControl+A");
    let next_page = CustomMenuItem::new("next_page".to_string(), t!("tray_menu.next_page"))
        .accelerator("CommandOrControl+D");
    let first_page = CustomMenuItem::new("first_page".to_string(), t!("tray_menu.first_page"));

    let preferences = CustomMenuItem::new("preferences".to_string(), t!("tray_menu.preferences"));
    let search = CustomMenuItem::new("search".to_string(), t!("tray_menu.search"));
    let text = if paused {
        t!("tray_menu.resume_monitoring")
    } else {
        t!("tray_menu.pause_monitoring")
    };
    let pause = CustomMenuItem::new("pause".to_string(), text);

    let quit = CustomMenuItem::new("quit".to_string(), t!("tray_menu.quit"));
    let mut tray_menu = SystemTrayMenu::new();
    tray_menu = tray_menu
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(pause)
        .add_item(search)
        .add_item(preferences)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(first_page)
        .add_item(next_page)
        .add_item(prev_page)
        .add_item(page_info)
        .add_native_item(SystemTrayMenuItem::Separator);

    // add the clips slot
    for i in (0..page_len).rev() {
        let clip = CustomMenuItem::new("tray_clip_".to_string() + &i.to_string(), "");
        tray_menu = tray_menu.add_item(clip);
    }
    tray_menu = tray_menu.add_native_item(SystemTrayMenuItem::Separator);

    // add the label submenus
    //    -default label: favourites
    let mut favourite_menu = SystemTrayMenu::new();
    for i in (0..favourite_clips_num).rev() {
        let clip = CustomMenuItem::new("favourite_clip_".to_string() + &i.to_string(), "");
        favourite_menu = favourite_menu.add_item(clip);
    }
    let favourite = SystemTraySubmenu::new(t!("tray_menu.favourite"), favourite_menu);

    tray_menu = tray_menu
        .add_submenu(favourite)
        .add_native_item(SystemTrayMenuItem::Separator);

    // add the pinned clips slot
    for i in (0..pinned_clips_num).rev() {
        let clip = CustomMenuItem::new("pinned_clip_".to_string() + &i.to_string(), "");
        tray_menu = tray_menu.add_item(clip);
    }
    tray_menu = tray_menu.add_native_item(SystemTrayMenuItem::Separator);

    tray_menu.add_item(notice_select)
}

/// handle the tray event
pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            event_sender(app, CopyClipEvent::TrayMenuItemClickEvent(id));
        }
        _ => {
            // do nothing
        }
    }
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
            std::process::exit(0);
        }
        "next_page" => {
            let clip_data = app.state::<ClipStateMutex>();
            let mut clip_data = clip_data.clip_state.lock().await;
            let res = clip_data.next_page(app).await;
            if let Err(e) = res {
                warn!("Failed to get next page: {}", e);
                return;
            }

            // update the tray
            send_tray_update_event(app);
        }
        "prev_page" => {
            let clip_data = app.state::<ClipStateMutex>();
            let mut clip_data = clip_data.clip_state.lock().await;
            let res = clip_data.prev_page(app).await;
            if let Err(e) = res {
                warn!("Failed to get prev page: {}", e);
                return;
            }

            // update the tray
            send_tray_update_event(app);
        }
        "first_page" => {
            let clip_data = app.state::<ClipStateMutex>();
            let mut clip_data = clip_data.clip_state.lock().await;
            clip_data.first_page().await;

            // update the tray
            send_tray_update_event(app);
        }
        "preferences" => {
            // open the preferences window
            // test if the window is already open
            let windows = app.windows();
            let preferences_window = windows.get("preferences");
            if let Some(preferences_window) = preferences_window {
                let res = preferences_window.show();
                if let Err(e) = res {
                    panic_app(&format!("Failed to show preferences window: {e}"));
                }
            } else {
                let app_handle = app.app_handle();
                std::thread::spawn(move || {
                    let preferences_window = tauri::WindowBuilder::new(
                        &app_handle,
                        "preferences",
                        tauri::WindowUrl::App("preferences".into()),
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
                let app_handle = app.app_handle();
                std::thread::spawn(move || {
                    let preferences_window = tauri::WindowBuilder::new(
                        &app_handle,
                        "search",
                        tauri::WindowUrl::App("search".into()),
                    )
                    .title("Copy Clip")
                    .build();
                    if let Err(e) = preferences_window {
                        panic_app(&format!("Failed to open search window: {e}"));
                    }
                });
            }
        }
        "pause" => {
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

                // get the index of the clip
                let index = id.replace("tray_clip_", "").parse::<i64>().unwrap();

                // select the index
                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;

                let item_id = clip_data.tray_ids_map.get(index as usize);
                if item_id.is_none() {
                    warn!("Failed to get the item id for the tray id: {}", index);
                    return;
                }
                let item_id = *item_id.unwrap();

                let res = clip_data.select_clip(app, Some(item_id)).await;
                if res.is_err() {
                    warn!("Failed to select the clip: {}", res.err().unwrap());
                    return;
                }
            } else if id.starts_with("pinned_clip_") {
                // test if the id is a pinned_clip

                // get the index of the clip
                let index = id.replace("pinned_clip_", "").parse::<i64>().unwrap();

                // select the index
                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;

                let item_id = clip_data.pinned_clips.get(index as usize);
                if item_id.is_none() {
                    warn!(
                        "Failed to get the item id for the pinned clip id: {}",
                        index
                    );
                    return;
                }
                let item_id = item_id.unwrap();
                let id = *item_id;

                let res = clip_data.select_clip(app, Some(id)).await;
                if res.is_err() {
                    warn!("Failed to select the clip: {}", res.err().unwrap());
                    return;
                }
            } else if id.starts_with("favourite_clip_") {
                // test if the id is a favourite_clip

                // get the index of the clip
                let index = id.replace("favourite_clip_", "").parse::<i64>().unwrap();

                // select the index
                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;

                let item_id = clip_data.favourite_clips.get(index as usize);
                if item_id.is_none() {
                    warn!(
                        "Failed to get the item id for the favourite clip id: {}",
                        index
                    );
                    return;
                }
                let item_id = item_id.unwrap();
                let id = *item_id;

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

pub fn send_tray_update_event(app: &AppHandle) {
    event_sender(app, CopyClipEvent::TrayUpdateEvent);
}
