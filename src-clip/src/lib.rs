mod clip_struct;
mod clip_type;
#[cfg(feature = "compress")]
mod compress_data;
mod search_constraint;
#[cfg(feature = "search")]
mod search_text;

use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub use clip_struct::Clip;
pub use clip_type::ClipType;
#[cfg(feature = "compress")]
use compress_data::{compress_text, decompress_text};
use once_cell::sync::Lazy;
#[cfg(feature = "search")]
use search_text::convert_text_to_search_text;
#[cfg(feature = "search")]
pub use search_text::init_search;
use unicode_segmentation::UnicodeSegmentation;

pub use search_constraint::SearchConstraint;
pub use search_constraint::TextSearchMethod;

/// return the unix epoch in seconds
pub fn timestamp() -> i64 {
    let time_now = SystemTime::now();
    match time_now.duration_since(UNIX_EPOCH) {
        Ok(time_now) => time_now.as_secs() as i64,
        Err(_) => -(UNIX_EPOCH.duration_since(time_now).unwrap().as_secs() as i64),
    }
}

/// Calculate the thumbnail path of an img if given an img path
/// the thumbnail path is like [img_path]_thumbnail
pub fn thumbnail_path(img_path: &Path) -> PathBuf {
    let mut thumbnail_path = img_path.to_path_buf();
    let file_name = thumbnail_path.file_name().unwrap().to_str().unwrap();
    let file_name = file_name
        .split('.')
        .collect::<Vec<&str>>()
        .join("_thumbnail.");
    thumbnail_path.set_file_name(file_name);
    thumbnail_path
}

/// chars that consider as white space
static WHITE_SPACE: Lazy<Vec<&str>> = Lazy::new(|| vec![" ", "\t", "\n", "\r"]);

/// Trim the text to the given length.
///
/// Also take care of slicing the text in the middle of a unicode character
/// Also take care of the width of a unicode character
///
/// l is treated as 20 if l <= 6
pub fn trimming_clip_text(text: &str, l: u64) -> String {
    // trim the leading white space
    let mut text = text.graphemes(true);
    let l = if l <= 6 { 20 } else { l };

    let mut res: String = String::new();
    loop {
        let char = text.next();
        if char.is_none() {
            break;
        }
        let char = char.unwrap();
        if WHITE_SPACE.contains(&char) {
            continue;
        } else {
            res += char;
            break;
        }
    }

    let mut final_width = 0;
    loop {
        let char = text.next();
        if char.is_none() {
            break;
        }
        let char = char.unwrap();
        let width = unicode_width::UnicodeWidthStr::width(char);
        if final_width + width > l as usize {
            res.push_str("...");
            break;
        }
        final_width += width;
        res.push_str(char);
    }

    res
}
