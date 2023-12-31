use std::sync::Arc;

use tauri::AppHandle;

use crate::error;

use super::{pinned::PinnedClip, Clip};

/// A clip is a piece of text that is copied to the clipboard
/// require to implement Debug, Clone, Serialize, Deserialize, Default
pub enum ClipType {
    /// A normal clip
    Clip(Clip),
    /// A pinned clip
    PinnedClip(PinnedClip),
}

impl ClipType {
    /// copy the clip to the clipboard
    pub fn copy_clip_to_clipboard(&self, app: &AppHandle) -> Result<(), error::Error> {
        match self {
            ClipType::Clip(clip) => clip.copy_clip_to_clipboard(app),
            ClipType::PinnedClip(pinned_clip) => pinned_clip.copy_clip_to_clipboard(app),
        }
    }

    /// get the text of the clip
    pub fn get_text(&self) -> Arc<String> {
        match self {
            ClipType::Clip(clip) => clip.text.clone(),
            ClipType::PinnedClip(pinned_clip) => pinned_clip.text.clone(),
        }
    }

    /// get the id of the clip
    pub fn get_id(&self) -> i64 {
        match self {
            ClipType::Clip(clip) => clip.id,
            ClipType::PinnedClip(pinned_clip) => pinned_clip.id,
        }
    }
}
