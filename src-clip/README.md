# Clip Library

This library provides data type and related functions for working with clips.

## Clip

There are five types of clips:

```rust
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize, Default)]
pub enum ClipType {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
    /// file path
    ///
    /// json encoded vec<FileURI>
    #[serde(rename = "file")]
    File,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "rtf")]
    Rtf,
}
```

A clip is like below.

If the feature `yew` is enabled, the clip will derive `yew::Properties` and `PartialEq`
which is used to support passing clip and storing clip in frontend.

```rust
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
```

## Features

Default features:
- `compress`
- `search`

### yew

The `yew` feature is used to derive `yew::Properties` and `PartialEq` for `Clip`.

### Compress

The `compress` feature is used to compress the clip text when serializing and deserializing.
This is used to reduce the size of the clip when stored in the database.
API:

```rust
pub fn compress_text(text: &str) -> Result<Vec<u8>, String>

pub fn decompress_text(data: &[u8]) -> Result<String, String>
```

### Search

The `search` feature is used to provide a method to turn clips with types other than `Text` into searchable text.

This feature needs to be initialized by calling `init_search()` before using the API.
As we are using [ocrs](https://github.com/robertknight/ocrs), which requires a trained model, the model should be downloaded and path the path as the parameter of `init_search()`.

```
pub fn init_search<T>(detection_model_path: T, recognition_model_path: T) -> Result<(), String>
where
    T: AsRef<path::Path>,
```

The exported API is as below:

```rust
pub fn convert_text_to_search_text(clip_type: ClipType, text: &str) -> Result<String, String>

pub fn convert_clip_to_search_text(clip: &Clip) -> Result<String, String>
```
