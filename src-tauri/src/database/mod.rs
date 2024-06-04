mod init;
mod search;

use clip::{Clip, SearchConstraint};
use data_encoding::BASE32;
use log::debug;
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

pub use init::init_database_connection;

/// Convert the label name to the table name
///
/// In the format of label_{base32(label_name)}
/// and the padding '=' is replaced by '_'
///
/// ```rust
/// use clip::database::label_name_to_table_name;
/// assert_eq!(label_name_to_table_name("pinned"), "OBUW43TFMQ______".to_string());
/// ```
pub fn label_name_to_table_name(label_name: &str) -> String {
    format!(
        "label_{}",
        BASE32.encode(label_name.as_bytes()).replace('=', "_")
    )
}

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
    pub async fn set_database_connection(
        &self,
        connection: Connection,
    ) -> Result<(), anyhow::Error> {
        let mut db_connection = self.database_connection.lock().await;
        *db_connection = connection;

        Ok(())
    }

    /// search the database for clips that match the search constraints
    pub async fn search_clips(
        &self,
        constraints: Vec<SearchConstraint>,
    ) -> Result<Vec<clip::Clip>, anyhow::Error> {
        search::search_clips(&self.database_connection, constraints).await
    }

    /// Get the id of a clip in the database by the position counting from the lower id to the higher id.
    /// 0 will return the lowest id.
    /// 1 will return the second lowest id.
    ///
    /// If the position is out of range, return None.
    pub async fn get_id_with_pos(&self, pos: u64) -> Result<Option<u64>, anyhow::Error> {
        let db_connection = self.database_connection.lock().await;

        match db_connection.query_row(
            "SELECT id FROM clips ORDER BY id ASC LIMIT 1 OFFSET ?",
            [pos.to_string()],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// Get all labels from the labels table
    /// This function is used to get all the labels from the labels table
    /// and return a Vec<String> of the labels
    pub async fn get_all_labels(&self) -> Result<Vec<String>, anyhow::Error> {
        let connection = self.database_connection.lock().await;

        // get all the labels from the labels table
        let mut labels: Vec<String> = Vec::new();
        let mut statement = connection.prepare("SELECT name FROM labels")?;
        let statement = statement.query_map([], |row| row.get(0))?;
        for label in statement {
            match label {
                Ok(label) => labels.push(label),
                Err(err) => return Err(err.into()),
            }
        }

        Ok(labels)
    }

    /// Get all the versions from the database,
    /// in the format of Vec<id,String>.
    /// This function is used to export the data only.
    /// And should not be used to check the version of the database,
    /// when the app is launched.
    pub async fn get_all_versions(&self) -> Result<Vec<(i64, String)>, anyhow::Error> {
        let db_connection = self.database_connection.lock().await;

        let mut res: Vec<(i64, String)> = Vec::new();
        let mut stmt = db_connection.prepare("SELECT * FROM version")?;
        fn get_id_and_version(row: &rusqlite::Row) -> Result<(i64, String), rusqlite::Error> {
            let id: i64 = row.get(0)?;
            let version: String = row.get(1)?;
            Ok((id, version))
        }
        let stmt = stmt.query_map([], get_id_and_version)?;
        for row in stmt {
            match row {
                Ok(row) => res.push(row),
                Err(err) => return Err(err.into()),
            }
        }

        Ok(res)
    }

    /// Get the total number of clips in the database
    pub async fn get_total_number_of_clip(&self) -> Result<u64, anyhow::Error> {
        let db_connection = self.database_connection.lock().await;

        match db_connection.query_row("SELECT COUNT(1) FROM clips", [], |row| row.get(0)) {
            Ok(res) => Ok(res),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(err) => Err(err.into()),
        }
    }

    /// Test if a clip have a label
    ///   - return true if the clip have the label
    ///   - otherwise return false
    pub async fn clip_have_label(&self, id: u64, label: &str) -> Result<bool, anyhow::Error> {
        let db_connection = self.database_connection.lock().await;

        match db_connection.query_row(
            &format!(
                "SELECT id FROM {} WHERE id = ?",
                label_name_to_table_name(label)
            ),
            [id.to_string()],
            |_| Ok(()),
        ) {
            Ok(_) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    /// Get the id of the latest clip,
    ///
    /// If there is no clip, return None.
    pub async fn get_latest_clip_id(&self) -> Result<Option<u64>, anyhow::Error> {
        let db_connection = self.database_connection.lock().await;

        match db_connection.query_row("SELECT id FROM clips ORDER BY id DESC LIMIT 1", [], |row| {
            row.get(0)
        }) {
            Ok(res) => Ok(Some(res)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// get how many clips one label has
    ///   - return the number of clips
    ///   - return 0 if the label is not found
    pub async fn get_label_clip_number(&self, label: &str) -> Result<u64, anyhow::Error> {
        let db_connection = self.database_connection.lock().await;

        let res = match db_connection.query_row(
            &format!("SELECT COUNT(1) FROM {}", label_name_to_table_name(label)),
            [],
            |row| row.get(0),
        ) {
            Ok(res) => res,
            Err(rusqlite::Error::QueryReturnedNoRows) => 0,
            Err(err) => {
                return Err(err.into());
            }
        };

        Ok(res)
    }

    /// get the clip with the label with the position of the clip in that label
    ///   - return the clip id
    ///   - return None if the clip is not found
    ///   - count from the lower id to the higher id
    ///   - for instance, label has 3 clips, then the first clip will have a position of 0
    pub async fn get_label_clip_id_with_pos(
        &self,
        label: &str,
        pos: u64,
    ) -> Result<Option<u64>, anyhow::Error> {
        let db_connection = self.database_connection.lock().await;

        let res = match db_connection.query_row(
            &format!(
                "SELECT id FROM {} ORDER BY id ASC LIMIT 1 OFFSET ?",
                label_name_to_table_name(label)
            ),
            [pos.to_string()],
            |row| row.get(0),
        ) {
            Ok(res) => res,
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(err) => {
                return Err(err.into());
            }
        };

        Ok(res)
    }

    /// add or delete a label to a clip
    ///   - if target is true, add the label to the clip
    ///   - if target is false, delete the label from the clip
    ///   - trigger a tray update event
    pub async fn change_clip_label(
        &self,
        id: u64,
        label: &str,
        target: bool,
    ) -> Result<(), anyhow::Error> {
        let db_connection = self.database_connection.lock().await;

        // change the clip in the database
        // wait for any error
        if target {
            // add the label to the clip
            db_connection.execute(
                &format!(
                    "INSERT OR IGNORE INTO {} (id) VALUES (?)",
                    label_name_to_table_name(label)
                ),
                [id.to_string()],
            )?;
        } else {
            // remove the label from the clip
            db_connection.execute(
                &format!(
                    "DELETE FROM {} WHERE id = ?",
                    label_name_to_table_name(label),
                ),
                [id.to_string()],
            )?;
        };

        Ok(())
    }

    /// Get all the labels for a clip in the database
    ///   - return a vector of labels
    ///   - return None if the clip is not found
    pub async fn get_clip_labels(&self, id: u64) -> Result<Option<Vec<String>>, anyhow::Error> {
        let all_labels = self.get_all_labels().await?;

        let db_connection = self.database_connection.lock().await;

        let mut labels: Vec<String> = Vec::new();
        for label in all_labels {
            let mut statement = db_connection.prepare(&format!(
                "SELECT * FROM {} WHERE id = ?",
                label_name_to_table_name(&label)
            ))?;
            match statement.query_row([id.to_string()], |_| Ok(())) {
                Ok(_) => labels.push(label),
                Err(rusqlite::Error::QueryReturnedNoRows) => (),
                Err(err) => return Err(err.into()),
            };
        }

        Ok(Some(labels))
    }

    /// Get a clip by id from database
    ///
    /// Get the latest clip if the id is none.
    /// None if the clip is not found.
    #[warn(unused_must_use)]
    pub async fn get_clip(&self, id: Option<u64>) -> Result<Option<Clip>, anyhow::Error> {
        debug!("Get clip: {:?}", id);
        // if id is none, change it to the latest clip
        let id = match id {
            Some(id) => id,
            None => match self.get_latest_clip_id().await? {
                Some(id) => id,
                None => return Ok(None),
            },
        };

        // get the clip from the database
        let db_connection = self.database_connection.lock().await;

        debug!("Starting to query the clip");
        let mut res = match db_connection.query_row(
            "SELECT id, type, data, timestamp, search_text FROM clips WHERE id = ?",
            [id],
            Clip::from_database_row,
        ) {
            Ok(res) => res,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        drop(db_connection);

        let labels = (self.get_clip_labels(id).await?).unwrap_or_default();

        res.labels = labels;

        Ok(Some(res))
    }

    /// Get the position of the id in the database counting from the lower id to the higher id.
    /// The lowest id will return 0.
    /// The highest id will return the length of the list - 1.
    ///
    /// If the id is not in the list, return None.
    /// If the id is none, return the position of the latest clip.
    pub async fn get_id_pos_in_whole_list_of_ids(
        &self,
        id: Option<u64>,
    ) -> Result<Option<u64>, anyhow::Error> {
        let id = match id {
            Some(id) => id,
            None => match self.get_latest_clip_id().await? {
                Some(id) => id,
                None => return Ok(None),
            },
        };

        let db_connection = self.database_connection.lock().await;

        let res: u64 = match db_connection.query_row(
            "SELECT COUNT(1) FROM clips WHERE id <= ?",
            [id.to_string()],
            |row| row.get(0),
        ) {
            Ok(res) => res,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        Ok(Some(res - 1))
    }

    /// Create a new clip in the database and return the id of the new clip
    pub async fn new_clip(
        &self,
        clip: &mut Clip,
        auto_delete_duplicate_clip: bool,
    ) -> Result<u64, anyhow::Error> {
        debug!("Create a new clip");

        let db_connection = self.database_connection.lock().await;

        let id: u64 = db_connection.query_row(
            "INSERT INTO clips (data, timestamp, type, search_text)
            VALUES (?, ?, ?, ?)
            RETURNING id",
            [
                clip.get_data_database(),
                clip.get_timestamp_database(),
                clip.get_clip_type_database(),
                clip.get_search_text_database(),
            ],
            |row| row.get("id"),
        )?;
        clip.id = id;

        // see if we need to auto delete duplicate clip
        if !auto_delete_duplicate_clip {
            debug!("Auto delete duplicate clip is disabled");
            return Ok(id);
        }

        debug!("Start auto delete duplicate clip");
        db_connection.execute(
            "DELETE FROM clips WHERE data = ? AND id != ?",
            [clip.get_data_database(), clip.get_id_database()],
        )?;

        Ok(id)
    }
}
