use std::{fs, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error;

pub mod command;

/// the config struct
pub struct ConfigMutex {
    pub config: Mutex<Config>,
}

/// the config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// the number of clips to show in the tray menu
    pub clip_per_page: i64,
    /// the max length of a clip to show in the tray menu
    pub clip_max_show_length: i64,
    /// clip per page in the search page
    pub search_clip_per_page: i64,
}

// TODO save config

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
    if data_dir.is_none() {
        // TODO log error
        return Config::default();
    }

    let data_dir = data_dir.unwrap();
    let mut config_file = data_dir.clone();
    config_file.push("config.json");

    // test if data_dir exist
    let data_dir_exist = data_dir.try_exists();
    if data_dir_exist.is_err() {
        // TODO log error
        return Config::default();
    }

    let data_dir_exist = data_dir_exist.unwrap();
    if !data_dir_exist {
        // create the data_dir
        let create_data_dir = fs::create_dir(data_dir.as_path());
        if create_data_dir.is_err() {
            // TODO log error
            return Config::default();
        }
    }

    // test if config_file exist
    let config_file_exist = config_file.try_exists();
    if config_file_exist.is_err() {
        // TODO log error
        return Config::default();
    }

    let config_file_exist = config_file_exist.unwrap();

    if !config_file_exist {
        let c = Config::default();
        let c_json = serde_json::to_string(&c);

        if c_json.is_err() {
            // TODO log error
            return Config::default();
        }

        let c_json = c_json.unwrap();

        let write_config_file = fs::write(config_file.as_path(), c_json);
        if write_config_file.is_err() {
            // TODO log error
            return Config::default();
        }

        return c;
    }

    // config file exist, load it
    let read_config_file = fs::read_to_string(config_file.as_path());
    if read_config_file.is_err() {
        // TODO log error
        return Config::default();
    }

    let read_config_file = read_config_file.unwrap();

    let c_json: Result<Config, serde_json::Error> = serde_json::from_str(&read_config_file);
    if c_json.is_err() {
        let c = Config::default();
        let c_json = serde_json::to_string(&c);

        if c_json.is_err() {
            // TODO log error
            return Config::default();
        }

        let c_json = c_json.unwrap();

        let write_config_file = fs::write(config_file.as_path(), c_json);
        if write_config_file.is_err() {
            // TODO log error
            return Config::default();
        }

        return c;
    }

    c_json.unwrap()
}

impl Config {
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

        let c_json = serde_json::to_string(&self);
        if let Err(e) = c_json {
            return Err(error::Error::SerializeConfigToJsonErr(e.to_string()));
        }

        let c_json = c_json.unwrap();

        // write the config file
        let write_config_file = fs::write(config_file.as_path(), c_json);
        if let Err(e) = write_config_file {
            return Err(error::Error::WriteConfigFileErr(e.to_string()));
        }

        Ok(())
    }
}

/// the default config
impl Default for Config {
    fn default() -> Self {
        Self {
            clip_per_page: 20,
            clip_max_show_length: 50,
            search_clip_per_page: 20,
        }
    }
}
