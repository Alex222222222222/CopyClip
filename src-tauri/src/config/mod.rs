use std::fs;

use log::warn;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::Mutex, AppHandle};
use tauri_plugin_logging::LogLevelFilter;

use crate::error;

pub mod command;

fn default_clip_per_page() -> i64 {
    20
}
fn default_clip_max_show_length() -> i64 {
    50
}
fn default_search_clip_per_batch() -> i64 {
    2
}
fn default_log_level() -> LogLevelFilter {
    LogLevelFilter::Info
}
fn default_dark_mode() -> bool {
    false
}
fn default_language() -> String {
    "en-GB".to_string()
}
fn default_auto_delete_duplicate_clip() -> bool {
    true
}
fn default_pause_monitoring() -> bool {
    false
}

/// the config struct
pub struct ConfigMutex {
    pub config: Mutex<Config>,
}

/// the config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// the number of clips to show in the tray menu
    #[serde(default = "default_clip_per_page")]
    pub clip_per_page: i64,
    /// the max length of a clip to show in the tray menu
    #[serde(default = "default_clip_max_show_length")]
    pub clip_max_show_length: i64,
    /// clip max show length in search page
    /// default 500
    #[serde(default = "default_search_clip_per_batch")]
    pub search_page_clip_max_show_length: i64,
    /// the number of clips to search in one batch
    #[serde(default = "default_search_clip_per_batch")]
    pub search_clip_per_batch: i64,
    /// log level
    #[serde(default = "default_log_level")]
    pub log_level: LogLevelFilter,
    /// dark mode
    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,
    /// user defined language
    #[serde(default = "default_language")]
    pub language: String,
    /// enable the feature to auto delete duplicate clips when insert new clip
    #[serde(default = "default_auto_delete_duplicate_clip")]
    pub auto_delete_duplicate_clip: bool,
    /// Whether the monitoring of the clipboard is paused.
    /// Allow user to pause monitoring without exiting the app.
    #[serde(default = "default_pause_monitoring")]
    pub pause_monitoring: bool,
}

/// the default config
impl Default for Config {
    fn default() -> Self {
        Self {
            clip_per_page: default_clip_per_page(),
            clip_max_show_length: default_clip_max_show_length(),
            search_clip_per_batch: default_search_clip_per_batch(),
            log_level: default_log_level(),
            dark_mode: default_dark_mode(),
            search_page_clip_max_show_length: 500,
            language: default_language(),
            auto_delete_duplicate_clip: default_auto_delete_duplicate_clip(),
            pause_monitoring: default_pause_monitoring(),
        }
    }
}

/// load the config from app data folder
/// if the config file does not exist, create it
/// if the config file exist, load it
///
/// if there is an error, return the default config
pub fn load_config(app: &AppHandle) -> Config {
    // find app data folder
    // find config file
    // if config file does not exist, create it
    // if config file exists, load it

    let data_dir = app.path_resolver().app_data_dir();
    let data_dir = match data_dir {
        Some(d) => d,
        None => {
            warn!("can not find app data dir");
            return Config::default();
        }
    };
    let mut config_file = data_dir.clone();
    config_file.push("config.json");

    // test if data_dir exist
    let data_dir_exist = data_dir.try_exists();
    let data_dir_exist = match data_dir_exist {
        Ok(d) => d,
        Err(e) => {
            warn!("can not verify existence of app data dir: {}", e);
            return Config::default();
        }
    };
    if !data_dir_exist {
        // create the data_dir
        let create_data_dir = fs::create_dir(data_dir.as_path());
        if create_data_dir.is_err() {
            warn!("can not create app data dir");
            return Config::default();
        }
    }

    // test if config_file exist
    let config_file_exist = config_file.try_exists();
    let config_file_exist = match config_file_exist {
        Ok(c) => c,
        Err(e) => {
            warn!("can not verify existence of config file: {}", e);
            return Config::default();
        }
    };
    if !config_file_exist {
        let c = Config::default();
        let c_json = serde_json::to_string(&c);
        let c_json = match c_json {
            Ok(c) => c,
            Err(e) => {
                warn!("can not serialize config to json: {}", e);
                return Config::default();
            }
        };
        let write_config_file = fs::write(config_file.as_path(), c_json);
        if write_config_file.is_err() {
            warn!("can not write config file");
            return Config::default();
        }

        return c;
    }

    // config file exist, load it
    let read_config_file = fs::read_to_string(config_file.as_path());
    if read_config_file.is_err() {
        warn!("can not read config file");
        return Config::default();
    }

    let read_config_file = read_config_file.unwrap();

    let c_json: Result<Config, serde_json::Error> = serde_json::from_str(&read_config_file);
    if c_json.is_err() {
        let c = Config::default();
        let c_json = serde_json::to_string(&c);

        if c_json.is_err() {
            warn!("can not serialize config to json");
            return Config::default();
        }

        let c_json = c_json.unwrap();

        let write_config_file = fs::write(config_file.as_path(), c_json);
        if write_config_file.is_err() {
            warn!("can not write config file");
            return Config::default();
        }

        return c;
    }

    c_json.unwrap()
}

impl Config {
    /// convert the config to json string
    pub fn to_json(&self) -> Result<String, error::Error> {
        let c_json = serde_json::to_string(&self);
        if let Err(e) = c_json {
            return Err(error::Error::SerializeConfigToJsonErr(e.to_string()));
        }

        Ok(c_json.unwrap())
    }

    /// save the config to the config file
    /// if the data dir does not exist, create it
    /// if the config file does not exist, create it
    /// if the config file exist, overwrite it
    pub fn save_config(&self, app: &AppHandle) -> Result<(), error::Error> {
        let data_dir = app.path_resolver().app_data_dir();
        if data_dir.is_none() {
            return Err(error::Error::GetAppDataDirErr);
        }
        let data_dir = data_dir.unwrap();

        let mut config_file = data_dir.clone();
        config_file.push("config.json");

        // test if data_dir exist
        if !data_dir.exists() {
            // create the data_dir
            let create_data_dir = fs::create_dir(data_dir.as_path());
            if let Err(e) = create_data_dir {
                return Err(error::Error::CreateAppDataDirErr(e.to_string()));
            }
        }

        let c_json = self.to_json()?;

        // write the config file
        let write_config_file = fs::write(config_file.as_path(), c_json);
        if let Err(e) = write_config_file {
            return Err(error::Error::WriteConfigFileErr(e.to_string()));
        }

        Ok(())
    }

    pub fn load_config(&mut self, app: &AppHandle) {
        let config = load_config(app);
        self.clip_per_page = if config.clip_per_page > 50 {
            50
        } else if config.clip_per_page < 1 {
            1
        } else {
            config.clip_per_page
        };
        self.clip_max_show_length = config.clip_max_show_length;
        self.search_clip_per_batch = config.search_clip_per_batch;
        self.log_level = config.log_level;
        self.dark_mode = config.dark_mode;
        self.search_page_clip_max_show_length = config.search_page_clip_max_show_length;
        self.language = config.language;
        self.auto_delete_duplicate_clip = config.auto_delete_duplicate_clip;
        self.pause_monitoring = config.pause_monitoring;
    }
}
