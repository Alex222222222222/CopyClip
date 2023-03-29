/// this is the package containing all the related error types
/// some of them contain error message and some of them contain additional information
///
// Path: src-tauri/src/error.rs
use std::fmt;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum Error {
    /// clip not found in the database
    /// the first i64 is the id of the clip
    ClipNotFoundErr(i64),
    /// failed to create the app data dir
    /// the error message is the error message from the std::fs::create_dir_all
    CreateAppDataDirErr(String),
    /// failed to create clips table
    /// error message from sqlite::statement::next
    CreateClipsTableErr(String),
    /// failed to create version table
    /// error message from sqlite::statement::next
    CreateVersionTableErr(String),
    /// the database connection is none
    /// this error should not happen
    /// if it happens, it means the database connection is not initialized
    /// or the database connection is dropped
    DatabaseConnectionErr,
    /// delete clip from the database failed
    /// the error message is the error message from the sqlite::execute
    /// the first i64 is the id of the clip, the second string is the error message
    DeleteClipFromDatabaseErr(i64, String),
    /// error occurred when exporting data
    /// the first string is the error message
    ExportError(String),
    /// failed to get the app data dir, using tauri path resolver
    GetAppDataDirErr,
    /// get clip data from the database failed
    /// the error message is the error message from the sqlite::query_row
    /// the first i64 is the id of the clip, the second string is the error message
    GetClipDataFromDatabaseErr(i64, String),
    /// get config file path error
    /// the first string is the error message
    GetConfigFilePathErr(String),
    /// failed to get version from the database
    /// the error message is the error message from the sqlite::query_row
    GetVersionFromDatabaseErr(String),
    /// failed to get current version from tauri
    GetVersionFromTauriErr,
    /// failed to read whole ids list from the database
    /// error message from sqlite::query_row
    GetWholeIdsErr(String),
    /// failed to insert new clip to the database
    /// the error message is the error message from the sqlite::execute
    /// the first string is the clip data, the second string is the error message
    InsertClipIntoDatabaseErr(String, String),
    /// failed to insert version to the database
    /// the error message is the error message from the sqlite::execute
    /// the version is the version that failed to insert
    /// the first string is the version, the second string is the error message
    InsertVersionErr(String, String),
    /// get empty or invalid id from the whole ids list
    /// the first string is the id given by the whole ids list
    InvalidIDFromWholeListErr(Option<i64>),
    /// failed to open the database
    /// the error message is the error message from the sqlite::open
    OpenDatabaseErr(String),
    /// read from system clipboard failed
    /// the first string is the error message
    ReadFromSystemClipboardErr(String),
    /// invalid regexp string
    RegexpErr(String),
    /// serialize config to json error
    /// the first string is the error message
    SerializeConfigToJsonErr(String),
    /// failed to set system tray title
    /// the first string is the title, the second string is the error message
    SetSystemTrayTitleErr(String, String),
    /// unknown error
    Unknown,
    /// update clips in database failed
    /// the first string is the error message, the second string is the error message from the sqlite::execute
    UpdateClipsInDatabaseErr(String, String),
    /// the whole ids list is empty
    WholeListIDSEmptyErr,
    /// failed to write config file to the disk
    /// the first string is the error message from the std::fs::write
    WriteConfigFileErr(String),
    /// failed to write to system clipboard
    /// the string is the text that failed to write to the system clipboard, the second string is the error message
    WriteToSystemClipboardErr(String, String),
}

impl Error {
    /// return the error message
    pub fn message(&self) -> String {
        match self {
            Error::Unknown => "unknown error".to_string(),
            Error::GetAppDataDirErr => "failed to get the app data dir, using tauri path resolver".to_string(),
            Error::CreateAppDataDirErr(err) => format!("failed to create the app data dir, error message: {err}"),
            Error::OpenDatabaseErr(err) => format!("failed to open the database, error message: {err}"),
            Error::GetVersionFromDatabaseErr(err) => format!("failed to get version from the database, error message: {err}"),
            Error::GetVersionFromTauriErr => "failed to get current version from tauri".to_string(),
            Error::InsertVersionErr(version, err) => format!("failed to insert version to the database, version: {version}, error message: {err}"),
            Error::CreateVersionTableErr(err) => format!("failed to create version table, error message: {err}"),
            Error::GetWholeIdsErr(err) => format!("failed to read whole ids list from the database, error message: {err}"),
            Error::CreateClipsTableErr(err) => format!("failed to create clips table, error message: {err}"),
            Error::WholeListIDSEmptyErr => "the whole ids list is empty".to_string(),
            Error::InvalidIDFromWholeListErr(id) => format!("get empty or invalid id from the whole ids list, id: {id:?}"),
            Error::GetClipDataFromDatabaseErr(id, err) => format!("get clip data from the database failed, id: {id}, error message: {err}"),
            Error::ClipNotFoundErr(id) => format!("clip not found in the database, id: {id}"),
            Error::DeleteClipFromDatabaseErr(id, err) => format!("delete clip from the database failed, id: {id}, error message: {err}"),
            Error::DatabaseConnectionErr => "the database connection is none".to_string(),
            Error::InsertClipIntoDatabaseErr(clip, err) => format!("failed to insert new clip to the database, clip data: {clip}, error message: {err}"),
            Error::WriteToSystemClipboardErr(clip, err) => format!("failed to write to system clipboard, clip data: {clip}, error message: {err}"),
            Error::SetSystemTrayTitleErr(title, err) => format!("failed to set system tray title, title: {title}, error message: {err}"),
            Error::GetConfigFilePathErr(err) => format!("get config file path error, error message: {err}"),
            Error::SerializeConfigToJsonErr(err) => format!("serialize config to json error, error message: {err}"),
            Error::WriteConfigFileErr(err) => format!("failed to write config file to the disk, error message: {err}"),
            Error::UpdateClipsInDatabaseErr(err, err2) => format!("update clips in database failed, error message: {err}, error message from sqlite::execute: {err2}"),
            Error::RegexpErr(err) => format!("invalid regexp string, error message: {err}"),
            Error::ReadFromSystemClipboardErr(err) => format!("read from system clipboard failed, error message: {err}"),
            Error::ExportError(err) => format!("error occurred when exporting data, error message: {err}"),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
