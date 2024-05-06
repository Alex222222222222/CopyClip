use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// the type of the clip
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize, Default)]
pub enum ClipType {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
    /// file path
    ///
    /// each line is a file path
    #[serde(rename = "file")]
    File,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "rtf")]
    Rtf,
}

impl std::fmt::Display for ClipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Image => write!(f, "image"),
            Self::File => write!(f, "file"),
            Self::Html => write!(f, "html"),
            Self::Rtf => write!(f, "rtf"),
        }
    }
}

impl From<ClipType> for String {
    fn from(clip_type: ClipType) -> Self {
        clip_type.to_string()
    }
}

impl From<&ClipType> for String {
    fn from(clip_type: &ClipType) -> Self {
        clip_type.to_string()
    }
}

impl From<&str> for ClipType {
    fn from(s: &str) -> Self {
        match s {
            "text" => Self::Text,
            "image" => Self::Image,
            "file" => Self::File,
            "html" => Self::Html,
            "rtf" => Self::Rtf,
            _ => Self::Text,
        }
    }
}

impl From<String> for ClipType {
    fn from(s: String) -> Self {
        ClipType::from(s.as_str())
    }
}

impl From<&String> for ClipType {
    fn from(s: &String) -> Self {
        ClipType::from(s.as_str())
    }
}

impl From<Arc<String>> for ClipType {
    fn from(s: Arc<String>) -> Self {
        ClipType::from(s.as_str())
    }
}

impl From<&Arc<String>> for ClipType {
    fn from(s: &Arc<String>) -> Self {
        ClipType::from(s.as_str())
    }
}

impl From<ClipType> for Arc<String> {
    fn from(clip_type: ClipType) -> Self {
        Arc::new(clip_type.to_string())
    }
}
