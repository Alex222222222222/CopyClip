use tauri::{SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, AppHandle, SystemTrayEvent, SystemTraySubmenu, Manager};

use crate::clip::ClipDataMutex;

pub fn create_tray(num : i64) -> SystemTray {
      // TODO set icon for tray

      // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
      let notice_select = CustomMenuItem::new("notice_select".to_string(), "Select the clip you want to add to your clipboard.");

      let tray_clip_submenu = create_tray_clip_submenu_menu(num);

      let page_info = CustomMenuItem::new("page_info".to_string(), ""); // Total clips: 0, Current page: 0/0
      let next_page = CustomMenuItem::new("next_page".to_string(), "Next page");
      let prev_page = CustomMenuItem::new("prev_page".to_string(), "Previous page");
      let first_page = CustomMenuItem::new("first_page".to_string(), "First page");

      let preferences = CustomMenuItem::new("preferences".to_string(), "Preferences");

      let quit = CustomMenuItem::new("quit".to_string(), "Quit");
      let tray_menu = SystemTrayMenu::new()
            .add_item(notice_select)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_submenu(tray_clip_submenu)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(page_info)
            .add_item(next_page)
            .add_item(prev_page)
            .add_item(first_page)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(preferences)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit);

      let tray = SystemTray::new().with_menu(tray_menu);

      tray
}

pub fn handle_tray_event(app: &AppHandle,event: SystemTrayEvent) {
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

fn handle_menu_item_click(app: &AppHandle, _tray_id: String, id: String) {
      match id.as_str(){
            "quit" => {
                  // quit the app
                  std::process::exit(0);
            }
            "next_page" => {
                  let clips = app.state::<ClipDataMutex>();
                  let mut clips = clips.clip_data.lock().unwrap();
                  clips.next_page(app);
            }
            "prev_page" => {
                  let clips = app.state::<ClipDataMutex>();
                  let mut clips = clips.clip_data.lock().unwrap();
                  clips.prev_page(app);
            }
            "first_page" => {
                  let clips = app.state::<ClipDataMutex>();
                  let mut clips = clips.clip_data.lock().unwrap();
                  clips.first_page();
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
                              // TODO send the error notification and panic
                              return;
                        }
                        let item_id = item_id.unwrap();
                        let item_id = (*item_id).clone();
                        let res = clips.select_clip(app, item_id);
                        if res.is_err() {
                              // TODO send the error notification and panic
                              return;
                        }
                  }
            }
      }
}

fn handle_left_click(_app: &AppHandle) {
      // do nothing
}

fn create_tray_clip_submenu_menu(num :i64) -> SystemTraySubmenu {

      let menu = SystemTrayMenu::new();
      // first create the empty menu
      // when the user clicks on the menu, the app will populate the menu with the latest clips

      let mut submenu = SystemTraySubmenu::new("Clips", menu);
      let mut menu = SystemTrayMenu::new();

      for i in 0..num {
            let clip = CustomMenuItem::new("tray_clip_".to_string() + &i.to_string(), "");
            menu = menu.add_item(clip);
      }

      submenu.inner = menu;

      submenu
}