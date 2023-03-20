use tauri::{Manager, Runtime, State};

use crate::event::{CopyClipEvent, EventSender};

use super::ConfigMutex;

/// get the number of clips to show in the tray menu
#[tauri::command]
pub fn get_per_page_data(config: State<'_, ConfigMutex>) -> Result<String, String> {
    let config = config.config.lock().unwrap();
    Ok(config.clips_to_show.to_string())
}

/// set the number of clips to show in the tray menu
///
/// this will also rebuild the tray menu
/// also trigger save config
#[tauri::command]
pub async fn set_per_page_data<R: Runtime>(
    app: tauri::AppHandle<R>,
    config: State<'_, ConfigMutex>,
    data: i64,
) -> Result<String, String> {
    let mut config = config.config.lock().unwrap();
    config.clips_to_show = data;

    let event_sender = app.state::<EventSender>();
    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent);

    Ok(config.clips_to_show.to_string())
}
