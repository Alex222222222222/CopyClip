pub mod clip_data;
mod image_clip;
pub mod monitor;

use clip_data::ClipState;
use log::debug;
use tauri::{AppHandle, Manager};

use crate::{
    error::Error,
    event::{CopyClipEvent, EventSender},
};

use self::clip_data::ClipStateMutex;

/// copy the clip to the clipboard
pub fn copy_clip_to_clipboard_in(clip: &clip::Clip, app: &AppHandle) -> Result<(), anyhow::Error> {
    let clipboard_manager = app.state::<tauri_plugin_clipboard::ClipboardManager>();
    let res = match clip.clip_type {
        clip::ClipType::Text => clipboard_manager.write_text(clip.decompress_text()?),
        clip::ClipType::Image => {
            let img = image_clip::get_img(&clip.decompress_text()?)?;
            clipboard_manager.write_image_binary(img)
        }
        clip::ClipType::File => {
            // json deserialize the json encoded file uri
            let file: Vec<String> = serde_json::from_str(&clip.decompress_text()?)?;
            clipboard_manager.write_files_uris(file)
        }
        clip::ClipType::Html => clipboard_manager.write_html(clip.decompress_text()?),
        clip::ClipType::Rtf => clipboard_manager.write_rtf(clip.decompress_text()?),
    };

    debug!("write to clipboard result: {:?}", res);

    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::WriteToSystemClipboardErr("".to_string(), err).into()),
    }
}

/// tell if the clip is pinned
#[tauri::command]
pub async fn id_is_pinned(app: AppHandle, id: u64) -> Result<bool, String> {
    let res = ClipState::clip_have_label(&app, id, "pinned").await;
    match res {
        Ok(res) => Ok(res),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn copy_clip_to_clipboard(
    app: tauri::AppHandle,
    event_sender: tauri::State<'_, EventSender>,
    clip_state: tauri::State<'_, ClipStateMutex>,
    id: u64,
) -> Result<(), String> {
    let clip_data_mutex = clip_state.clip_state.lock().await;
    let clip_data = clip_data_mutex.get_clip(&app, Some(id)).await;
    drop(clip_data_mutex);
    if let Err(err) = clip_data {
        return Err(err.to_string());
    }
    let clip_data = clip_data.unwrap();
    if clip_data.is_none() {
        return Err("Clip not found.".to_string());
    }
    let clip_data = clip_data.unwrap();
    let res = copy_clip_to_clipboard_in(&clip_data, &app);
    if let Err(err) = res {
        return Err(err.to_string());
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
    id: u64,
) -> Result<(), String> {
    let mut clip_state_mutex = clip_state.clip_state.lock().await;
    let res = clip_state_mutex.delete_clip(&app, Some(id)).await;
    drop(clip_state_mutex);

    if let Err(err) = res {
        return Err(err.to_string());
    }

    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;
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
    id: u64,
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
    id: u64,
    pinned: bool,
) -> Result<(), String> {
    let mut clip_state_mutex = clip_state.clip_state.lock().await;
    let res = clip_state_mutex
        .change_clip_pinned_status(&app, id, !pinned)
        .await;
    drop(clip_state_mutex);
    if let Err(err) = res {
        return Err(err.to_string());
    }

    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;
    event_sender
        .send(CopyClipEvent::SendNotificationEvent(
            "Clip pinned to tray.".to_string(),
        ))
        .await;

    Ok(())
}
