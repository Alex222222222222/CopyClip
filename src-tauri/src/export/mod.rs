use std::io::Write;

use crate::clip;
use crate::clip::ClipDataMutex;
use crate::config::ConfigMutex;

use crate::error;
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
async fn export_data(app: &AppHandle, path: String) -> Result<(), error::Error> {
    // write to file
    // get the file path
    let mut file_path = path;
    file_path.push_str("/copy_clip_data.gz");
    #[cfg(debug_assertions)]
    {
        let d = file_path.to_string();
        debug!("{}", d);
    }

    let file = std::fs::File::create(file_path);
    if let Err(e) = file {
        return Err(error::Error::ExportError(e.to_string()));
    }
    let file = file.unwrap();

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
    let res = e.write_all(c_json.as_bytes());
    if let Err(e) = res {
        return Err(error::Error::ExportError(e.to_string()));
    }

    // write the separator
    let res = e.write_all(":".as_bytes());
    if let Err(e) = res {
        return Err(error::Error::ExportError(e.to_string()));
    }

    // get all versions from database
    let versions = clip::database::get_all_versions(app).await?;
    // convert to json
    let v_json = serde_json::to_string(&versions);
    if let Err(e) = v_json {
        return Err(error::Error::ExportError(e.to_string()));
    }
    let v_json = v_json.unwrap();
    // encode the json to base64
    let v_json = base64::engine::general_purpose::STANDARD.encode(v_json.as_bytes());
    // write to the encoder
    let res = e.write_all(v_json.as_bytes());
    if let Err(e) = res {
        return Err(error::Error::ExportError(e.to_string()));
    }

    // write the separator
    let res = e.write_all(":".as_bytes());
    if let Err(e) = res {
        return Err(error::Error::ExportError(e.to_string()));
    }

    // the clips
    let clip_data = app.state::<ClipDataMutex>();
    let mut clip_data = clip_data.clip_data.lock().await;
    for i in 0..clip_data.clips.whole_list_of_ids.len() {
        let id = clip_data.clips.whole_list_of_ids[i];
        let clip = clip_data.get_clip(id).await?;
        let c_json = serde_json::to_string(&clip);
        if let Err(e) = c_json {
            return Err(error::Error::ExportError(e.to_string()));
        }
        let c_json = c_json.unwrap();
        let c_json = base64::engine::general_purpose::STANDARD.encode(c_json.as_bytes());
        let res = e.write_all(c_json.as_bytes());
        if let Err(e) = res {
            return Err(error::Error::ExportError(e.to_string()));
        }
        let res = e.write_all(":".as_bytes());
        if let Err(e) = res {
            return Err(error::Error::ExportError(e.to_string()));
        }
    }
    drop(clip_data);

    let res = e.finish();
    if let Err(e) = res {
        return Err(error::Error::ExportError(e.to_string()));
    }
    let mut file = res.unwrap();

    let res = file.flush();
    if let Err(err) = res {
        return Err(error::Error::ExportError(err.to_string()));
    }

    let res = file.sync_all();
    if let Err(err) = res {
        return Err(error::Error::ExportError(err.to_string()));
    }

    #[cfg(debug_assertions)]
    debug!("export data success");

    Ok(())
}

#[tauri::command]
pub async fn export_data_invoke(
    app: tauri::AppHandle,
    event_sender: tauri::State<'_, EventSender>,
) -> Result<(), error::Error> {
    // save to user download dir
    // get the download dir
    let path = directories::UserDirs::new();
    if path.is_none() {
        return Err(error::Error::ExportError(
            "can not get user download dir".to_string(),
        ));
    }
    let path = path.unwrap();
    let path = path.download_dir();
    if path.is_none() {
        return Err(error::Error::ExportError(
            "can not get user download dir".to_string(),
        ));
    }
    let path = path.unwrap();
    let path = path.to_str();
    if path.is_none() {
        return Err(error::Error::ExportError(
            "can not get user download dir".to_string(),
        ));
    }
    let path = path.unwrap();
    let path = path.to_string();
    let res = export_data(&app, path).await;
    if let Err(err) = res {
        warn!("{}", err);
        return Err(err);
    }

    event_sender.send(crate::event::CopyClipEvent::SendNotificationEvent(
        "Export data successful.".to_string(),
    ));

    Ok(())
}
