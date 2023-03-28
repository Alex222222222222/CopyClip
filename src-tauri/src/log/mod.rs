use std::{fmt::Display, path::PathBuf};

use log::error;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::config::ConfigMutex;

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

pub async fn setup_logger(app: &AppHandle) -> Result<(), fern::InitError> {
    let path = get_user_log_path(app);
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

    let level = get_user_log_level(app).await;
    log = log.level(level);

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
fn get_user_log_path(app: &AppHandle) -> Option<PathBuf> {
    let log_dir = app.path_resolver().app_log_dir();
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
pub async fn get_user_log_level(app: &AppHandle) -> log::LevelFilter {
    let config = app.state::<ConfigMutex>();
    let config = config.config.lock().await;
    let log_level = config.log_level.clone();
    log::LevelFilter::from(log_level)
}

/// get human readable time with millisecond with timezone
fn get_time() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S%.3f %z %Z").to_string()
}

pub fn panic_app(msg: &str) {
    error!("{}", msg);
    panic!("{}", msg);
}
