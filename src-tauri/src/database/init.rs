use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
/// The functions related to the initialization of the database.
use log::debug;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// init the database connection and create the table
/// also init the clips mutex
pub fn init_database_connection(app: &AppHandle) -> Result<Connection, anyhow::Error> {
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

    connection.cache_flush()?;

    Ok(connection)
}

/// get the app data dir and create it if it does not exist
fn get_and_create_app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, anyhow::Error> {
    // get the app data dir
    let app_data_dir = app.path().app_data_dir();
    if let Err(err) = app_data_dir {
        return Err(crate::error::Error::GetAppDataDirErr(err.to_string()).into());
    }
    let app_data_dir = app_data_dir.unwrap();

    // if the app data dir does not exist, create it
    if !app_data_dir.exists() {
        std::fs::create_dir_all(app_data_dir.as_path())?
    }

    Ok(app_data_dir)
}

/// get the database connection and create it if it does not exist
fn get_and_create_database(app_data_dir: std::path::PathBuf) -> Result<Connection, anyhow::Error> {
    // create the database dir if it does not exist
    let database_path = app_data_dir.join("database");

    Ok(Connection::open(database_path)?)
}

/// create and load the fuzzy search function to the database
fn create_fuzzy_search_function(connection: &Connection) -> Result<(), anyhow::Error> {
    fn fuzzy_search(ctx: &rusqlite::functions::Context) -> Result<isize, rusqlite::Error> {
        let pattern = ctx.get::<String>(1)?;
        let text = ctx.get::<String>(0)?;

        let matcher = SkimMatcherV2::default();

        match matcher.fuzzy_match(&text, &pattern) {
            Some(score) => {
                debug!("score: {}", score);
                Ok(score as isize)
            }
            None => Ok(0),
        }
    }

    connection.create_scalar_function(
        "fuzzy_search",
        2,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
        fuzzy_search,
    )?;

    Ok(())
}

/// create regex search function
fn create_regexp_function(db: &Connection) -> Result<(), anyhow::Error> {
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

    db.create_scalar_function(
        "regexp",
        2,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
        regexp_match,
    )?;

    Ok(())
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
fn init_version_table(connection: &Connection, app: &AppHandle) -> Result<(), anyhow::Error> {
    // create the version table if it does not exist
    connection.execute(
        "CREATE TABLE IF NOT EXISTS version (id INTEGER PRIMARY KEY AUTOINCREMENT, version TEXT NOT NULL)",
        [],
    )?;

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

/// get the save version from the database
/// if there is no version, it is the first time the app is launched and return is 0.0.0
///
/// this function does not test validity of the version in the database
fn get_save_version(connection: &Connection) -> Result<String, anyhow::Error> {
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
        Err(err) => Err(err.into()),
    }
}

/// when the app is launched by the user for the first time, init the version table
///
/// this function will
///     - insert the current version into the version table
#[warn(unused_must_use)]
fn first_lunch_the_version_table(
    connection: &Connection,
    app: &AppHandle,
) -> Result<(), anyhow::Error> {
    let current_version = get_current_version(app)?;

    // insert the current version into the version table
    insert_version(connection, current_version)
}

/// get the current version from the tauri config
fn get_current_version(app: &AppHandle) -> Result<String, anyhow::Error> {
    let current_version = app.config().version.clone();
    if current_version.is_none() {
        return Err(crate::error::Error::GetVersionFromTauriErr.into());
    }
    Ok(current_version.unwrap())
}

/// init the clips table
///
/// this function will
///     - create the clips table if it does not exist
#[warn(unused_must_use)]
fn init_clips_table(connection: &Connection) -> Result<(), anyhow::Error> {
    // create the clips table if it does not exist
    connection.execute(
        "CREATE TABLE IF NOT EXISTS clips (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type INTEGER NOT NULL DEFAULT 0,
            data BLOB NOT NULL,
            search_text TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// init the labels table
///
/// this function will
///   - create the labels table if it does not exist
///   - insert the default labels into the labels table
///   - create the label_base32(label_name) tables for each label
#[warn(unused_must_use)]
fn init_labels_table(connection: &Connection) -> Result<(), anyhow::Error> {
    // create the labels table if it does not exist
    connection.execute(
        "CREATE TABLE IF NOT EXISTS labels (name TEXT PRIMARY KEY)",
        [],
    )?;

    // insert the default labels into the labels table
    connection.execute("INSERT OR IGNORE INTO labels (name) VALUES ('pinned')", [])?;
    connection.execute(
        "INSERT OR IGNORE INTO labels (name) VALUES ('favourite')",
        [],
    )?;

    // get all the labels from the labels table
    let labels = get_all_labels(connection)?;

    // create the label_base32(label_name) tables for each label
    for label in labels {
        connection.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY,
                    FOREIGN KEY (id) REFERENCES clips (id)
                        ON UPDATE CASCADE
                        ON DELETE CASCADE
                )",
                super::label_name_to_table_name(&label)
            ),
            [],
        )?;
    }

    Ok(())
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
) -> Result<(), anyhow::Error> {
    let current_version = get_current_version(app)?;

    debug!("save version: {}", save_version);
    debug!("current version: {}", current_version);

    if current_version == save_version {
        return Ok(());
    }

    // if the save version is not the same as the current version, trigger the backward comparability
    crate::backward::backward_comparability(app, connection, save_version)?;

    // update the version table
    insert_version(connection, current_version)
}

#[warn(unused_must_use)]
/// Insert a row into the version table
/// This function is used to insert a new version into the version table
fn insert_version(connection: &Connection, version: String) -> Result<(), anyhow::Error> {
    connection.execute("INSERT INTO version (version) VALUES (?)", [&version])?;

    Ok(())
}

/// Get all labels from the labels table
/// This function is used to get all the labels from the labels table
/// and return a Vec<String> of the labels
fn get_all_labels(connection: &Connection) -> Result<Vec<String>, anyhow::Error> {
    // get all the labels from the labels table
    let mut labels: Vec<String> = Vec::new();
    let mut statement = connection.prepare("SELECT name FROM labels")?;
    let statement = statement.query_map([], |row| row.get(0))?;
    for label in statement {
        match label {
            Ok(label) => labels.push(label),
            Err(err) => return Err(err.into()),
        }
    }

    Ok(labels)
}
