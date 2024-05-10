use log::debug;
use rusqlite::Connection;

use crate::error;

/// when moving from 0.3.7 to 0.3.8,
/// we need to update the pinned clips table
/// FROM:
/// ```sql
/// CREATE TABLE IF NOT EXISTS pinned_clips (
///     id INTEGER PRIMARY KEY,
///     text TEXT,
///     timestamp INTEGER,
/// );
///
/// TO:
/// ```sql
/// CREATE TABLE IF NOT EXISTS clips (
///     id INTEGER PRIMARY KEY,
///     text TEXT,
///     timestamp INTEGER,
///     favourite INTEGER DEFAULT 0,
///     pinned INTEGER DEFAULT 0
/// );
/// ```
#[warn(unused_must_use)]
pub fn upgrade(connection: &Connection) -> Result<(), error::Error> {
    debug!("update from 0.3.7 to 0.3.8");

    // create the new table;
    match connection.execute(
        "CREATE TABLE IF NOT EXISTS clips_new (
            id INTEGER PRIMARY KEY,
            text TEXT,
            timestamp INTEGER,
            favourite INTEGER DEFAULT 0,
            pinned INTEGER DEFAULT 0
        )",
        [],
    ) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                "failed to create new clips table when update from 0.3.7 to 0.3.8".to_string(),
                err.to_string(),
            ));
        }
    }

    // insert the old data into new table
    // we can get the old data from pinned_clips table
    match connection.execute(
        "INSERT INTO clips_new(text, timestamp, favourite, pinned)
        SELECT text, timestamp, favourite, 0 as pinned FROM clips
        UNION
        SELECT text, timestamp, 0 as favourite, 1 as pinned FROM pinned_clips
        ORDER BY timestamp ASC, pinned ASC
        ",
        [],
    ) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                "failed to insert old data into new table when update from 0.3.7 to 0.3.8"
                    .to_string(),
                err.to_string(),
            ));
        }
    }

    // set the new table's id to auto increment (rowid)
    match connection.execute("UPDATE clips_new SET id = rowid", []) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                "failed to set the new table's id to auto increment when update from 0.3.7 to 0.3.8"
                    .to_string(),
                err.to_string(),
            ));
        }
    }

    // drop the old pinned table
    match connection.execute("DROP TABLE pinned_clips", []) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                "failed to drop the old table when update from 0.3.7 to 0.3.8".to_string(),
                err.to_string(),
            ));
        }
    }

    // drop the old clips table
    match connection.execute("DROP TABLE clips", []) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                "failed to drop the old table when update from 0.3.7 to 0.3.8".to_string(),
                err.to_string(),
            ));
        }
    }

    // rename the new table
    match connection.execute("ALTER TABLE clips_new RENAME TO clips", []) {
        Ok(_) => Ok(()),
        Err(err) => Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to rename new table to clips when update from 0.3.7 to 0.3.8".to_string(),
            err.to_string(),
        )),
    }
}
