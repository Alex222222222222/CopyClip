use crate::error;

/// when moving fom 0.3.5, 0.3.6, to 0.3.7,
/// we need to update the pinned clips table
/// FROM:
/// ```sql
/// CREATE TABLE IF NOT EXISTS pinned_clips (
///     id INTEGER PRIMARY KEY
/// );
/// ```
/// TO:
/// ```sql
/// CREATE TABLE IF NOT EXISTS pinned_clips (
///     id INTEGER PRIMARY KEY,
///     text TEXT,
///     timestamp INTEGER,
/// );
/// ```
#[warn(unused_must_use)]
pub async fn upgrade(connection: &sqlx::SqlitePool) -> Result<(), error::Error> {
    // create the new table;
    let res = sqlx::query(
        "CREATE TABLE IF NOT EXISTS pinned_clips_new (
            id INTEGER PRIMARY KEY,
            text TEXT,
            timestamp INTEGER
        )",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to create new pinned_clips table when update from 0.3.x to 0.3.7".to_string(),
            err.to_string(),
        ));
    };

    // insert the old data into new table
    // we can get the old data from clips table
    let res = sqlx::query(
        "INSERT INTO pinned_clips_new(id, text, timestamp)
        SELECT id, text, timestamp FROM clips WHERE id IN (SELECT id FROM pinned_clips)",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to insert old data into new table when update from 0.3.x to 0.3.7".to_string(),
            err.to_string(),
        ));
    };

    // drop the old table
    let res = sqlx::query("DROP TABLE pinned_clips")
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to drop the old table when update from 0.3.x to 0.3.7".to_string(),
            err.to_string(),
        ));
    }

    // rename the new table
    let res = sqlx::query("ALTER TABLE pinned_clips_new RENAME TO pinned_clips")
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::UpdateClipsInDatabaseErr(
            "failed to rename to new table to pinned_clips when update from 0.3.x to 0.3.7"
                .to_string(),
            err.to_string(),
        ));
    };

    Ok(())
}
