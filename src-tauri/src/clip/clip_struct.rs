use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, ClipboardManager};

use crate::error::Error;

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
        let mut clipboard_manager = app.clipboard_manager();
        let res = clipboard_manager.write_text((*self.text).clone());
        if let Err(e) = res {
            return Err(Error::WriteToSystemClipboardErr(
                (*self.text).clone(),
                e.to_string(),
            ));
        }
        Ok(())
    }

    /// convert to json format string
    pub fn to_json_string(&self) -> Result<String, Error> {
        let res = serde_json::to_string(self);
        if let Err(e) = res {
            return Err(Error::ExportError(e.to_string()));
        }
        Ok(res.unwrap())
    }
}
