use tauri::{Manager, Runtime, State};

use crate::event::{CopyClipEvent, EventSender};

use super::ConfigMutex;

/// get the number of clips to show in the tray menu
#[tauri::command]
pub fn get_per_page_data(config: State<'_, ConfigMutex>) -> Result<String, String> {
    let config = config.config.lock().unwrap();
    let res = config.clip_per_page.to_string();
    drop(config);
    Ok(res)
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
) -> Result<(), String> {
    let mut config = config.config.lock().unwrap();
    config.clip_per_page = data;

    let event_sender = app.state::<EventSender>();
    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent);
    event_sender.send(CopyClipEvent::SaveConfigEvent);

    Ok(())
}

/// get the len of maximum show length of a clip
/// if the len is 0, then the clip will not be truncated
///
/// this is used to prevent the tray menu from being too long
#[tauri::command]
pub fn get_max_clip_len(config: State<'_, ConfigMutex>) -> Result<String, String> {
    let config = config.config.lock().unwrap();
    let res = config.clip_max_show_length.to_string();
    drop(config);
    Ok(res)
}

/// set the len of maximum show length of a clip
///
/// this will also rebuild the tray menu
/// also trigger save config
#[tauri::command]
pub async fn set_max_clip_len<R: Runtime>(
    app: tauri::AppHandle<R>,
    config: State<'_, ConfigMutex>,
    data: i64,
) -> Result<(), String> {
    let mut config = config.config.lock().unwrap();
    config.clip_max_show_length = data;

    let event_sender = app.state::<EventSender>();
    event_sender.send(CopyClipEvent::TrayUpdateEvent);
    event_sender.send(CopyClipEvent::SaveConfigEvent);

    Ok(())
}
