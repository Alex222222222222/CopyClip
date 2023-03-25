use tauri::{Manager, Runtime, State};

use crate::event::{CopyClipEvent, EventSender};

use super::ConfigMutex;

/// get the number of clips to show in the tray menu
///
/// input: {}
///
/// output: {
///    data: i64
/// }
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
///
/// input: {
///     data: i64
/// }
///
/// output: {}
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
///
/// input: {}
///
/// output: {
///    i64.to_string()
/// }
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
///
/// input: {
///     data: i64
/// }
///
/// output: {}
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

/// get search_clip_per_page
/// this define how many clips to show per page in the search page
///
/// input: {}
///
/// output: {
///     i64.to_string()
/// }
#[tauri::command]
pub fn get_search_clip_per_page(config: State<'_, ConfigMutex>) -> Result<String, String> {
    let config = config.config.lock().unwrap();
    let res = config.search_clip_per_page.to_string();
    drop(config);
    Ok(res)
}

/// set search_clip_per_page
///
/// input: {
///     data: i64
/// }
///
/// output: {}
#[tauri::command]
pub fn set_search_clip_per_page(
    app: tauri::AppHandle,
    config: State<'_, ConfigMutex>,
    data: i64,
) -> Result<(), String> {
    let mut config = config.config.lock().unwrap();
    config.search_clip_per_page = data;

    let event_sender = app.state::<EventSender>();
    event_sender.send(CopyClipEvent::SaveConfigEvent);

    Ok(())
}
