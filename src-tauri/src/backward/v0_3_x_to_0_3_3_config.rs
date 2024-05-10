use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_logging::LogLevelFilter;

use crate::{config::Config, error};

/// the config struct before version 0.3.3
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigBefore {
    pub clip_per_page: u64,
    pub clip_max_show_length: u64,
    pub search_clip_per_page: u64,
    pub log_level: LogLevelFilter,
}

pub fn upgrade(app: &AppHandle) -> Result<(), error::Error> {
    // find the config file
    let data_dir = app.path_resolver().app_data_dir();
    if data_dir.is_none() {
        return Err(error::Error::GetAppDataDirErr);
    }

    let data_dir = data_dir.unwrap();
    let mut config_file = data_dir.clone();
    config_file.push("config.json");

    // test if data_dir exist
    let data_dir_exist = data_dir.try_exists();
    if data_dir_exist.is_err() {
        return Err(error::Error::GetAppDataDirErr);
    }

    let data_dir_exist = data_dir_exist.unwrap();
    if !data_dir_exist {
        // app data dir does not exist, no need to upgrade
        // this should not happen, as database is also in the app data dir, and this function is called after database related upgrade
        return Ok(());
    }

    // test if config_file exist
    let config_file_exist = config_file.try_exists();
    if let Err(err) = config_file_exist {
        return Err(error::Error::GetConfigFilePathErr(format!(
            "can not verify existence of config file: {}",
            err
        )));
    }

    let config_file_exist = config_file_exist.unwrap();
    if !config_file_exist {
        // config file does not exist, no need to upgrade
        return Ok(());
    }

    // config file exist, load it
    let config_file_content = std::fs::read_to_string(config_file.as_path());
    if let Err(err) = config_file_content {
        return Err(error::Error::GetConfigFilePathErr(format!(
            "can not read config file: {}",
            err
        )));
    }

    let config_file_content = config_file_content.unwrap();
    let config_before = serde_json::from_str::<ConfigBefore>(&config_file_content);
    let mut config = Config::default();
    if config_before.is_err() {
        // can not parse the config file, use default config
        // and save the default config to the config file
        let config_json = serde_json::to_string(&config);
        if let Err(err) = config_json {
            return Err(error::Error::SerializeConfigToJsonErr(format!(
                "can not serialize default config: {}",
                err
            )));
        }

        let config_json = config_json.unwrap();
        let write_config = std::fs::write(config_file.as_path(), config_json);
        if let Err(err) = write_config {
            return Err(error::Error::WriteConfigFileErr(format!(
                "can not write default config to config file: {}",
                err
            )));
        }

        return Ok(());
    }

    let config_before = config_before.unwrap();
    config.clip_per_page = config_before.clip_per_page;
    config.clip_max_show_length = config_before.clip_max_show_length;
    config.search_clip_per_batch = config_before.search_clip_per_page;
    config.log_level = config_before.log_level;

    // save the config to the config file
    let config_json = serde_json::to_string(&config);
    if let Err(err) = config_json {
        return Err(error::Error::SerializeConfigToJsonErr(format!(
            "can not serialize default config: {}",
            err
        )));
    }

    let config_json = config_json.unwrap();
    let write_config = std::fs::write(config_file.as_path(), config_json);
    if let Err(err) = write_config {
        return Err(error::Error::WriteConfigFileErr(format!(
            "can not write default config to config file: {}",
            err
        )));
    }

    Ok(())
}
