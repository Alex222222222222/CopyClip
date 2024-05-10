use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ClipType;

/// a single clip
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "yew", derive(yew::Properties, PartialEq))]
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
    pub id: u64,
    /// the labels of the clip
    /// each label is a string
    pub labels: Vec<String>,
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
    /// convert to json format string
    pub fn to_json_string(&self) -> Result<String, String> {
        match serde_json::to_string(self) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        }
    }
}
