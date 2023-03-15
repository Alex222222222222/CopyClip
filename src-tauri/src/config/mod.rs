use std::{sync::Mutex, fs};

use serde::{Deserialize, Serialize};
use tauri::App;

pub struct  ConfigMutex{
      pub config: Mutex<Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
      // do not know how to change this, as the way to control tauri system tray is really limited
      pub clips_to_show: i64,
      pub clip_max_show_length: i64,
}

pub fn load_config(app: &App) -> Config {
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

            return c
      }

      let c_json = c_json.unwrap();

      return c_json;
}

impl Default for Config{
      fn default() -> Self {
            Self {
                  clips_to_show: 10,
                  clip_max_show_length: 50,
            }
      }
}