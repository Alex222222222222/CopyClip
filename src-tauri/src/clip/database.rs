use sqlite::{Connection, State};
use tauri::{AppHandle, Manager};

use crate::error;

use super::ClipDataMutex;

pub fn init_database_connection(app: &AppHandle) -> Result<(), error::Error> {
    // get the app data dir
    let app_data_dir = get_and_create_app_data_dir(app);
    let app_data_dir = app_data_dir?;

    // create the database dir if it does not exist
    let connection = get_and_create_database(app_data_dir);
    let connection = connection?;

    // init the version of the database
    let version = init_version_table(&connection, app);
    version?;

    // init the clips table
    init_clips_table(connection, app)
}

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

fn get_current_version(app: &AppHandle) -> Option<String> {
    app.config().package.version.clone()
}

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
                // if it is the first time the app is launched, insert the current version
                let current_version = get_current_version(app);
                if current_version.is_none() {
                    return Err(error::Error::GetVersionFromTauriErr);
                }
                let current_version = current_version.unwrap();

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
                    Ok(State::Done) => {
                        return Ok(());
                    }
                    Ok(State::Row) => {
                        return Err(error::Error::InsertVersionErr(
                            current_version,
                            "Failed to insert version".to_string(),
                        ));
                    }
                    Err(err) => {
                        return Err(error::Error::InsertVersionErr(
                            current_version,
                            err.to_string(),
                        ));
                    }
                }
            } else {
                // if it is not the first time the app is launched, check if the save version is the same as the current version
                let current_version = get_current_version(app);
                if current_version.is_none() {
                    return Err(error::Error::GetVersionFromTauriErr);
                }
                let current_version = current_version.unwrap();

                if save_version != current_version {
                    // if the save version is not the same as the current version, deal with the backward comparability
                    let backward_comparability = backward_comparability(app, save_version);
                    backward_comparability?;

                    // update the version
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
                        Ok(State::Done) => {
                            return Ok(());
                        }
                        Ok(State::Row) => {
                            return Err(error::Error::InsertVersionErr(
                                current_version,
                                "Failed to insert version".to_string(),
                            ));
                        }
                        Err(err) => {
                            return Err(error::Error::InsertVersionErr(
                                current_version,
                                err.to_string(),
                            ));
                        }
                    }
                }
            }

            Ok(())
        }
        Ok(State::Row) => Err(error::Error::CreateVersionTableErr(
            "Failed to create version table".to_string(),
        )),
        Err(err) => Err(error::Error::CreateVersionTableErr(err.to_string())),
    }
}

fn backward_comparability(_app: &AppHandle, _save_version: String) -> Result<(), error::Error> {
    // deal with the backward comparability based on the save version

    Ok(())
}

fn init_clips_table(connection: Connection, app: &AppHandle) -> Result<(), error::Error> {
    // create the clips table if it does not exist
    let mut statement = connection.prepare("CREATE TABLE IF NOT EXISTS clips (id INTEGER PRIMARY KEY AUTOINCREMENT, text TEXT, timestamp INTEGER, favorite INTEGER)").unwrap();
    let state = statement.next();
    match state {
        Ok(State::Done) => {
            let clip_data_mutex = app.state::<ClipDataMutex>();
            let mut clip_data = clip_data_mutex.clip_data.lock().unwrap();
            drop(statement);
            clip_data.database_connection = Some(connection);

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
        Ok(State::Row) => Err(error::Error::CreateClipsTableErr(
            "Failed to create clips table".to_string(),
        )),
        Err(err) => Err(error::Error::CreateClipsTableErr(err.to_string())),
    }
}
