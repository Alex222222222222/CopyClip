use std::{fmt::Display, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::PathResolver;

use log::warn;

/// the config struct that only load log_level
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogLevelConfig {
    /// log level
    pub log_level: LogLevelFilter,
}

#[derive(Clone, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<log::Level> for LogLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Trace => LogLevel::Trace,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Info => LogLevel::Info,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Error => LogLevel::Error,
        }
    }
}

impl From<LogLevel> for log::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => log::Level::Trace,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum LogLevelFilter {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl From<log::LevelFilter> for LogLevelFilter {
    fn from(level: log::LevelFilter) -> Self {
        match level {
            log::LevelFilter::Trace => LogLevelFilter::Trace,
            log::LevelFilter::Debug => LogLevelFilter::Debug,
            log::LevelFilter::Info => LogLevelFilter::Info,
            log::LevelFilter::Warn => LogLevelFilter::Warn,
            log::LevelFilter::Error => LogLevelFilter::Error,
            log::LevelFilter::Off => LogLevelFilter::Off,
        }
    }
}

impl From<LogLevelFilter> for log::LevelFilter {
    fn from(level: LogLevelFilter) -> Self {
        match level {
            LogLevelFilter::Trace => log::LevelFilter::Trace,
            LogLevelFilter::Debug => log::LevelFilter::Debug,
            LogLevelFilter::Info => log::LevelFilter::Info,
            LogLevelFilter::Warn => log::LevelFilter::Warn,
            LogLevelFilter::Error => log::LevelFilter::Error,
            LogLevelFilter::Off => log::LevelFilter::Off,
        }
    }
}

impl Display for LogLevelFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevelFilter::Trace => write!(f, "trace"),
            LogLevelFilter::Debug => write!(f, "debug"),
            LogLevelFilter::Info => write!(f, "info"),
            LogLevelFilter::Warn => write!(f, "warn"),
            LogLevelFilter::Error => write!(f, "error"),
            LogLevelFilter::Off => write!(f, "off"),
        }
    }
}

impl From<String> for LogLevelFilter {
    fn from(level: String) -> Self {
        match level.as_str() {
            "trace" => LogLevelFilter::Trace,
            "debug" => LogLevelFilter::Debug,
            "info" => LogLevelFilter::Info,
            "warn" => LogLevelFilter::Warn,
            "error" => LogLevelFilter::Error,
            "off" => LogLevelFilter::Off,
            _ => LogLevelFilter::Info,
        }
    }
}

pub fn setup_logger(path_resolver: &PathResolver) -> Result<(), fern::InitError> {
    let path = get_user_log_path(path_resolver);
    if path.is_none() {
        return Ok(());
    }
    let path = path.unwrap();

    let mut log = fern::Dispatch::new().format(|out, message, record| {
        out.finish(format_args!(
            "[{} {} {}] {}",
            get_time(),
            record.level(),
            record.target(),
            message
        ))
    });

    // at this stage the user config is still not loaded, so we needed to manually get the config.
    // user config is loaded after the database config to prevent issue with backward compatibility
    // however, we only need the log level, so there should be no issue with the backward compatibility
    let level = get_user_log_level(path_resolver);
    log = log.level(level);

    #[cfg(debug_assertions)]
    println!("log level is {}", level);

    #[cfg(debug_assertions)]
    {
        println!("debug mode, log level is trace");
        log = log.level(log::LevelFilter::Trace).chain(std::io::stdout());
    }

    log = log.chain(fern::log_file(path)?);

    log.apply()?;

    Ok(())
}

/// get the app log path
pub fn get_user_log_path(path_resolver: &PathResolver) -> Option<PathBuf> {
    let log_dir = path_resolver.app_log_dir();
    let mut log_dir = log_dir?;

    // test if log_dir exist
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).unwrap();
    }

    log_dir.push("log");

    // test if log file exist
    if !log_dir.exists() {
        std::fs::File::create(&log_dir).unwrap();
    }

    Some(log_dir)
}

/// get the user log level
fn get_user_log_level(path_resolver: &PathResolver) -> log::LevelFilter {
    // warn in this functions will not go to the correct place as the logger is not yet setup

    let data_dir = path_resolver.app_data_dir();
    if data_dir.is_none() {
        warn!("can not find app data dir");
        return log::LevelFilter::Info;
    }

    let data_dir = data_dir.unwrap();
    let mut config_file = data_dir.clone();
    config_file.push("config.json");

    // test if data_dir exist
    let data_dir_exist = data_dir.try_exists();
    if data_dir_exist.is_err() {
        warn!("can not verify existence of app data dir");
        return log::LevelFilter::Info;
    }

    let data_dir_exist = data_dir_exist.unwrap();
    if !data_dir_exist {
        // create the data_dir
        let create_data_dir = fs::create_dir(data_dir.as_path());
        if create_data_dir.is_err() {
            warn!("can not create app data dir");
            return log::LevelFilter::Info;
        }
    }

    // test if config_file exist
    let config_file_exist = config_file.try_exists();
    if config_file_exist.is_err() {
        warn!("can not verify existence of config file");
        return log::LevelFilter::Info;
    }

    let config_file_exist = config_file_exist.unwrap();

    if !config_file_exist {
        return log::LevelFilter::Info;
    }

    // config file exist, load it
    let read_config_file = fs::read_to_string(config_file.as_path());
    if read_config_file.is_err() {
        warn!("can not read config file");
        return log::LevelFilter::Info;
    }

    let read_config_file = read_config_file.unwrap();

    let c_json: Result<LogLevelConfig, serde_json::Error> = serde_json::from_str(&read_config_file);
    if c_json.is_err() {
        warn!("can not serialize config to json");
        return log::LevelFilter::Info;
    }

    if let Ok(c_json) = c_json {
        return c_json.log_level.into();
    }

    log::LevelFilter::Info
}

/// get human readable time with millisecond with timezone
fn get_time() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S%.3f %z %Z").to_string()
}
