use log::debug;
use rusqlite::Connection;

use crate::{database::label_name_to_table_name, error::Error};

/// when moving from 0.3.8 to 0.3.9,
/// we need to update the pinned clips table
/// FROM:
/// ```sql
/// CREATE TABLE IF NOT EXISTS clips (
///     id INTEGER PRIMARY KEY,
///     text TEXT,
///     timestamp INTEGER,
///     favourite INTEGER DEFAULT 0,
///     pinned INTEGER DEFAULT 0
/// );
///
/// TO:
/// ```sql
/// CREATE TABLE IF NOT EXISTS clips (
///     id INTEGER PRIMARY KEY,
///     text TEXT,
///     timestamp INTEGER,
///     type INTEGER DEFAULT 0,
/// );
///
/// Labels table
///
/// ```
#[warn(unused_must_use)]
pub fn upgrade(connection: &Connection) -> Result<(), Error> {
    debug!("update from 0.3.8 to 0.3.9");

    // create new clips table
    match connection.execute(
        "CREATE TABLE IF NOT EXISTS clips_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type INTEGER,
            text TEXT,
            timestamp INTEGER
        )",
        [],
    ) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

    // insert the old data into new table
    match connection.execute(
        "INSERT INTO clips_new(id, type, text, timestamp)
        SELECT id, 0 as type, text, timestamp FROM clips",
        [],
    ) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

    // rename the old table
    match connection.execute("ALTER TABLE clips RENAME TO clips_old", []) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

    // rename the new table
    match connection.execute("ALTER TABLE clips_new RENAME TO clips", []) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

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
    let labels = vec!["pinned".to_string(), "favourite".to_string()];

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

    // insert the pinned clips into the new table
    match connection.execute(
        &format!(
            "INSERT INTO {} (id)
            SELECT id FROM clips_old WHERE pinned = 1",
            label_name_to_table_name("pinned")
        ),
        [],
    ) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

    // insert the favourite clips into the new table
    match connection.execute(
        &format!(
            "INSERT INTO {} (id)
            SELECT id FROM clips_old WHERE favourite = 1",
            label_name_to_table_name("favourite")
        ),
        [],
    ) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

    // drop the old table
    match connection.execute("DROP TABLE clips_old", []) {
        Ok(_) => (),
        Err(err) => return Err(Error::DatabaseWriteErr(err.to_string())),
    };

    Ok(())
}
