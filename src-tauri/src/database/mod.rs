mod init;

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
///     - data BLOB (gz compressed data)
///     - search_text TEXT (Using for search, trimmed under 10000 chars)
///     - timestamp INTEGER
///  - label_base32(label_name) tables for each label
///     - used to store the clips id for each label
///     - id INTEGER PRIMARY KEY foreign key to clips table
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::Error;

pub use init::init_database_connection;

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
