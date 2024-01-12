#[cfg(feature = "clip-cache")]
pub mod cache;
pub mod clip_data;
pub mod clip_struct;
pub mod database;
pub mod monitor;
pub mod search;

use crate::event::{CopyClipEvent, EventSender};

use self::clip_data::ClipData;

/// get the unix epoch timestamp in seconds
pub fn get_system_timestamp() -> i64 {
    let now = std::time::SystemTime::now();
    let unix_epoch = now
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    unix_epoch.as_secs() as i64
}

/// tell if the clip is pinned
#[tauri::command]
pub async fn id_is_pinned(clip_data: tauri::State<'_, ClipData>, id: i64) -> Result<bool, String> {
    let clips = clip_data.clips.lock().await;
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
    clip_data: tauri::State<'_, ClipData>,
    id: i64,
) -> Result<(), String> {
    let clip_data = clip_data.get_clip(Some(id)).await;
    if let Err(err) = clip_data {
        return Err(err.message());
    }
    let clip_data = clip_data.unwrap();
    if clip_data.is_none() {
        return Err("Clip not found.".to_string());
    }
    let clip_data = clip_data.unwrap();
    let res = clip_data.copy_clip_to_clipboard(&app);
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
    clip_data: tauri::State<'_, ClipData>,
    event_sender: tauri::State<'_, EventSender>,
    id: i64,
) -> Result<(), String> {
    let res = clip_data.delete_clip(Some(id)).await;

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
    clip_data: tauri::State<'_, ClipData>,
    event_sender: tauri::State<'_, EventSender>,
    id: i64,
    target: bool,
) -> Result<(), String> {
    let res = clip_data.change_clip_favourite_status(id, target).await;

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
    event_sender: tauri::State<'_, EventSender>,
    clip_data: tauri::State<'_, ClipData>,
    id: i64,
    pinned: bool,
) -> Result<(), String> {
    let res = clip_data.change_clip_pinned_status(id, !pinned).await;
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
