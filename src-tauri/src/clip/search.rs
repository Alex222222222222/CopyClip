use std::{collections::HashMap, sync::Arc};

use log::debug;
use rusqlite::Row;

use tauri::{AppHandle, Manager};

use crate::{
    config::ConfigMutex,
    database::{label_name_to_table_name, DatabaseStateMutex},
    error,
};

use clip::Clip;

use super::clip_data::ClipStateMutex;

/// search for a clip in the database
/// SELECT * FROM clips WHERE content MATCH 'linux';
///
/// get the clip data from a sqlite row
/// this function will not test if the row is valid
///
/// as the row will not contain info about the labels, the clip will not have any label
#[warn(unused_must_use)]
fn clip_from_row(row: &Row) -> Result<Clip, rusqlite::Error> {
    let id = row.get("id")?;
    let text = row.get("text")?;
    let timestamp: i64 = row.get("timestamp")?;
    let clip_type: u8 = row.get("type")?;

    let clip = Clip {
        id,
        text: Arc::new(text),
        timestamp,
        clip_type: clip_type.into(),
        labels: vec![],
    };

    Ok(clip)
}

/// normal search for a clip in the database
/// SELECT count(*) FROM clips WHERE content LIKE '%linux%'
/// will search for substring
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
/// will return a list of clips
pub async fn normal_search(
    app: &AppHandle,
    min_id: u64,
    max_id: u64,
    limit: u64,
    data: String,
    favourite: bool,
    pinned: bool,
) -> Result<HashMap<u64, Clip>, error::Error> {
    let db_connection = app.state::<DatabaseStateMutex>();
    let db_connection = db_connection.database_connection.lock().await;

    let mut stmt: String =
        "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text LIKE ?".to_string();

    if favourite {
        stmt = format!(
            "{}
            INNER JOIN {}
            ON clips.id = {}.id;
            ",
            stmt,
            label_name_to_table_name("favourite"),
            label_name_to_table_name("favourite")
        );
    }

    if pinned {
        stmt = format!(
            "{}
            INNER JOIN {}
            ON clips.id = {}.id;
            ",
            stmt,
            label_name_to_table_name("pinned"),
            label_name_to_table_name("pinned")
        );
    }

    let stmt = format!(
        "{}
        ORDER BY id DESC LIMIT ?",
        stmt
    );

    let mut stmt = match db_connection.prepare(&stmt) {
        Ok(stmt) => stmt,
        Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
    };

    let data = format!("%{}%", data);

    let res = match stmt.query_map(
        [
            min_id.to_string(),
            max_id.to_string(),
            data,
            limit.to_string(),
        ],
        clip_from_row,
    ) {
        Ok(res) => res,
        Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
    };

    let mut clips = HashMap::new();
    for clip in res {
        let clip = match clip {
            Ok(clip) => clip,
            Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
        };
        clips.insert(clip.id, clip);
    }

    Ok(clips)
}

/// fuzzy search for a clip in the database
/// try iterate with each clip in the database such that the id is between min_id and max_id and maximum limit clips
/// will return a list of clips
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
pub async fn fuzzy_search(
    app: &AppHandle,
    min_id: u64,
    max_id: u64,
    limit: u64,
    data: String,
    favourite: bool, // true: filter-on, false: filter-off
    pinned: bool,
) -> Result<HashMap<u64, Clip>, error::Error> {
    let db_connection = app.state::<DatabaseStateMutex>();
    let db_connection = db_connection.database_connection.lock().await;

    let mut stmt: String =
        "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND fuzzy_search(text, ?) > 0".to_string();

    if favourite {
        stmt = format!(
            "{}
            INNER JOIN {}
            ON clips.id = {}.id;
            ",
            stmt,
            label_name_to_table_name("favourite"),
            label_name_to_table_name("favourite")
        );
    }

    if pinned {
        stmt = format!(
            "{}
            INNER JOIN {}
            ON clips.id = {}.id;
            ",
            stmt,
            label_name_to_table_name("pinned"),
            label_name_to_table_name("pinned")
        );
    }

    let stmt = format!(
        "{}
        ORDER BY id DESC LIMIT ?",
        stmt
    );

    let mut stmt = match db_connection.prepare(&stmt) {
        Ok(stmt) => stmt,
        Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
    };

    let res = match stmt.query_map(
        [
            min_id.to_string(),
            max_id.to_string(),
            data,
            limit.to_string(),
        ],
        clip_from_row,
    ) {
        Ok(res) => res,
        Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
    };

    let mut clips = HashMap::new();
    for clip in res {
        let clip = match clip {
            Ok(clip) => clip,
            Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
        };
        clips.insert(clip.id, clip);
    }

    Ok(clips)
}

/// regexp search for a clip in the database
/// try iterate with each clip in the database such that the id is between min_id and max_id and maximum limit clips
/// will return a list of clips
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
pub async fn regexp_search(
    app: &AppHandle,
    min_id: u64,
    max_id: u64,
    limit: u64,
    data: String,
    favourite: bool, // true: filter-on, false: filter-off
    pinned: bool,
) -> Result<HashMap<u64, Clip>, error::Error> {
    let db_connection = app.state::<DatabaseStateMutex>();
    let db_connection = db_connection.database_connection.lock().await;

    let mut stmt: String =
        "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND regexp(text, ?)".to_string();

    if favourite {
        stmt = format!(
            "{}
            INNER JOIN {}
            ON clips.id = {}.id;
            ",
            stmt,
            label_name_to_table_name("favourite"),
            label_name_to_table_name("favourite")
        );
    }

    if pinned {
        stmt = format!(
            "{}
            INNER JOIN {}
            ON clips.id = {}.id;
            ",
            stmt,
            label_name_to_table_name("pinned"),
            label_name_to_table_name("pinned")
        );
    }

    let stmt = format!(
        "{}
        ORDER BY id DESC LIMIT ?",
        stmt
    );

    let mut stmt = match db_connection.prepare(&stmt) {
        Ok(stmt) => stmt,
        Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
    };

    let res = match stmt.query_map(
        [
            min_id.to_string(),
            max_id.to_string(),
            data,
            limit.to_string(),
        ],
        clip_from_row,
    ) {
        Ok(res) => res,
        Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
    };

    let mut clips = HashMap::new();
    for clip in res {
        let clip = match clip {
            Ok(clip) => clip,
            Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
        };
        clips.insert(clip.id, clip);
    }

    Ok(clips)
}

/// Return all the clips in the database with id, min_id <= id <= max_id.
/// The total number of clips is limited to limit.
async fn empty_search(
    app: &AppHandle,
    min_id: u64,
    max_id: u64,
    limit: u64,
    favourite: bool, // true: filter-on, false: filter-off
    pinned: bool,
) -> Result<HashMap<u64, Clip>, error::Error> {
    let db_connection = app.state::<DatabaseStateMutex>();
    let db_connection = db_connection.database_connection.lock().await;

    let mut stmt: String = "SELECT * FROM clips WHERE id BETWEEN ? AND ?".to_string();

    if favourite {
        stmt = format!(
            "{}
            INNER JOIN {}
            ON clips.id = {}.id;
            ",
            stmt,
            label_name_to_table_name("favourite"),
            label_name_to_table_name("favourite")
        );
    }

    if pinned {
        stmt = format!(
            "{}
            INNER JOIN {}
            ON clips.id = {}.id;
            ",
            stmt,
            label_name_to_table_name("pinned"),
            label_name_to_table_name("pinned")
        );
    }

    let stmt = format!(
        "{}
        ORDER BY id DESC LIMIT ?",
        stmt
    );

    let mut stmt = match db_connection.prepare(&stmt) {
        Ok(stmt) => stmt,
        Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
    };

    let res = match stmt.query_map(
        [min_id.to_string(), max_id.to_string(), limit.to_string()],
        clip_from_row,
    ) {
        Ok(res) => res,
        Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
    };

    let mut clips = HashMap::new();
    for clip in res {
        let clip = match clip {
            Ok(clip) => clip,
            Err(err) => return Err(error::Error::GetClipDataFromDatabaseErr(0, err.to_string())),
        };
        clips.insert(clip.id, clip);
    }

    Ok(clips)
}

/// get the max id of the clip in the database,
/// if no clip in the database, return 0
#[tauri::command]
pub async fn get_max_id(
    app: AppHandle,
    clip_state: tauri::State<'_, ClipStateMutex>,
) -> Result<u64, error::Error> {
    let clip_data = clip_state.clip_state.lock().await;
    let res = clip_data.get_latest_clip_id(&app).await?;
    if res.is_none() {
        return Ok(0);
    }

    Ok(res.unwrap())
}

/// search for a clip in the database
///
/// the method is decide by the input
/// the limit is the config.search_clip_per_page
///
/// input {
///     data: String,
///     min_id: i64,
///     max_id: i64,
///     search_method: String,
/// }
///
/// output {
///     HashMap<id, Clip>
/// }
#[tauri::command]
pub async fn search_clips(
    app: AppHandle,
    data: String,
    minid: i64,
    maxid: u64,
    favourite: bool, // true: filter-on, false: filter-off
    pinned: bool,
    searchmethod: String,
) -> Result<HashMap<u64, Clip>, String> {
    debug!(
        "search_clips: data: {}, minid: {}, maxid: {}, searchmethod: {}",
        data, minid, maxid, searchmethod
    );
    let config = app.state::<ConfigMutex>();
    let config = config.config.lock().await;
    let limit = config.search_clip_per_batch;
    drop(config);
    let clip_state = app.state::<ClipStateMutex>();
    let minid = if minid < 0 {
        match clip_state
            .clip_state
            .lock()
            .await
            .get_latest_clip_id(&app)
            .await
        {
            Ok(res) => res.unwrap_or(0),
            Err(err) => {
                return Err(err.message());
            }
        }
    } else {
        minid as u64
    };

    // if data is empty, return all clips
    let mut res = if data.is_empty() {
        debug!("search_clips: empty search");
        let res = empty_search(&app, minid, maxid, limit, favourite, pinned).await;
        if let Err(err) = res {
            return Err(err.message());
        }
        res.unwrap()
    } else {
        match searchmethod.as_str() {
            "fuzzy" => {
                let res = fuzzy_search(&app, minid, maxid, limit, data, favourite, pinned).await;
                if let Err(err) = res {
                    return Err(err.message());
                }
                res.unwrap()
            }
            "normal" => {
                let res = normal_search(&app, minid, maxid, limit, data, favourite, pinned).await;
                if let Err(err) = res {
                    return Err(err.message());
                }
                res.unwrap()
            }
            "regexp" => {
                let res = regexp_search(&app, minid, maxid, limit, data, favourite, pinned).await;
                if let Err(err) = res {
                    return Err(err.message());
                }
                res.unwrap()
            }
            _ => return Err("invalid search method".to_string()),
        }
    };

    for (id, clip) in res.iter_mut() {
        clip.labels = match clip_state
            .clip_state
            .lock()
            .await
            .get_clip_labels(&app, *id)
            .await
        {
            Ok(labels) => match labels {
                Some(labels) => labels,
                None => vec![],
            },
            Err(err) => {
                return Err(err.message());
            }
        }
    }

    Ok(res)
}
