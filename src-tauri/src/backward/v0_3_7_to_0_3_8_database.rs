use log::debug;

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
pub async fn upgrade(connection: &sqlx::SqlitePool) -> Result<(), error::Error> {
    debug!("update from 0.3.7 to 0.3.8");

    // create the new table;
    let res = sqlx::query(
        "CREATE VIRTUAL TABLE IF NOT EXISTS clips_new USING fts4 (
            id INTEGER PRIMARY KEY,
            text TEXT,
            timestamp INTEGER,
            favourite INTEGER DEFAULT 0,
            pinned INTEGER DEFAULT 0
        )",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to create new clips table when update from 0.3.7 to 0.3.8".to_string(),
            err.to_string(),
        ));
    };

    // insert the old data into new table
    // we can get the old data from pinned_clips table
    let res = sqlx::query(
        "INSERT INTO clips_new(text, timestamp, favourite, pinned)
        SELECT text, timestamp, favourite, 0 as pinned FROM clips
        UNION
        SELECT text, timestamp, 0 as favourite, 1 as pinned FROM pinned_clips
        ORDER BY timestamp ASC, pinned ASC
        ",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to insert old data into new table when update from 0.3.7 to 0.3.8".to_string(),
            err.to_string(),
        ));
    };

    // set the new table's id to auto increment (rowid)
    let res = sqlx::query("UPDATE clips_new SET id = rowid")
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to set the new table's id to auto increment when update from 0.3.7 to 0.3.8"
                .to_string(),
            err.to_string(),
        ));
    };

    // drop the old pinned table
    let res = sqlx::query("DROP TABLE pinned_clips")
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to drop the old table when update from 0.3.7 to 0.3.8".to_string(),
            err.to_string(),
        ));
    }
    // drop the old clips table
    let res = sqlx::query("DROP TABLE clips")
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to drop the old table when update from 0.3.7 to 0.3.8".to_string(),
            err.to_string(),
        ));
    }

    // rename the new table
    let res = sqlx::query("ALTER TABLE clips_new RENAME TO clips")
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to rename to new table to pinned_clips when update from 0.3.7 to 0.3.8"
                .to_string(),
            err.to_string(),
        ));
    };

    Ok(())
}
