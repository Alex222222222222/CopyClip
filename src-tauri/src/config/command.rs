use tauri::{Manager, Runtime, State};

use crate::{
    event::{CopyClipEvent, EventSender},
    log::LogLevelFilter,
};

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

/// get log_level_filter
///
/// input: {}
#[tauri::command]
pub fn get_log_level_filter(config: State<'_, ConfigMutex>) -> Result<String, String> {
    let config = config.config.lock().unwrap();
    let res = config.log_level.to_string();
    drop(config);
    Ok(res)
}

/// set log_level_filter
///
/// input: {
///    data: i64
/// }
#[tauri::command]
pub fn set_log_level_filter(
    app: tauri::AppHandle,
    config: State<'_, ConfigMutex>,
    data: String,
) -> Result<(), String> {
    let mut config = config.config.lock().unwrap();
    let log_level = LogLevelFilter::from(data);
    if log_level != config.log_level {
        config.log_level = log_level;
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::SaveConfigEvent);
        // TODO add restart to take effect to description
    }

    Ok(())
}

/// get dark_mode
///
/// input: {}
#[tauri::command]
pub fn get_dark_mode(config: State<'_, ConfigMutex>) -> Result<bool, String> {
    let config = config.config.lock().unwrap();
    let res = config.dark_mode;
    drop(config);
    Ok(res)
}

/// set dark_mode
///
/// input: {
///   data: bool
/// }
#[tauri::command]
pub fn set_dark_mode(
    app: tauri::AppHandle,
    config: State<'_, ConfigMutex>,
    data: bool,
) -> Result<(), String> {
    let mut config = config.config.lock().unwrap();
    if config.dark_mode != data {
        config.dark_mode = data;
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::SaveConfigEvent);
        // TODO send event to change theme
    }

    Ok(())
}

/// get log_level_filter
///
/// input: {}
#[tauri::command]
pub fn get_log_level_filter(config: State<'_, ConfigMutex>) -> Result<String, String> {
    let config = config.config.lock().unwrap();
    let res = config.log_level.to_string();
    drop(config);
    Ok(res)
}

/// set log_level_filter
///
/// input: {
///    data: i64
/// }
#[tauri::command]
pub fn set_log_level_filter(
    app: tauri::AppHandle,
    config: State<'_, ConfigMutex>,
    data: String,
) -> Result<(), String> {
    let mut config = config.config.lock().unwrap();
    let log_level = LogLevelFilter::from(data);
    if log_level != config.log_level {
        config.log_level = log_level;
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::SaveConfigEvent);
        // TODO add restart to take effect to description
    }

    Ok(())
}
