use serde::{Deserialize, Serialize};

use crate::ClipType;

/// a single clip
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "yew", derive(yew::Properties, PartialEq))]
pub struct Clip {
    /// The text of the clip.
    /// After the clip is created, the text should not be changed
    pub data: Vec<u8>,
    /// The search text of the clip
    pub search_text: String,
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

impl Clip {
    /// convert to json format string
    pub fn to_json_string(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)?)
    }

    /// Create a new clip with given id and text and clip_type and timestamp
    /// automatically assign none labels
    #[cfg(feature = "backend")]
    pub fn new_clip_with_id_timestamp<T>(
        id: u64,
        text: &str,
        clip_type: T,
        timestamp: i64,
    ) -> Result<Self, anyhow::Error>
    where
        T: Into<ClipType> + Copy,
    {
        use crate::{compress_text, convert_text_to_search_text};

        let st = convert_text_to_search_text(clip_type, text)?;
        let compressed_data = compress_text(text)?;

        Ok(Self {
            data: compressed_data,
            id,
            search_text: st,
            clip_type: clip_type.into(),
            timestamp,
            labels: vec![],
        })
    }

    /// Create a new clip with given id and text and clip_type
    /// automatically assign none labels
    #[cfg(feature = "backend")]
    pub fn new_clip_with_id<T>(id: u64, text: &str, clip_type: T) -> Result<Self, anyhow::Error>
    where
        T: Into<ClipType> + Copy,
    {
        use crate::timestamp;

        Self::new_clip_with_id_timestamp(id, text, clip_type, timestamp())
    }

    /// Get the decompressed data of a clip
    #[cfg(feature = "compress")]
    pub fn decompress_text(&self) -> Result<String, anyhow::Error> {
        use crate::decompress_text;

        decompress_text(&self.data)
    }

    /// Create a clip from a database row,
    /// the database row should contain,
    /// - id, u64
    /// - timestamp, u64
    /// - data, BLOB
    /// - search_text, String
    /// - type, u8
    #[cfg(feature = "database")]
    pub fn from_database_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        let id: u64 = row.get("id")?;
        let clip_type: u8 = row.get("type")?;
        let data: Vec<u8> = row.get("data")?;
        let timestamp: i64 = row.get("timestamp")?;
        let search_text: String = row.get("search_text")?;

        Ok(Self {
            data,
            id,
            search_text,
            clip_type: clip_type.into(),
            timestamp,
            labels: vec![],
        })
    }

    /// Get id in rusqlite::types::Value format
    #[cfg(feature = "database")]
    pub fn get_id_database(&self) -> rusqlite::types::Value {
        rusqlite::types::Value::Integer(self.id as i64)
    }

    /// Get timestamp in rusqlite::types::Value format
    #[cfg(feature = "database")]
    pub fn get_timestamp_database(&self) -> rusqlite::types::Value {
        rusqlite::types::Value::Integer(self.timestamp)
    }

    /// Get clip_type in rusqlite::types::Value format
    #[cfg(feature = "database")]
    pub fn get_clip_type_database(&self) -> rusqlite::types::Value {
        rusqlite::types::Value::Integer(self.clip_type.into())
    }

    /// Get data in rusqlite::types::Value format
    #[cfg(feature = "database")]
    pub fn get_data_database(&self) -> rusqlite::types::Value {
        rusqlite::types::Value::Blob(self.data.clone())
    }

    /// Get search_text in rusqlite::types::Value format
    #[cfg(feature = "database")]
    pub fn get_search_text_database(&self) -> rusqlite::types::Value {
        rusqlite::types::Value::Text(self.search_text.clone())
    }
}
