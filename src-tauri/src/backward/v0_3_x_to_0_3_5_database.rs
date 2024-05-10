use rusqlite::Connection;

use crate::error;

/// when moving from 0.3.3, 0.3.4, to 0.3.5,
/// I rename the column favorite to favourite in clips table, so need to do a sql ALTER command
#[warn(unused_must_use)]
pub fn upgrade(connection: &Connection) -> Result<(), error::Error> {
    // virtual table does not support ALTER column command, so need to recreate the table;

    // create the new table;
    match connection.execute(
        "CREATE TABLE IF NOT EXISTS clips_new (
            id INTEGER PRIMARY KEY,
            text TEXT,
            timestamp INTEGER,
            favourite INTEGER,
        )",
        [],
    ) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                "failed to create new clips table when update from 0.3.x to 0.3.5".to_string(),
                err.to_string(),
            ));
        }
    }

    // insert the data into new table
    match connection.execute(
        "INSERT INTO clips_new(id, text, timestamp, favourite)
        SELECT id, text, timestamp, favorite FROM clips",
        [],
    ) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                "failed to insert old data into new table when update from 0.3.x to 0.3.5"
                    .to_string(),
                err.to_string(),
            ));
        }
    }

    // drop the old table
    match connection.execute("DROP TABLE clips", []) {
        Ok(_) => (),
        Err(err) => {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                "failed to drop the old table when update from 0.3.x to 0.3.5".to_string(),
                err.to_string(),
            ));
        }
    }

    // rename the new table
    match connection.execute("ALTER TABLE clips_new RENAME TO clips", []) {
        Ok(_) => Ok(()),
        Err(err) => Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to rename to new table to clips when update from 0.3.x to 0.3.5".to_string(),
            err.to_string(),
        )),
    }
}
