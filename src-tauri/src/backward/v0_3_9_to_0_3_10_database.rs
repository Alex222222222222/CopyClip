use std::collections::HashMap;

use clip::Clip;
use log::debug;
use rusqlite::Connection;

use crate::database::label_name_to_table_name;

/// when moving from 0.3.9 to 0.3.10,
/// we need to upgrade clips table
/// FROM:
/// ```sql
/// CREATE TABLE IF NOT EXISTS clips (
///     id INTEGER PRIMARY KEY,
///     text TEXT,
///     timestamp INTEGER,
///     type INTEGER DEFAULT 0,
/// );
///
/// TO:
/// ```sql
/// CREATE TABLE IF NOT EXISTS clips (
///     id INTEGER PRIMARY KEY AUTOINCREMENT,
///     type INTEGER NOT NULL DEFAULT 0,
///     data BLOB NOT NULL,
///     search_text TEXT NOT NULL,
///     timestamp INTEGER NOT NULL
/// );
/// ```
#[warn(unused_must_use)]
pub fn upgrade(connection: &Connection) -> Result<(), anyhow::Error> {
    debug!("update from 0.3.9 to 0.3.10");

    debug!("Creating New clips table");
    // create new clips table
    connection.execute(
        "CREATE TABLE IF NOT EXISTS clips_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type INTEGER NOT NULL DEFAULT 0,
            data BLOB NOT NULL,
            search_text TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;

    // select the old data from the old table
    let mut stmt = connection.prepare("SELECT id, type, text, timestamp FROM clips")?;
    let mut rows = stmt.query([])?;

    // insert the old data into new table
    while let Some(row) = rows.next()? {
        debug!("Processing row: {:?}", row);
        let id: u64 = row.get(0)?;
        let clip_type: u8 = row.get(1)?;
        let text: String = row.get(2)?;
        let timestamp: i64 = row.get(3)?;
        let clip = Clip::new_clip_with_id_timestamp(id, &text, clip_type, timestamp)?;
        connection.execute(
            "INSERT INTO clips_new(id, type, data, search_text, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                rusqlite::types::Value::Integer(id as i64),
                rusqlite::types::Value::Integer(clip_type as i64),
                clip.get_data_database(),
                clip.get_search_text_database(),
                rusqlite::types::Value::Integer(timestamp),
            ],
        )?;
    }

    // get all the labels
    debug!("Get all the labels");
    let mut labels: HashMap<String, Vec<u64>> = HashMap::new();
    let mut stmt = connection.prepare("SELECT name FROM labels")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let label: String = row.get("name")?;
        labels.insert(label, vec![]);
    }
    // for each label, get the corresponding id
    for label in labels.iter_mut() {
        let stmt: String = format!(
            "SELECT id FROM {}",
            label_name_to_table_name(label.0.as_str())
        );
        let mut stmt = connection.prepare(&stmt)?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let id: u64 = row.get("id")?;
            label.1.push(id);
        }
    }

    // drop the old table
    // as we are using CASCADE in the labels table, this will also delete all the label info
    debug!("Dropping the old table");
    connection.execute("DROP TABLE clips", [])?;

    // rename the new table to clips
    debug!("Renaming the new table to clips");
    connection.execute("ALTER TABLE clips_new RENAME TO clips", [])?;

    // insert the labels id back into labels table
    // for each label, get the corresponding id
    for label in labels.iter() {
        let stmt: String = format!(
            "INSERT INTO {}(id)
            VALUES (?)",
            label_name_to_table_name(label.0.as_str())
        );
        for id in label.1.iter() {
            connection.execute(&stmt, [id])?;
        }
    }

    Ok(())
}
