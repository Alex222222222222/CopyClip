use sqlite::{Connection, State};
use tauri::{AppHandle, Manager};

use crate::{backward, error};

use super::ClipDataMutex;

/// init the database connection and create the table
/// also init the clips mutex
pub fn init_database_connection(app: &AppHandle) -> Result<(), error::Error> {
    // get the app data dir
    let app_data_dir = get_and_create_app_data_dir(app);
    let app_data_dir = app_data_dir?;

    // create the database dir if it does not exist
    let connection = get_and_create_database(app_data_dir)?;

    // init the version of the database
    init_version_table(&connection, app)?;

    // init the clips table
    init_clips_table(&connection)?;

    // init the clips mutex
    init_clips_mutex(connection, app)
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
fn get_and_create_database(app_data_dir: std::path::PathBuf) -> Result<Connection, error::Error> {
    // create the database dir if it does not exist
    let database_path = app_data_dir.join("database");

    let connection = sqlite::open(database_path.as_path());
    if connection.is_err() {
        return Err(error::Error::OpenDatabaseErr(
            connection.err().unwrap().to_string(),
        ));
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
fn get_save_version(connection: &Connection) -> Result<String, error::Error> {
    // get the latest version, if it exists
    // if not exists, it is the first time the app is launched return 0.0.0

    let mut statement = connection
        .prepare("SELECT version FROM version ORDER BY id DESC LIMIT 1")
        .unwrap();

    match statement.next() {
        Ok(State::Done) => {
            Ok("0.0.0".to_string()) // if there is no version, it is the first time the app is launched
        }
        Ok(State::Row) => {
            let version = statement.read::<String, _>("version");
            if version.is_err() {
                return Err(error::Error::GetVersionFromDatabaseErr(
                    version.err().unwrap().to_string(),
                ));
            }
            let version = version.unwrap();

            Ok(version)
        }
        Err(err) => Err(error::Error::GetVersionFromDatabaseErr(err.to_string())),
    }
}

/// when the app is launched by the user for the first time, init the version table
///
/// this function will
///     - insert the current version into the version table
fn first_lunch_the_version_table(
    connection: &Connection,
    app: &AppHandle,
) -> Result<(), error::Error> {
    let current_version = get_current_version(app)?;

    let mut statement = connection
        .prepare("INSERT INTO version (version) VALUES (?)")
        .unwrap();
    let res = statement.bind((1, current_version.as_str()));
    if res.is_err() {
        return Err(error::Error::InsertVersionErr(
            current_version,
            res.err().unwrap().to_string(),
        ));
    }

    match statement.next() {
        Ok(State::Done) => Ok(()),
        Ok(State::Row) => Err(error::Error::InsertVersionErr(
            current_version,
            "Failed to insert version".to_string(),
        )),
        Err(err) => Err(error::Error::InsertVersionErr(
            current_version,
            err.to_string(),
        )),
    }
}

/// this function will
///     - check if the save version is the same as the current version
///     - if not, trigger backward_comparability and update the version table
///     - if yes, do nothing
fn check_save_version_and_current_version(
    save_version: String,
    connection: &Connection,
    app: &AppHandle,
) -> Result<(), error::Error> {
    let current_version = get_current_version(app)?;

    if current_version == save_version {
        return Ok(());
    }

    // if the save version is not the same as the current version, trigger the backward comparability
    let res = backward_comparability(app, connection, save_version);
    res?;

    // update the version table
    let mut statement = connection
        .prepare("INSERT INTO version (version) VALUES (?)")
        .unwrap();
    let res = statement.bind((1, current_version.as_str()));
    if let Err(err) = res {
        return Err(error::Error::InsertVersionErr(
            current_version,
            err.to_string(),
        ));
    }
    match statement.next() {
        Ok(State::Done) => Ok(()),
        Ok(State::Row) => Err(error::Error::InsertVersionErr(
            current_version,
            "Failed to insert version".to_string(),
        )),
        Err(err) => Err(error::Error::InsertVersionErr(
            current_version,
            err.to_string(),
        )),
    }
}

/// deal with the backward comparability based on the save version
fn backward_comparability(
    app: &AppHandle,
    connection: &Connection,
    save_version: String,
) -> Result<String, error::Error> {
    // get the three version number from save_version
    let version = save_version.split('.').collect::<Vec<&str>>();
    if version.len() != 3 {
        return Err(error::Error::GetVersionFromDatabaseErr(
            "The version number is not correct".to_string(),
        ));
    }
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
        res?;
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
fn init_version_table(connection: &Connection, app: &AppHandle) -> Result<(), error::Error> {
    // create the version table if it does not exist
    let mut statement = connection.prepare("CREATE TABLE IF NOT EXISTS version (id INTEGER PRIMARY KEY AUTOINCREMENT, version TEXT)").unwrap();

    match statement.next() {
        Ok(State::Done) => {
            drop(statement);
            // try get the save version
            let save_version = get_save_version(connection);
            let save_version = save_version?;

            if save_version == *"0.0.0" {
                first_lunch_the_version_table(connection, app)?;
            } else {
                check_save_version_and_current_version(save_version, connection, app)?;
            }

            Ok(())
        }
        Ok(State::Row) => Err(error::Error::CreateVersionTableErr(
            "Failed to create version table".to_string(),
        )),
        Err(err) => Err(error::Error::CreateVersionTableErr(err.to_string())),
    }
}

/// init the clips table
///
/// this function will
///     - create the clips table if it does not exist
fn init_clips_table(connection: &Connection) -> Result<(), error::Error> {
    // create the clips table if it does not exist
    let mut statement = connection
        .prepare(
            "CREATE VIRTUAL TABLE IF NOT EXISTS clips USING fts4(
                id INTEGER PRIMARY KEY,
                text TEXT,
                timestamp INTEGER, 
                favorite INTEGER,
            )",
        )
        .unwrap();
    let state = statement.next();
    match state {
        Ok(State::Done) => Ok(()),
        Ok(State::Row) => Err(error::Error::CreateClipsTableErr(
            "Failed to create clips table".to_string(),
        )),
        Err(err) => Err(error::Error::CreateClipsTableErr(err.to_string())),
    }
}

/// init the clips mutex state
///
/// this function will
///     - fill the connection into the clips mutex
///     - init the whole list of ids
fn init_clips_mutex(connection: Connection, app: &AppHandle) -> Result<(), error::Error> {
    // fill the connection into the clips mutex
    let clip_data_mutex = app.state::<ClipDataMutex>();
    let mut clip_data = clip_data_mutex.clip_data.lock().unwrap();
    clip_data.database_connection = Some(connection);
    drop(clip_data);

    // init the whole list of ids
    init_whole_list_of_ids(app)
}

/// init the whole list of ids,
///
/// this function will
///     - get the whole clips ids
///     - fill the ids into the clips mutex
fn init_whole_list_of_ids(app: &AppHandle) -> Result<(), error::Error> {
    let clip_data_mutex = app.state::<ClipDataMutex>();
    let mut clip_data = clip_data_mutex.clip_data.lock().unwrap();

    // get the whole clips ids
    let mut ids = Vec::new();
    let mut statement = clip_data
        .database_connection
        .as_ref()
        .unwrap()
        .prepare("SELECT id FROM clips")
        .unwrap();

    loop {
        match statement.next() {
            Ok(State::Done) => {
                break;
            }
            Ok(State::Row) => {
                let id = statement.read::<i64, _>("id");

                if let Err(err) = id {
                    return Err(error::Error::GetWholeIdsErr(err.to_string()));
                }
                let id = id.unwrap();

                ids.push(id);
            }
            Err(err) => {
                return Err(error::Error::GetWholeIdsErr(err.to_string()));
            }
        }
    }
    drop(statement);
    clip_data.clips.whole_list_of_ids = ids;

    Ok(())
}
