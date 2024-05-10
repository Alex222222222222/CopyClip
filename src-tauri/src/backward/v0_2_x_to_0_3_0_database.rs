use rusqlite::Connection;

use crate::error;

/// the database before 0.3.0 is a sqlite database
/// the database after 0.3.0 is a sqlite3 fts4 database
pub fn upgrade(connection: &Connection) -> Result<(), error::Error> {
    match connection.execute(
        "CREATE TABLE IF NOT EXISTS clips_fts(
            id INTEGER PRIMARY KEY,
            text TEXT,
            timestamp INTEGER, 
            favorite INTEGER,
        )",
        [],
    ) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::CreateClipsTableErr(err.to_string()));
        }
    }

    match connection.execute(
        "INSERT INTO clips_fts(id, text, timestamp, favorite)
        SELECT id, text, timestamp, favorite FROM clips",
        [],
    ) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::CreateClipsTableErr(
                format!("When upgrading from 0.2.x to 0.3.0, failed to insert old data into new table, error message: {err}"),
            ));
        }
    }

    match connection.execute("DROP TABLE clips", []) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::CreateClipsTableErr(
                format!("When upgrading from 0.2.x to 0.3.0, failed to drop old table, error message: {err}"),
            ));
        }
    }

    match connection.execute("ALTER TABLE clips_fts RENAME TO clips", []) {
        Ok(_) => Ok(()),
        Err(err) => Err(error::Error::CreateClipsTableErr(format!(
            "When upgrading from 0.2.x to 0.3.0, failed to rename new table, error message: {err}",
        ))),
    }
}
