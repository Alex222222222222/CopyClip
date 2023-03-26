use log::{info, warn};
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

use crate::{clip::ClipDataMutex, event::EventSender, log::panic_app};

/// create the tray
pub fn create_tray(num: i64) -> SystemTray {
    // TODO set icon for tray

    let tray_menu = create_tray_menu(num);

    SystemTray::new().with_menu(tray_menu)
}

pub fn create_tray_menu(num: i64) -> SystemTrayMenu {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let notice_select = CustomMenuItem::new(
        "notice_select".to_string(),
        "Select the clip you want to add to your clipboard.",
    )
    .disabled();

    let page_info = CustomMenuItem::new("page_info".to_string(), "").disabled(); // Total clips: 0, Current page: 0/0
    let prev_page = CustomMenuItem::new("prev_page".to_string(), "Previous page")
        .accelerator("CommandOrControl+A");
    let next_page =
        CustomMenuItem::new("next_page".to_string(), "Next page").accelerator("CommandOrControl+D");
    let first_page = CustomMenuItem::new("first_page".to_string(), "First page");

    let preferences = CustomMenuItem::new("preferences".to_string(), "Preferences");
    let search = CustomMenuItem::new("search".to_string(), "Search");

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let mut tray_menu = SystemTrayMenu::new()
        .add_item(notice_select)
        .add_native_item(SystemTrayMenuItem::Separator);

    for i in 0..num {
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
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit)
}

/// handle the tray event
pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
            SystemTrayEvent::MenuItemClick { tray_id, id, .. } => handle_menu_item_click(app, tray_id, id),
            SystemTrayEvent::LeftClick {
                  // tray_id, 
                  // position, 
                  // size , 
                  ..
            } => handle_left_click(app),
            SystemTrayEvent::RightClick {
                  // tray_id, 
                  // position, 
                  // size , 
                  ..
            } => handle_left_click(app),
            SystemTrayEvent::DoubleClick {
                  // tray_id, 
                  // position, 
                  // size, 
                  ..
            } => handle_left_click(app),
            _ => todo!(),
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
fn handle_menu_item_click(app: &AppHandle, _tray_id: String, id: String) {
    match id.as_str() {
        "quit" => {
            // quit the app
            std::process::exit(0);
        }
        "next_page" => {
            let clips = app.state::<ClipDataMutex>();
            let mut clips = clips.clip_data.lock().unwrap();
            clips.next_page(app);

            // update the tray
            send_tray_update_event(app);
        }
        "prev_page" => {
            let clips = app.state::<ClipDataMutex>();
            let mut clips = clips.clip_data.lock().unwrap();
            clips.prev_page(app);

            // update the tray
            send_tray_update_event(app);
        }
        "first_page" => {
            let clips = app.state::<ClipDataMutex>();
            let mut clips = clips.clip_data.lock().unwrap();
            clips.first_page();

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
            }

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
            }

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
        _ => {
            // test if the id is a tray_clip
            if id.starts_with("tray_clip_") {
                // get the index of the clip
                let index = id.replace("tray_clip_", "").parse::<i64>().unwrap();

                // select the index
                let clips = app.state::<ClipDataMutex>();
                let mut clips = clips.clip_data.lock().unwrap();
                let item_id = clips.clips.tray_ids_map.get(index as usize);
                if item_id.is_none() {
                    warn!("Failed to get the item id for the tray id: {}", index);
                    return;
                }
                let item_id = item_id.unwrap();
                let item_id = *item_id;
                let res = clips.select_clip(app, item_id);
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

fn handle_left_click(_app: &AppHandle) {
    // do nothing
}

pub fn send_tray_update_event(app: &AppHandle) {
    let event = app.state::<EventSender>();
    event.send(crate::event::CopyClipEvent::TrayUpdateEvent);
}
