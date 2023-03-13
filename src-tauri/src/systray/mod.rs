use tauri::{SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, AppHandle, SystemTrayEvent, SystemTraySubmenu};

pub fn create_tray() -> SystemTray {
      // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
      let notice_select = CustomMenuItem::new("notice_select".to_string(), "Select the clip you want to add to your clipboard.");

      let tray_clip_submenu = create_tray_clip_submenu_menu();

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

fn handle_menu_item_click(_app: &AppHandle, _tray_id: String, id: String) {
      match id.as_str(){
            "quit" => {
                  // quit the app
                  std::process::exit(0);
            }
            _ => {
                  // do nothing
            }
      }
}

fn handle_left_click(_app: &AppHandle) {
      // do nothing
}

fn create_tray_clip_submenu_menu() -> SystemTraySubmenu {
      let menu = SystemTrayMenu::new();
      // first create the empty menu
      // when the user clicks on the menu, the app will populate the menu with the latest clips

      let submenu = SystemTraySubmenu::new("Clips", menu);

      submenu
}