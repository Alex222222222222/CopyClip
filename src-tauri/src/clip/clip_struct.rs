use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::Error;

use super::clip_type::ClipType;

/// a single clip
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Clip {
    /// The text of the clip.
    /// After the clip is created, the text should not be changed
    #[serde(
        deserialize_with = "arc_string_deserialize",
        serialize_with = "arc_string_serialize"
    )]
    pub text: Arc<String>,
    /// The type of the clip
    pub clip_type: ClipType,
    /// in seconds
    pub timestamp: i64,
    /// the id of the clip
    pub id: i64,
    ///  if the clip is a favourite 1 means true, 0 means false
    pub favourite: bool,
    /// if the clip is pinned 1 means true, 0 means false
    pub pinned: bool,
}

pub fn arc_string_deserialize<'de, D>(deserializer: D) -> Result<Arc<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(Arc::new(s))
}

pub fn arc_string_serialize<S>(s: &Arc<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(s)
}

impl Clip {
    /// copy the clip to the clipboard
    pub fn copy_clip_to_clipboard(&self, app: &AppHandle) -> Result<(), Error> {
        let clipboard_manager = app.state::<tauri_plugin_clipboard::ClipboardManager>();
        match clipboard_manager.write_text((*self.text).clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::WriteToSystemClipboardErr(
                (*self.text).clone(),
                e.to_string(),
            )),
        }
    }

    /// convert to json format string
    pub fn to_json_string(&self) -> Result<String, Error> {
        match serde_json::to_string(self) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::ExportError(e.to_string())),
        }
    }
}
