use data_encoding::BASE32;
/// The database module is used to deal with the database connection and the database table
/// Database design:
///   - version table
///     - id INTEGER PRIMARY KEY
///     - version TEXT
///  - labels table
///     - used to store the labels
///     - name TEXT PRIMARY KEY
///     - the label table will at lease have two labels: "pinned" and "favourite"
///  - clips table
///     - id INTEGER PRIMARY KEY AUTOINCREMENT
///     - type INTEGER
///     - text TEXT
///     - timestamp INTEGER
///  - label_base32(label_name) tables for each label
///     - used to store the clips id for each label
///     - id INTEGER PRIMARY KEY foreign key to clips table
use log::debug;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::backward::backward_comparability;
use crate::error::Error;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// database state mutex
pub struct DatabaseStateMutex {
    pub database_connection: tauri::async_runtime::Mutex<Connection>,
}

impl Default for DatabaseStateMutex {
    fn default() -> Self {
        Self {
            database_connection: tauri::async_runtime::Mutex::new(
                // create a temporary database connection which will be replaced by the real connection
                Connection::open_in_memory().unwrap(),
            ),
        }
    }
}

impl DatabaseStateMutex {
    pub async fn set_database_connection(&self, connection: Connection) -> Result<(), Error> {
        let mut db_connection = self.database_connection.lock().await;
        *db_connection = connection;

        Ok(())
    }
}

/// init the database connection and create the table
/// also init the clips mutex
pub fn init_database_connection(app: &AppHandle) -> Result<Connection, Error> {
    // get the app data dir
    let app_data_dir = get_and_create_app_data_dir(app);
    let app_data_dir = app_data_dir?;

    // create the database dir if it does not exist
    let connection = get_and_create_database(app_data_dir.clone())?;

    // create the fuzzy search function
    create_fuzzy_search_function(&connection)?;

    // create the regexp function
    create_regexp_function(&connection)?;

    // init the version of the database
    // this will also deal with the backward comparability
    init_version_table(&connection, app)?;

    // init the clips table
    init_clips_table(&connection)?;

    // init the labels table
    init_labels_table(&connection)?;

    let res = connection.cache_flush();
    if let Err(err) = res {
        return Err(Error::DatabaseWriteErr(err.to_string()));
    }

    Ok(connection)
}

/// create and load the fuzzy search function to the database
fn create_fuzzy_search_function(connection: &Connection) -> Result<(), Error> {
    fn fuzzy_search(ctx: &rusqlite::functions::Context) -> Result<isize, rusqlite::Error> {
        let pattern = ctx.get::<String>(0)?;
        let text = ctx.get::<String>(1)?;

        match sublime_fuzzy::best_match(&pattern, &text) {
            Some(score) => Ok(score.score()),
            None => Ok(0),
        }
    }

    match connection.create_scalar_function(
        "fuzzy_search",
        2,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
        fuzzy_search,
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::DatabaseWriteErr(err.to_string())),
    }
}

/// create regex search function
fn create_regexp_function(db: &Connection) -> Result<(), Error> {
    fn regexp_match(ctx: &rusqlite::functions::Context) -> Result<bool, rusqlite::Error> {
        assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");
        let regexp: std::sync::Arc<regex::Regex> = ctx
            .get_or_create_aux(0, |vr| -> Result<_, BoxError> {
                Ok(regex::Regex::new(vr.as_str()?)?)
            })?;
        let is_match = {
            let text = ctx
                .get_raw(1)
                .as_str()
                .map_err(|e| rusqlite::Error::UserFunctionError(e.into()))?;

            regexp.is_match(text)
        };

        Ok(is_match)
    }

    match db.create_scalar_function(
        "regexp",
        2,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
        regexp_match,
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::DatabaseWriteErr(err.to_string())),
    }
}

/// get the app data dir and create it if it does not exist
fn get_and_create_app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, Error> {
    // get the app data dir
    let app_data_dir = app.path_resolver().app_data_dir();
    if app_data_dir.is_none() {
        return Err(Error::GetAppDataDirErr);
    }
    let app_data_dir = app_data_dir.unwrap();

    // if the app data dir does not exist, create it
    if !app_data_dir.exists() {
        if let Err(err) = std::fs::create_dir_all(app_data_dir.as_path()) {
            return Err(Error::CreateAppDataDirErr(err.to_string()));
        }
    }

    Ok(app_data_dir)
}

/// get the database connection and create it if it does not exist
fn get_and_create_database(app_data_dir: std::path::PathBuf) -> Result<Connection, Error> {
    // create the database dir if it does not exist
    let database_path = app_data_dir.join("database");

    // TODO test if the database file does not exist

    match Connection::open(database_path) {
        Ok(connection) => Ok(connection),
        Err(err) => Err(Error::OpenDatabaseErr(err.to_string())),
    }
}

/// get the current version from the tauri config
fn get_current_version(app: &AppHandle) -> Result<String, Error> {
    let current_version = app.config().package.version.clone();
    if current_version.is_none() {
        return Err(Error::GetVersionFromTauriErr);
    }
    Ok(current_version.unwrap())
}

/// get the save version from the database
/// if there is no version, it is the first time the app is launched and return is 0.0.0
///
/// this function does not test validity of the version in the database
fn get_save_version(connection: &Connection) -> Result<String, Error> {
    // get the latest version, if it exists
    // if not exists, it is the first time the app is launched return 0.0.0

    let version = connection.query_row(
        "SELECT version FROM version ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    match version {
        Ok(version) => Ok(version),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // the version table is empty, it is the first time the app is launched
            Ok("0.0.0".to_string())
        }
        Err(err) => Err(Error::GetVersionFromDatabaseErr(err.to_string())),
    }
}

/// when the app is launched by the user for the first time, init the version table
///
/// this function will
///     - insert the current version into the version table
#[warn(unused_must_use)]
fn first_lunch_the_version_table(connection: &Connection, app: &AppHandle) -> Result<(), Error> {
    let current_version = get_current_version(app)?;

    // insert the current version into the version table
    insert_version(connection, current_version)
}

#[warn(unused_must_use)]
/// Insert a row into the version table
/// This function is used to insert a new version into the version table
fn insert_version(connection: &Connection, version: String) -> Result<(), Error> {
    match connection.execute("INSERT INTO version (version) VALUES (?)", [&version]) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::InsertVersionErr(version, err.to_string())),
    }
}

/// this function will
///     - check if the save version is the same as the current version
///     - if not, trigger backward_compatibility and update the version table
///     - if yes, do nothing
#[warn(unused_must_use)]
fn check_save_version_and_current_version(
    save_version: String,
    connection: &Connection,
    app: &AppHandle,
) -> Result<(), Error> {
    let current_version = get_current_version(app)?;

    debug!("save version: {}", save_version);
    debug!("current version: {}", current_version);

    if current_version == save_version {
        return Ok(());
    }

    // if the save version is not the same as the current version, trigger the backward comparability
    backward_comparability(app, connection, save_version)?;

    // update the version table
    insert_version(connection, current_version)
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
fn init_version_table(connection: &Connection, app: &AppHandle) -> Result<(), Error> {
    // create the version table if it does not exist
    match connection.execute(
        "CREATE TABLE IF NOT EXISTS version (id INTEGER PRIMARY KEY AUTOINCREMENT, version TEXT NOT NULL)",
        [],
    ) {
        Ok(_) => {}
        Err(err) => return Err(Error::CreateVersionTableErr(err.to_string())),
    };

    // try get the save version
    let save_version = get_save_version(connection)?;

    if save_version == *"0.0.0" {
        // first launch the app, init the version table
        first_lunch_the_version_table(connection, app)?;
    } else {
        // not the first time the app is launched, check the save version and the current version
        check_save_version_and_current_version(save_version, connection, app)?;
    }

    Ok(())
}

/// init the clips table
///
/// this function will
///     - create the clips table if it does not exist
#[warn(unused_must_use)]
fn init_clips_table(connection: &Connection) -> Result<(), Error> {
    // create the clips table if it does not exist
    match connection.execute(
        "CREATE TABLE IF NOT EXISTS clips (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type INTEGER NOT NULL DEFAULT 0,
            text TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::CreateClipsTableErr(err.to_string())),
    }
}

/// Get all labels from the labels table
/// This function is used to get all the labels from the labels table
/// and return a Vec<String> of the labels
pub fn get_all_labels(connection: &Connection) -> Result<Vec<String>, Error> {
    // get all the labels from the labels table
    let mut labels: Vec<String> = Vec::new();
    let mut statement = match connection.prepare("SELECT name FROM labels") {
        Ok(prepared_statement) => prepared_statement,
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };
    let statement = match statement.query_map([], |row| row.get(0)) {
        Ok(statement) => statement,
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };
    for label in statement {
        match label {
            Ok(label) => labels.push(label),
            Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
        }
    }

    Ok(labels)
}

/// init the labels table
///
/// this function will
///   - create the labels table if it does not exist
///   - insert the default labels into the labels table
///   - create the label_base32(label_name) tables for each label
#[warn(unused_must_use)]
fn init_labels_table(connection: &Connection) -> Result<(), Error> {
    // create the labels table if it does not exist
    match connection.execute(
        "CREATE TABLE IF NOT EXISTS labels (name TEXT PRIMARY KEY)",
        [],
    ) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

    // insert the default labels into the labels table
    match connection.execute("INSERT OR IGNORE INTO labels (name) VALUES ('pinned')", []) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };
    match connection.execute(
        "INSERT OR IGNORE INTO labels (name) VALUES ('favourite')",
        [],
    ) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

    // get all the labels from the labels table
    let labels = get_all_labels(connection)?;

    // create the label_base32(label_name) tables for each label
    for label in labels {
        match connection.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY,
                    FOREIGN KEY (id) REFERENCES clips (id)
                        ON UPDATE CASCADE
                        ON DELETE CASCADE
                )",
                label_name_to_table_name(&label)
            ),
            [],
        ) {
            Ok(_) => (),
            Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
        };
    }

    Ok(())
}

pub fn label_name_to_table_name(label_name: &str) -> String {
    format!(
        "label_{}",
        BASE32.encode(label_name.as_bytes()).replace('=', "_")
    )
}

/// Get all the versions from the database,
/// in the format of Vec<id,String>.
/// This function is used to export the data only.
/// And should not be used to check the version of the database,
/// when the app is launched.
pub async fn get_all_versions(app: &AppHandle) -> Result<Vec<(i64, String)>, Error> {
    let db_connection = app.state::<DatabaseStateMutex>();
    let db_connection = db_connection.database_connection.lock().await;

    let mut res: Vec<(i64, String)> = Vec::new();
    let mut stmt = match db_connection.prepare("SELECT * FROM version") {
        Ok(prepared_statement) => prepared_statement,
        Err(err) => return Err(Error::GetVersionFromDatabaseErr(err.to_string())),
    };
    fn get_id_and_version(row: &rusqlite::Row) -> Result<(i64, String), rusqlite::Error> {
        let id: i64 = row.get(0)?;
        let version: String = row.get(1)?;
        Ok((id, version))
    }
    let stmt = match stmt.query_map([], get_id_and_version) {
        Ok(res) => res,
        Err(err) => return Err(Error::GetVersionFromDatabaseErr(err.to_string())),
    };
    for row in stmt {
        match row {
            Ok(row) => res.push(row),
            Err(err) => return Err(Error::GetVersionFromDatabaseErr(err.to_string())),
        }
    }

    Ok(res)
}
