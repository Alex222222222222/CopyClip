#[cfg(debug_assertions)]
use log::debug;
use rust_i18n::set_locale;
use tauri::{Manager, Runtime, State};
use tauri_plugin_logging::LogLevelFilter;

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
pub async fn get_per_page_data(config: State<'_, ConfigMutex>) -> Result<u64, String> {
    let config = config.config.lock().await;
    let res = config.clip_per_page;
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
    data: u64,
) -> Result<(), String> {
    let mut config = config.config.lock().await;
    config.clip_per_page = data;

    let event_sender = app.state::<EventSender>();
    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;
    event_sender.send(CopyClipEvent::SaveConfigEvent).await;

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
pub async fn get_max_clip_len(config: State<'_, ConfigMutex>) -> Result<u64, String> {
    let config = config.config.lock().await;
    let res = config.clip_max_show_length;
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
    data: u64,
) -> Result<(), String> {
    let mut config = config.config.lock().await;
    config.clip_max_show_length = data;

    let event_sender = app.state::<EventSender>();
    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;
    event_sender.send(CopyClipEvent::SaveConfigEvent).await;

    Ok(())
}

/// get log_level_filter
///
/// input: {}
#[tauri::command]
pub async fn get_log_level_filter(config: State<'_, ConfigMutex>) -> Result<String, String> {
    let config = config.config.lock().await;
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
pub async fn set_log_level_filter(
    app: tauri::AppHandle,
    config: State<'_, ConfigMutex>,
    data: String,
) -> Result<(), String> {
    let mut config = config.config.lock().await;
    let log_level = LogLevelFilter::from(data);
    if log_level != config.log_level {
        config.log_level = log_level;
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::SaveConfigEvent).await;
        // TODO add restart to take effect to description
    }

    Ok(())
}

/// get dark_mode
///
/// input: {}
#[tauri::command]
pub async fn get_dark_mode(config: State<'_, ConfigMutex>) -> Result<bool, String> {
    let config = config.config.lock().await;
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
pub async fn set_dark_mode(
    app: tauri::AppHandle,
    config: State<'_, ConfigMutex>,
    data: bool,
) -> Result<(), String> {
    let mut config = config.config.lock().await;
    if config.dark_mode != data {
        config.dark_mode = data;
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::SaveConfigEvent).await;
    }

    Ok(())
}

/// get search_clip_per_batch
///
/// input: {}
#[tauri::command]
pub async fn get_search_clip_per_batch(config: State<'_, ConfigMutex>) -> Result<u64, String> {
    let config = config.config.lock().await;
    let res = config.search_clip_per_batch;

    #[cfg(debug_assertions)]
    debug!("get_search_clip_per_batch: {}", res);

    Ok(res)
}

/// set search_clip_per_page
///
/// input: {
///   data: i64
/// }
#[tauri::command]
pub async fn set_search_clip_per_batch(
    config: State<'_, ConfigMutex>,
    data: u64,
) -> Result<(), String> {
    let mut config = config.config.lock().await;
    config.search_clip_per_batch = data;

    Ok(())
}

/// get language
///
/// input: {}
#[tauri::command]
pub async fn get_language(config: State<'_, ConfigMutex>) -> Result<String, String> {
    let config = config.config.lock().await;
    let res = config.language.to_string();
    drop(config);
    Ok(res)
}

/// set language
///
/// input: {
///  data: String
///  }
#[tauri::command]
pub async fn set_language(
    app: tauri::AppHandle,
    config: State<'_, ConfigMutex>,
    data: String,
) -> Result<(), String> {
    let mut config = config.config.lock().await;
    if config.language != data {
        config.language = data;
        set_locale(&config.language);
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::SaveConfigEvent).await;
        event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;
    }

    Ok(())
}

/// get auto_delete_duplicate_clip
///
/// input: {}
#[tauri::command]
pub async fn get_auto_delete_duplicate_clip(
    config: State<'_, ConfigMutex>,
) -> Result<bool, String> {
    let config = config.config.lock().await;
    let res = config.auto_delete_duplicate_clip;
    drop(config);
    Ok(res)
}

/// set auto_delete_duplicate_clip
///
/// input: { data: bool }
#[tauri::command]
pub async fn set_auto_delete_duplicate_clip(
    app: tauri::AppHandle,
    config: State<'_, ConfigMutex>,
    data: bool,
) -> Result<(), String> {
    let mut config = config.config.lock().await;
    if config.auto_delete_duplicate_clip != data {
        config.auto_delete_duplicate_clip = data;
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::SaveConfigEvent).await;
    }

    Ok(())
}
