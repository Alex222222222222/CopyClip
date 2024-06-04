use std::io::Write;

use crate::clip::clip_data::{ClipState, ClipStateMutex};
use crate::config::ConfigMutex;

use crate::event::EventSender;

extern crate directories;

use base64::Engine;

#[cfg(debug_assertions)]
use log::debug;

use log::warn;
use tauri::{AppHandle, Manager};

use flate2::write::GzEncoder;
use flate2::Compression;

/// use to export the use data
/// the data is the data to export
///
/// the data contains three parts,
/// the first part is the user config encoded in json format, then encoded in base64
/// the second part is the version json Vec<id,version> in database file data encoded in base64
/// the third part is the data json Vec<Clip> in database file data encoded in base64
/// like this:
///     - config
///     - :
///     - versions
///     - :
///     - clip_1
///     - :
///     - clip_2
///     - :
///     - clip_3
///     - ...
/// and the three parts are separated by a ":"
///
/// then the data is compressed using gzip
/// and then save to user download folder
#[warn(unused_must_use)]
async fn export_data(app: &AppHandle, path: String) -> Result<(), anyhow::Error> {
    // write to file
    // get the file path
    let mut file_path = path;
    file_path.push_str("/copy_clip_data.gz");
    #[cfg(debug_assertions)]
    {
        let d = file_path.to_string();
        debug!("{}", d);
    }

    let file = std::fs::File::create(file_path)?;

    // let mut e = ZlibEncoder::new(file, Compression::default());
    let mut e = GzEncoder::new(file, Compression::new(9));

    // get config json
    let config = app.state::<ConfigMutex>();
    let config = config.config.lock().await;
    let c_json = config.to_json()?;
    drop(config);
    // encode the config json to base64
    let c_json = base64::engine::general_purpose::STANDARD.encode(c_json.as_bytes());
    #[cfg(debug_assertions)]
    debug!("config json: {}", c_json);
    // write to the encoder
    e.write_all(c_json.as_bytes())?;

    // write the separator
    e.write_all(":".as_bytes())?;

    // get all versions from database
    let versions = app
        .state::<crate::database::DatabaseStateMutex>()
        .get_all_versions()
        .await?;
    // convert to json
    let v_json = serde_json::to_string(&versions)?;
    // encode the json to base64
    let v_json = base64::engine::general_purpose::STANDARD.encode(v_json.as_bytes());
    // write to the encoder
    e.write_all(v_json.as_bytes())?;

    // write the separator
    e.write_all(":".as_bytes())?;

    // the clips
    let clip_data = app.state::<ClipStateMutex>();
    let clip_data = clip_data.clip_state.lock().await;
    let clips_len = ClipState::get_total_number_of_clip(app).await?;
    for i in 0..clips_len {
        let id = ClipState::get_id_with_pos(app, i).await?;
        let clip = clip_data.get_clip(app, id).await?;
        if clip.is_none() {
            continue;
        }
        let clip = clip.unwrap();
        let c_json = clip.to_json_string()?;
        let c_json = base64::engine::general_purpose::STANDARD.encode(c_json.as_bytes());
        e.write_all(c_json.as_bytes())?;
        e.write_all(":".as_bytes())?;
    }
    drop(clip_data);

    let mut file = e.finish()?;

    file.flush()?;

    file.sync_all()?;

    #[cfg(debug_assertions)]
    debug!("export data success");

    Ok(())
}

#[tauri::command]
pub async fn export_data_invoke(
    app: tauri::AppHandle,
    event_sender: tauri::State<'_, EventSender>,
) -> Result<(), String> {
    // save to user download dir
    // get the download dir
    let path = directories::UserDirs::new();
    if path.is_none() {
        return Err("can not get user download dir".to_string());
    }
    let path = path.unwrap();
    let path = path.download_dir();
    if path.is_none() {
        return Err("can not get user download dir".to_string());
    }
    let path = path.unwrap();
    let path = path.to_str();
    if path.is_none() {
        return Err("can not get user download dir".to_string());
    }
    let path = path.unwrap();
    let path = path.to_string();
    if let Err(err) = export_data(&app, path).await {
        return Err(err.to_string());
    }

    event_sender
        .send(crate::event::CopyClipEvent::SendNotificationEvent(
            "Export data successful.".to_string(),
        ))
        .await;

    Ok(())
}
