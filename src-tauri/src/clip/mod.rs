pub mod clip_data;
pub mod database;
pub mod monitor;
pub mod search;

use tauri::{AppHandle, Manager};

use crate::{
    error::Error,
    event::{CopyClipEvent, EventSender},
};

use self::clip_data::ClipStateMutex;

/// get the unix epoch timestamp in seconds
pub fn get_system_timestamp() -> i64 {
    let now = std::time::SystemTime::now();
    let unix_epoch = now
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    unix_epoch.as_secs() as i64
}

/// copy the clip to the clipboard
pub fn copy_clip_to_clipboard_in(clip: &clip::Clip, app: &AppHandle) -> Result<(), Error> {
    let clipboard_manager = app.state::<tauri_plugin_clipboard::ClipboardManager>();
    match clipboard_manager.write_text((*clip.text).clone()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::WriteToSystemClipboardErr(
            (*clip.text).clone(),
            e.to_string(),
        )),
    }
}

/// tell if the clip is pinned
#[tauri::command]
pub async fn id_is_pinned(
    clip_state: tauri::State<'_, ClipStateMutex>,
    id: i64,
) -> Result<bool, String> {
    let clips = clip_state.clip_state.lock().await;
    let res = clips.pinned_clips.iter().find(|&&x| x == id);
    if res.is_none() {
        return Ok(false);
    }
    Ok(true)
}

#[tauri::command]
pub async fn copy_clip_to_clipboard(
    app: tauri::AppHandle,
    event_sender: tauri::State<'_, EventSender>,
    clip_state: tauri::State<'_, ClipStateMutex>,
    id: i64,
) -> Result<(), String> {
    let clip_data_mutex = clip_state.clip_state.lock().await;
    let clip_data = clip_data_mutex.get_clip(&app, Some(id)).await;
    drop(clip_data_mutex);
    if let Err(err) = clip_data {
        return Err(err.message());
    }
    let clip_data = clip_data.unwrap();
    if clip_data.is_none() {
        return Err("Clip not found.".to_string());
    }
    let clip_data = clip_data.unwrap();
    let res = copy_clip_to_clipboard_in(&clip_data, &app);
    if let Err(err) = res {
        return Err(err.message());
    }

    event_sender
        .send(CopyClipEvent::SendNotificationEvent(
            "Clip copied to clipboard.".to_string(),
        ))
        .await;

    Ok(())
}

/// Delete a normal clip from the database
#[tauri::command]
pub async fn delete_clip_from_database(
    app: tauri::AppHandle,
    clip_state: tauri::State<'_, ClipStateMutex>,
    event_sender: tauri::State<'_, EventSender>,
    id: i64,
) -> Result<(), String> {
    let mut clip_state_mutex = clip_state.clip_state.lock().await;
    let res = clip_state_mutex.delete_clip(&app, Some(id)).await;
    drop(clip_state_mutex);

    if let Err(err) = res {
        return Err(err.to_string());
    }

    event_sender.send(CopyClipEvent::TrayUpdateEvent).await;
    event_sender
        .send(CopyClipEvent::SendNotificationEvent(
            "Clip deleted from database.".to_string(),
        ))
        .await;

    Ok(())
}

#[tauri::command]
pub async fn change_favourite_clip(
    app: tauri::AppHandle,
    clip_state: tauri::State<'_, ClipStateMutex>,
    event_sender: tauri::State<'_, EventSender>,
    id: i64,
    target: bool,
) -> Result<(), String> {
    let mut clip_state_mutex = clip_state.clip_state.lock().await;
    let res = clip_state_mutex
        .change_clip_favourite_status(&app, id, target)
        .await;
    drop(clip_state_mutex);

    if let Err(err) = res {
        return Err(err.to_string());
    }

    event_sender
        .send(CopyClipEvent::SendNotificationEvent(
            "Clip favourite status changed.".to_string(),
        ))
        .await;

    Ok(())
}

#[tauri::command]
pub async fn switch_pinned_status(
    app: tauri::AppHandle,
    event_sender: tauri::State<'_, EventSender>,
    clip_state: tauri::State<'_, ClipStateMutex>,
    id: i64,
    pinned: bool,
) -> Result<(), String> {
    let mut clip_state_mutex = clip_state.clip_state.lock().await;
    let res = clip_state_mutex
        .change_clip_pinned_status(&app, id, !pinned)
        .await;
    drop(clip_state_mutex);
    if let Err(err) = res {
        return Err(err.message());
    }

    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;
    event_sender
        .send(CopyClipEvent::SendNotificationEvent(
            "Clip pinned to tray.".to_string(),
        ))
        .await;

    Ok(())
}
