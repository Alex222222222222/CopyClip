mod clip_struct;
mod clip_type;
#[cfg(feature = "compress")]
mod compress_data;
#[cfg(feature = "search")]
mod search_text;

use std::time::{SystemTime, UNIX_EPOCH};

pub use clip_struct::Clip;
pub use clip_type::ClipType;
#[cfg(feature = "compress")]
use compress_data::{compress_text, decompress_text};
#[cfg(feature = "search")]
use search_text::convert_text_to_search_text;
#[cfg(feature = "search")]
pub use search_text::init_search;

/// return the unix epoch in seconds
pub fn timestamp() -> i64 {
    let time_now = SystemTime::now();
    match time_now.duration_since(UNIX_EPOCH) {
        Ok(time_now) => time_now.as_secs() as i64,
        Err(_) => -(UNIX_EPOCH.duration_since(time_now).unwrap().as_secs() as i64),
    }
}
