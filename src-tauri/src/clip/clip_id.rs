use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// wrapped the id of pinned and normal clip
pub enum ClipID {
    /// the current clip is the clip with the given id
    Clip(i64),
    /// is the pinned clip with the given id
    PinnedClip(i64),
    /// None means there is no current clip
    None,
}

impl Display for ClipID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipID::Clip(id) => write!(f, "Clip({})", id),
            ClipID::PinnedClip(id) => write!(f, "PinnedClip({})", id),
            ClipID::None => write!(f, "None"),
        }
    }
}

impl ClipID {
    /// Get the id of the clip,
    /// regardless of whether it is a pinned clip or a normal clip
    pub fn get_id(&self) -> Option<i64> {
        match self {
            ClipID::Clip(id) => Some(*id),
            ClipID::PinnedClip(id) => Some(*id),
            ClipID::None => None,
        }
    }

    /// Test if the clip is a pinned clip
    pub fn is_pinned_clip(&self) -> bool {
        matches!(self, ClipID::PinnedClip(_))
    }

    /// Test if the clip is a normal clip,
    /// we consider none as a normal clip
    pub fn is_clip(&self) -> bool {
        matches!(self, ClipID::Clip(_) | ClipID::None)
    }
}
