use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, ClipboardManager};

use crate::error;

/// A pinned clip is a clip that is pinned to the top of the list
/// Stored in the database table `pinned_clips`
/// ```sql
/// CREATE TABLE pinned_clips (
///     id INTEGER PRIMARY KEY,
///     text TEXT,
///     timestamp INTEGER
/// );
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinnedClip {
    /// id of the pinned clip,
    /// may be same with some normal clip id.
    /// Need to be manually managed
    pub id: i64,
    /// The text of the clip.
    pub text: Arc<String>,
    /// in seconds
    pub timestamp: i64,
}

impl PinnedClip {
    /// copy the clip to the clipboard
    pub fn copy_clip_to_clipboard(&self, app: &AppHandle) -> Result<(), error::Error> {
        let mut clipboard_manager = app.clipboard_manager();
        let res = clipboard_manager.write_text((*self.text).clone());
        if let Err(e) = res {
            return Err(error::Error::WriteToSystemClipboardErr(
                (*self.text).clone(),
                e.to_string(),
            ));
        }
        Ok(())
    }
}
