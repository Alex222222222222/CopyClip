use std::{sync::Arc, time::Duration};

use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use tauri::{AppHandle, Manager};

use crate::{backward, error};

use super::ClipDataMutex;

/// init the database connection and create the table
/// also init the clips mutex
pub async fn init_database_connection(app: &AppHandle) -> Result<(), error::Error> {
    // get the app data dir
    let app_data_dir = get_and_create_app_data_dir(app);
    let app_data_dir = app_data_dir?;

    // create the database dir if it does not exist
    let connection = get_and_create_database(app_data_dir.clone(), 1).await?;

    // init the version of the database
    init_version_table(&connection, app).await?;

    // init the clips table
    init_clips_table(&connection).await?;

    // init the pinned clips table
    init_pinned_clips_table(&connection).await?;

    // after init the database, recreate a connection with higher connection number
    let connection = get_and_create_database(app_data_dir, 10).await?;
    // init the clips mutex
    init_clips_mutex(connection, app).await
}

/// get the app data dir and create it if it does not exist
fn get_and_create_app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, error::Error> {
    // get the app data dir
    let app_data_dir = app.path_resolver().app_data_dir();
    if app_data_dir.is_none() {
        return Err(error::Error::GetAppDataDirErr);
    }
    let app_data_dir = app_data_dir.unwrap();

    // if the app data dir does not exist, create it
    if !app_data_dir.exists() {
        if let Err(err) = std::fs::create_dir_all(app_data_dir.as_path()) {
            return Err(error::Error::CreateAppDataDirErr(err.to_string()));
        }
    }

    Ok(app_data_dir)
}

/// get the database connection and create it if it does not exist
async fn get_and_create_database(
    app_data_dir: std::path::PathBuf,
    connection_num: u32,
) -> Result<SqlitePool, error::Error> {
    // create the database dir if it does not exist
    let database_path = app_data_dir.join("database");

    // test if database_path exist
    if !database_path.exists() {
        // create database file
        if let Err(err) = std::fs::File::create(database_path.as_path()) {
            return Err(error::Error::OpenDatabaseErr(err.to_string()));
        }
    }

    let connection = SqlitePoolOptions::new()
        .max_connections(connection_num)
        .idle_timeout(Duration::from_secs(1))
        .connect_lazy(database_path.to_str().unwrap());
    if let Err(err) = connection {
        return Err(error::Error::OpenDatabaseErr(err.to_string()));
    }

    let connection = connection.unwrap();

    Ok(connection)
}

/// get the current version from the tauri config
fn get_current_version(app: &AppHandle) -> Result<String, error::Error> {
    let current_version = app.config().package.version.clone();
    if current_version.is_none() {
        return Err(error::Error::GetVersionFromTauriErr);
    }
    Ok(current_version.unwrap())
}

/// get the save version from the database
/// if there is no version, it is the first time the app is launched and return is 0.0.0
///
/// this function does not test validity of the version in the database
async fn get_save_version(connection: &SqlitePool) -> Result<String, error::Error> {
    // get the latest version, if it exists
    // if not exists, it is the first time the app is launched return 0.0.0

    let res: Result<Option<(String,)>, sqlx::Error> =
        sqlx::query_as("SELECT version FROM version ORDER BY id DESC LIMIT 1")
            .fetch_optional(connection)
            .await;
    if let Err(err) = res {
        return Err(error::Error::GetVersionFromDatabaseErr(err.to_string()));
    }
    let res = res.unwrap();
    if let Some(res) = res {
        Ok(res.0)
    } else {
        Ok("0.0.0".to_string())
    }
}

/// when the app is launched by the user for the first time, init the version table
///
/// this function will
///     - insert the current version into the version table
#[warn(unused_must_use)]
async fn first_lunch_the_version_table(
    connection: &SqlitePool,
    app: &AppHandle,
) -> Result<(), error::Error> {
    let current_version = get_current_version(app)?;

    let res = sqlx::query("INSERT INTO version (version) VALUES (?)")
        .bind(&current_version)
        .fetch_optional(connection)
        .await;

    if let Err(err) = res {
        return Err(error::Error::InsertVersionErr(
            current_version,
            err.to_string(),
        ));
    }

    Ok(())
}

/// this function will
///     - check if the save version is the same as the current version
///     - if not, trigger backward_compability and update the version table
///     - if yes, do nothing
#[warn(unused_must_use)]
async fn check_save_version_and_current_version(
    save_version: String,
    connection: &SqlitePool,
    app: &AppHandle,
) -> Result<(), error::Error> {
    let current_version = get_current_version(app)?;

    if current_version == save_version {
        return Ok(());
    }

    // if the save version is not the same as the current version, trigger the backward comparability
    let res = backward_comparability(app, connection, save_version);
    res.await?;

    // update the version table
    let res = sqlx::query("INSERT INTO version (version) VALUES (?)")
        .bind(&current_version)
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::InsertVersionErr(
            current_version,
            err.to_string(),
        ));
    }

    Ok(())
}

/// deal with the backward comparability based on the save version
#[warn(unused_must_use)]
async fn backward_comparability(
    app: &AppHandle,
    connection: &SqlitePool,
    save_version: String,
) -> Result<String, error::Error> {
    // get the three version number from save_version
    let version = save_version.split('.').collect::<Vec<&str>>();
    if version.len() != 3 {
        return Err(error::Error::GetVersionFromDatabaseErr(
            "The version number is not correct".to_string(),
        ));
    }
    // a version number is like [major].[minor].[patch]
    let major = version[0].parse::<i32>();
    let minor = version[1].parse::<i32>();
    let patch = version[2].parse::<i32>();

    if major.is_err() || minor.is_err() || patch.is_err() {
        return Err(error::Error::GetVersionFromDatabaseErr(
            "The version number is not correct".to_string(),
        ));
    }

    let major = major.unwrap();
    let mut minor = minor.unwrap();
    let mut patch = patch.unwrap();

    // deal with the backward comparability

    // if the major version is 0, the minor version is smaller than 3, need to upgrade the database to 0.3.0
    if major == 0 && minor < 3 {
        // upgrade the database to 0.3.0
        let res = backward::v0_2_x_to_0_3_0_database::upgrade(connection);
        res.await?;
        minor = 3;
        patch = 0;
    }

    // if the major version is 0, the minor version is 3, the patch version is smaller than 3, need to upgrade the config file to 0.3.3
    // before 0.3.3, there is search_clip_per_page in the config file, after 0.3.3, this entry changed to search_clip_per_batch
    if major == 0 && minor == 3 && patch < 3 {
        // upgrade the config file to 0.3.3
        let res = backward::v0_3_x_to_0_3_3_config::upgrade(app);
        res?;
        patch = 3;
    }

    // when moving from 0.3.3, 0.3.4, to 0.3.5,
    // I rename the column "favorite" to favourite in clips table, so need to do a sql ALTER command
    if major == 0 && minor == 3 && patch < 5 {
        backward::v0_3_x_to_0_3_5_database::upgrade(connection).await?;
        patch = 5;
    }

    // when moving fom 0.3.5, 0.3.6, to 0.3.7,
    // we need to update the pinned clips table
    if major == 0 && minor == 3 && patch < 7 {
        backward::v0_3_x_to_0_3_7_database::upgrade(connection).await?;
        patch = 7;
    }

    Ok(format!("{}.{}.{}", major, minor, patch))
}

/// init the version table
///
/// this function will
///     - create the version table if it does not exist
///     - get the save version
///     - if the save version is 0.0.0, it is the first time the app is launched, init the version table
///     - if the save version is not 0.0.0, check if the save version is the same as the current version
///     - if not, trigger backward_comparability and update the version table
#[warn(unused_must_use)]
async fn init_version_table(connection: &SqlitePool, app: &AppHandle) -> Result<(), error::Error> {
    // create the version table if it does not exist
    let res = sqlx::query(
        "CREATE TABLE IF NOT EXISTS version (id INTEGER PRIMARY KEY AUTOINCREMENT, version TEXT)",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::CreateVersionTableErr(err.to_string()));
    }

    // try get the save version
    let save_version = get_save_version(connection).await;
    let save_version = save_version?;

    if save_version == *"0.0.0" {
        first_lunch_the_version_table(connection, app).await?;
    } else {
        check_save_version_and_current_version(save_version, connection, app).await?;
    }

    Ok(())
}

/// init the clips table
///
/// this function will
///     - create the clips table if it does not exist
#[warn(unused_must_use)]
async fn init_clips_table(connection: &SqlitePool) -> Result<(), error::Error> {
    // create the clips table if it does not exist
    let res = sqlx::query(
        "CREATE VIRTUAL TABLE IF NOT EXISTS clips USING fts4(
            id INTEGER PRIMARY KEY,
            text TEXT,
            timestamp INTEGER, 
            favourite INTEGER,
        )",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::CreateClipsTableErr(err.to_string()));
    }
    Ok(())
}

// init the pinned clips table
//
// this function will
//     - create the pinned clips table if it does not exist
#[warn(unused_must_use)]
async fn init_pinned_clips_table(connection: &SqlitePool) -> Result<(), error::Error> {
    // create the pinned clips table if it does not exist
    let res = sqlx::query(
        "CREATE TABLE IF NOT EXISTS pinned_clips_new (
            id INTEGER PRIMARY KEY,
            text TEXT,
            timestamp INTEGER
        )",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::CreatePinnedClipsTableErr(err.to_string()));
    }
    Ok(())
}

/// init the clips mutex state
///
/// this function will
///     - fill the connection into the clips mutex
///     - init the whole list of ids
async fn init_clips_mutex(connection: SqlitePool, app: &AppHandle) -> Result<(), error::Error> {
    // fill the connection into the clips mutex
    let clip_data_mutex = app.state::<ClipDataMutex>();
    let mut clip_data = clip_data_mutex.clip_data.lock().await;
    clip_data.database_connection = Some(connection);
    drop(clip_data);

    // init the whole list of ids
    init_whole_list_of_ids(app).await?;

    // init pinned clips
    init_pinned_clips(app).await
}

/// init the whole list of ids,
///
/// this function will
///     - get the whole clips ids
///     - fill the ids into the clips mutex
async fn init_whole_list_of_ids(app: &AppHandle) -> Result<(), error::Error> {
    let clip_data_mutex = app.state::<ClipDataMutex>();
    let mut clip_data = clip_data_mutex.clip_data.lock().await;

    // get the whole clips ids
    let mut ids: Vec<i64> = Vec::new();
    let res = sqlx::query("SELECT id FROM clips")
        .fetch_all(clip_data.database_connection.as_ref().unwrap())
        .await;
    if let Err(err) = res {
        return Err(error::Error::GetWholeIdsErr(err.to_string()));
    }
    let res = res.unwrap();

    for row in res {
        let id = row.try_get::<i64, _>("id");
        if let Err(err) = id {
            return Err(error::Error::GetWholeIdsErr(err.to_string()));
        }
        ids.push(id.unwrap());
    }

    clip_data.clips.whole_list_of_ids = ids;

    Ok(())
}

/// init the pinned clips
///
/// this function will
///     - get the pinned clips from the database
///     - fill the pinned clips into the clips mutex
///     - the order is based on the timestamp
async fn init_pinned_clips(app: &AppHandle) -> Result<(), error::Error> {
    let clip_data_mutex = app.state::<ClipDataMutex>();
    let mut clip_data = clip_data_mutex.clip_data.lock().await;

    // get the pinned clips from the database
    let res = sqlx::query("SELECT * FROM pinned_clips ORDER BY timestamp DESC")
        .fetch_all(clip_data.database_connection.as_ref().unwrap())
        .await;
    if let Err(err) = res {
        return Err(error::Error::GetPinnedClipsErr(err.to_string()));
    }
    let res = res.unwrap();

    for row in res {
        let id = row.try_get::<i64, _>("id");
        if let Err(err) = id {
            return Err(error::Error::GetPinnedClipsErr(err.to_string()));
        }
        let text = row.try_get::<String, _>("text");
        if let Err(err) = text {
            return Err(error::Error::GetPinnedClipsErr(err.to_string()));
        }
        let timestamp = row.try_get::<i64, _>("timestamp");
        if let Err(err) = timestamp {
            return Err(error::Error::GetPinnedClipsErr(err.to_string()));
        }
        clip_data
            .clips
            .pinned_clips
            .push(super::pinned::PinnedClip {
                id: id.unwrap(),
                text: Arc::new(text.unwrap()),
                timestamp: timestamp.unwrap(),
            });
    }

    Ok(())
}

/// get all the versions from the database
/// in the format of Vec<id,String>
pub async fn get_all_versions(app: &AppHandle) -> Result<Vec<(i64, String)>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().await;
    let res = sqlx::query("SELECT * FROM version")
        .fetch_all(clip_data.database_connection.as_ref().unwrap())
        .await;
    if let Err(err) = res {
        return Err(error::Error::GetVersionFromDatabaseErr(err.to_string()));
    }
    let res = res.unwrap();
    if res.is_empty() {
        return Err(error::Error::GetVersionFromDatabaseErr(
            "the version table is empty".to_string(),
        ));
    }

    let mut versions = Vec::new();
    for row in res {
        let id = row.try_get::<i64, _>("id");
        if let Err(err) = id {
            return Err(error::Error::GetVersionFromDatabaseErr(err.to_string()));
        }
        let version = row.try_get::<String, _>("version");
        if let Err(err) = version {
            return Err(error::Error::GetVersionFromDatabaseErr(err.to_string()));
        }
        versions.push((id.unwrap(), version.unwrap()));
    }

    Ok(versions)
}
