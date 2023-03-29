use sqlx::SqlitePool;

use crate::error;

/// the database before 0.3.0 is a sqlite database
/// the database after 0.3.0 is a sqlite3 fts4 database
pub async fn upgrade(connection: &SqlitePool) -> Result<(), error::Error> {
    let res = sqlx::query(
        "CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts4(
            id INTEGER PRIMARY KEY,
            text TEXT,
            timestamp INTEGER, 
            favorite INTEGER,
        )",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::CreateClipsTableErr(err.to_string()));
    }

    let res = sqlx::query(
        "INSERT INTO clips_fts(id, text, timestamp, favorite)
        SELECT id, text, timestamp, favorite FROM clips",
    )
    .fetch_optional(connection)
    .await;
    if let Err(err) = res {
        return Err(error::Error::CreateClipsTableErr(
            format!("When upgrading from 0.2.x to 0.3.0, failed to insert old data into new table, error message: {err}"),
        ));
    }

    let res = sqlx::query("DROP TABLE clips")
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::CreateClipsTableErr(format!(
            "When upgrading from 0.2.x to 0.3.0, failed to drop old table, error message: {err}",
        )));
    }

    let res = sqlx::query("ALTER TABLE clips_fts RENAME TO clips")
        .fetch_optional(connection)
        .await;
    if let Err(err) = res {
        return Err(error::Error::CreateClipsTableErr(format!(
            "When upgrading from 0.2.x to 0.3.0, failed to rename new table, error message: {err}",
        )));
    }

    Ok(())
}
