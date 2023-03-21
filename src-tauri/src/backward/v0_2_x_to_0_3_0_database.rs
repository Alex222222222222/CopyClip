use sqlite::State;

use crate::error;

/// the database before 0.3.0 is a sqlite database
/// the database after 0.3.0 is a a sqlite3 fts4 database
pub fn upgrade(connection: &sqlite::Connection) -> Result<(), error::Error> {
    let mut statement = connection
        .prepare(
            "CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts4(
                id INTEGER PRIMARY KEY,
                text TEXT,
                timestamp INTEGER, 
                favorite INTEGER,
            )",
        )
        .unwrap();
    let state = statement.next();
    match state {
        Ok(State::Done) => {
            // do nothing
        }
        Ok(State::Row) => {
            // return error
            return Err(error::Error::CreateClipsTableErr(
                "Failed to create clips table".to_string(),
            ));
        }
        Err(err) => {
            return Err(error::Error::CreateClipsTableErr(err.to_string()));
        }
    }

    let mut statement = connection
        .prepare(
            "INSERT INTO clips_fts(id, text, timestamp, favorite)
                SELECT id, text, timestamp, favorite FROM clips",
        )
        .unwrap();
    let state = statement.next();
    match state {
        Ok(State::Done) => {
            // do nothing
        }
        Ok(State::Row) => {
            // return error
            return Err(error::Error::CreateClipsTableErr(
                "When upgrading from 0.2.x to 0.3.0, failed to insert old data into new table"
                    .to_string(),
            ));
        }
        Err(err) => {
            return Err(error::Error::CreateClipsTableErr(
                format!("When upgrading from 0.2.x to 0.3.0, failed to insert old data into new table, error message: {err}"),
            ));
        }
    }

    let mut statement = connection.prepare("DROP TABLE clips").unwrap();
    let state = statement.next();
    match state {
        Ok(State::Done) => {
            // do nothing
        }
        Ok(State::Row) => {
            // return error
            return Err(error::Error::CreateClipsTableErr(
                "When upgrading from 0.2.x to 0.3.0, failed to drop old table".to_string(),
            ));
        }
        Err(err) => {
            return Err(error::Error::CreateClipsTableErr(format!(
                "When upgrading from 0.2.x to 0.3.0, failed to drop old table, error message: {err}",
            )));
        }
    }

    let mut statement = connection
        .prepare("ALTER TABLE clips_fts RENAME TO clips")
        .unwrap();
    let state = statement.next();
    match state {
        Ok(State::Done) => {
            // do nothing
        }
        Ok(State::Row) => {
            // return error
            return Err(error::Error::CreateClipsTableErr(
                "When upgrading from 0.2.x to 0.3.0, failed to rename new table".to_string(),
            ));
        }
        Err(err) => {
            return Err(error::Error::CreateClipsTableErr(format!(
                "When upgrading from 0.2.x to 0.3.0, failed to rename new table, error message: {err}",
            )));
        }
    }

    Ok(())
}
