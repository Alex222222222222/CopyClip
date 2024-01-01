pub mod clip_data;
pub mod clip_id;
pub mod clip_struct;
pub mod clip_type;
pub mod database;
pub mod monitor;
pub mod pinned;
pub mod search;

mod cache;

use std::sync::Arc;

use log::debug;

use crate::event::{CopyClipEvent, EventSender};

use self::clip_data::ClipData;
use self::clip_id::ClipID;
use self::clip_struct::Clip;

pub fn get_system_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Copy a normal clip to the clipboard
#[tauri::command]
pub async fn copy_clip_to_clipboard(
    app: tauri::AppHandle,
    event_sender: tauri::State<'_, EventSender>,
    clip_data: tauri::State<'_, ClipData>,
    id: i64,
) -> Result<(), String> {
    let clip_data = clip_data.get_clip(ClipID::Clip(id)).await;
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
    let res = clip_data.delete_normal_clip(Some(id)).await;

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
    let res = clip_data.change_favourite_clip(id, target).await;

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

/// the function to handle the pinned clip change
/// if action is 0, then add the clip to the pinned clips
/// if action is 1, then remove the clip from the pinned clips
#[tauri::command]
pub async fn add_remove_pinned_clip(
    clip_data: tauri::State<'_, ClipData>,
    event_sender: tauri::State<'_, EventSender>,
    text: String,
    action: i64,
) -> Result<(), String> {
    if action != 0 && action != 1 {
        return Err("Invalid action.".to_string());
    }

    let text = Arc::new(text);
    let res = if action == 0 {
        debug!("add pinned clip");
        clip_data.add_pinned_clips(text).await
    } else {
        debug!("remove pinned clip");
        clip_data.remove_pinned_clip(text).await
    };

    if let Err(err) = res {
        return Err(err.to_string());
    }

    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;

    Ok(())
}
