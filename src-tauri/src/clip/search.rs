use std::{collections::HashMap, sync::Arc};

use log::debug;
use regex::Regex;
use sublime_fuzzy::best_match;
use tauri::{AppHandle, Manager};

use crate::{config::ConfigMutex, error};

use super::{clip_data::ClipData, clip_struct::Clip};

use sqlx::{sqlite::SqliteRow, Row};

/// search for a clip in the database
/// SELECT * FROM clips WHERE content MATCH 'linux';
///
/// get the clip data from a sqlite row
/// this function will not test if the row is valid
#[warn(unused_must_use)]
fn clip_from_row(row: SqliteRow) -> Result<Clip, error::Error> {
    let id = row.try_get::<i64, _>("id");
    if let Err(err) = id {
        return Err(error::Error::GetClipDataFromDatabaseErr(
            -1,
            err.to_string(),
        ));
    }
    let id = id.unwrap();

    let text = row.try_get::<String, _>("text");
    if let Err(err) = text {
        return Err(error::Error::GetClipDataFromDatabaseErr(
            id,
            err.to_string(),
        ));
    }
    let text = text.unwrap();

    let favourite = row.try_get::<i64, _>("favourite");
    if let Err(err) = favourite {
        return Err(error::Error::GetClipDataFromDatabaseErr(
            id,
            err.to_string(),
        ));
    }
    let favourite = favourite.unwrap();
    let favourite = favourite == 1;

    let pinned = row.try_get::<i64, _>("pinned");
    if let Err(err) = pinned {
        return Err(error::Error::GetClipDataFromDatabaseErr(
            id,
            err.to_string(),
        ));
    }
    let pinned = pinned.unwrap();
    let pinned = pinned == 1;

    let timestamp = row.try_get::<i64, _>("timestamp");
    if let Err(err) = timestamp {
        return Err(error::Error::GetClipDataFromDatabaseErr(
            id,
            err.to_string(),
        ));
    }
    let timestamp = timestamp.unwrap();

    let clip = Clip {
        id,
        text: Arc::new(text),
        favourite,
        timestamp,
        pinned,
    };

    Ok(clip)
}

/// fast search for a clip in the database
/// using the fts4 table search, will be faster however, for install data = "install" and a clip = "installapp"
/// then this clip will not be in the search result.
/// fts4 only searches for as a discrete token, not as a substring
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
/// will return a list of clips
pub async fn fast_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
    favourite: i64,
) -> Result<HashMap<i64, Clip>, error::Error> {
    let db_connection_mutex = app.state::<ClipData>();
    let db_connection_mutex = db_connection_mutex.database_connection.lock().await;
    let db_connection = db_connection_mutex.clone().unwrap();
    drop(db_connection_mutex);
    let res = if favourite == -1 {
        sqlx::query(
            "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text MATCH ? ORDER BY id DESC LIMIT ?",
            )
            .bind(min_id)
            .bind(max_id)
            .bind(&data)
            .bind(limit)
            .fetch_all(db_connection.as_ref()).await
    } else {
        sqlx::query("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favourite = ? AND text MATCH ? ORDER BY id DESC LIMIT ?").bind(min_id).bind(max_id).bind(favourite).bind(&data).bind(limit)
            .fetch_all(db_connection.as_ref()).await
    };

    if let Err(err) = res {
        debug!(
            "failed to get clip from row, error message: {}",
            err.to_string()
        );
        return Err(error::Error::GetClipDataFromDatabaseErr(
            -1,
            err.to_string(),
        ));
    }
    let mut clips = HashMap::new();
    for row in res.unwrap() {
        let clip = clip_from_row(row);
        if let Err(err) = clip {
            debug!(
                "failed to get clip from row, error message: {}",
                err.to_string()
            );
            return Err(err);
        }
        let clip = clip.unwrap();
        clips.insert(clip.id, clip);
    }

    Ok(clips)
}

/// normal search for a clip in the database
/// SELECT count(*) FROM clips WHERE content LIKE '%linux%'
/// will search for substring
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
/// will return a list of clips
pub async fn normal_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
    favourite: i64,
) -> Result<HashMap<i64, Clip>, error::Error> {
    let db_connection_mutex = app.state::<ClipData>();
    let db_connection_mutex = db_connection_mutex.database_connection.lock().await;
    let db_connection = db_connection_mutex.clone().unwrap();
    drop(db_connection_mutex);
    let res = if favourite == -1 {
        sqlx::query(
            "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text Like ? ORDER BY id DESC LIMIT ?",
        )
        .bind(min_id)
        .bind(max_id)
        .bind("%".to_string() + data.as_str() + "%")
        .bind(limit)
        .fetch_all(db_connection.as_ref())
        .await
    } else {
        sqlx::query(            "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favourite = ? AND text Like ? ORDER BY id DESC LIMIT ?",
).bind(min_id).bind(max_id).bind(favourite).bind("%".to_string() + data.as_str() + "%").bind(limit)
            .fetch_all(db_connection.as_ref()).await
    };

    if let Err(err) = res {
        return Err(error::Error::GetClipDataFromDatabaseErr(
            -1,
            err.to_string(),
        ));
    }
    let mut clips = HashMap::new();
    for row in res.unwrap() {
        let clip = clip_from_row(row)?;
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
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
    favourite: i64, // 0: not favourite, 1: favourite, -1: all
) -> Result<HashMap<i64, Clip>, error::Error> {
    let db_connection_mutex = app.state::<ClipData>();
    let db_connection_mutex = db_connection_mutex.database_connection.lock().await;
    let db_connection = db_connection_mutex.clone().unwrap();
    drop(db_connection_mutex);

    let mut max_id = max_id;
    let mut count = 0;
    let mut clips = HashMap::new();

    while max_id >= min_id && count < limit {
        let res = if favourite == -1 {
            sqlx::query("SELECT * FROM clips WHERE id BETWEEN ? AND ? ORDER BY id DESC LIMIT 1")
                .bind(min_id)
                .bind(max_id)
                .fetch_all(db_connection.as_ref())
                .await
        } else {
            sqlx::query(
                "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favourite = ? ORDER BY id DESC LIMIT 1",
            )
            .bind(min_id)
            .bind(max_id)
            .bind(favourite)
            .fetch_all(db_connection.as_ref())
            .await
        };

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }
        let res = res.unwrap();
        if res.is_empty() {
            break;
        }

        for row in res {
            let clip = clip_from_row(row)?;

            // if the text is too long do not do fuzzy search
            // TODO change this to user configurable
            if clip.text.len() > 5000 {
                max_id = clip.id - 1;
                continue;
            }
            let result = best_match(&data, &clip.text);
            if result.is_none() {
                max_id = clip.id - 1;
                continue;
            }
            let result = result.unwrap();
            if result.score() <= 0 {
                max_id = clip.id - 1;
                continue;
            }
            max_id = clip.id - 1;
            clips.insert(clip.id, clip);
            count += 1;
        }
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
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
    favourite: i64, // 0: not favourite, 1: favourite, -1: all
) -> Result<HashMap<i64, Clip>, error::Error> {
    let re = Regex::new(&data);
    if let Err(err) = re {
        return Err(error::Error::RegexpErr(err.to_string()));
    }
    let re = re.unwrap();

    let db_connection_mutex = app.state::<ClipData>();
    let db_connection_mutex = db_connection_mutex.database_connection.lock().await;
    let db_connection = db_connection_mutex.clone().unwrap();
    drop(db_connection_mutex);

    let mut max_id = max_id;
    let mut count = 0;
    let mut clips = HashMap::new();

    while max_id >= min_id && count < limit {
        let res = if favourite == -1 {
            sqlx::query("SELECT * FROM clips WHERE id BETWEEN ? AND ? ORDER BY id DESC LIMIT 1")
                .bind(min_id)
                .bind(max_id)
                .fetch_all(db_connection.as_ref())
                .await
        } else {
            sqlx::query(
                "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favourite = ? ORDER BY id DESC LIMIT 1",
            )
            .bind(max_id)
            .bind(min_id)
            .bind(favourite)
            .fetch_all(db_connection.as_ref())
            .await
        };

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }
        let res = res.unwrap();
        if res.is_empty() {
            break;
        }

        for row in res {
            let clip = clip_from_row(row)?;

            let result = re.is_match(&clip.text);
            if !result {
                max_id = clip.id - 1;
                continue;
            }

            max_id = clip.id - 1;
            clips.insert(clip.id, clip);
            count += 1;
        }
    }

    Ok(clips)
}

/// Return all the clips in the database with id, min_id <= id <= max_id.
/// The total number of clips is limited to limit.
/// If favourite is 1, only return favourite clips.
/// If favourite is 0, only return non-favourite clips.
/// If favourite is -1, return all clips.
async fn empty_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    favourite: i64, // 0: not favourite, 1: favourite, -1: all
) -> Result<HashMap<i64, Clip>, error::Error> {
    let db_connection_mutex = app.state::<ClipData>();
    let db_connection_mutex = db_connection_mutex.database_connection.lock().await;
    let db_connection = db_connection_mutex.clone().unwrap();
    drop(db_connection_mutex);

    let res = if favourite == -1 {
        sqlx::query("SELECT * FROM clips WHERE id BETWEEN ? AND ? ORDER BY id DESC LIMIT ?")
            .bind(min_id)
            .bind(max_id)
            .bind(limit)
            .fetch_all(db_connection.as_ref())
            .await
    } else {
        sqlx::query(
            "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favourite = ? ORDER BY id DESC LIMIT ?",
        )
        .bind(min_id)
        .bind(max_id)
        .bind(favourite)
        .bind(limit)
        .fetch_all(db_connection.as_ref())
        .await
    };

    if let Err(err) = res {
        return Err(error::Error::GetClipDataFromDatabaseErr(
            -1,
            err.to_string(),
        ));
    }

    let res = res.unwrap();
    let mut clips = HashMap::new();
    for row in res {
        let clip = clip_from_row(row)?;
        clips.insert(clip.id, clip);
    }

    Ok(clips)
}

/// get the max id of the clip in the database,
/// if no clip in the database, return 0
#[tauri::command]
pub async fn get_max_id(clip_data: tauri::State<'_, ClipData>) -> Result<i64, error::Error> {
    let res = clip_data.get_latest_clip_id().await;
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
    maxid: i64,
    favourite: i64, // 0: not favourite, 1: favourite, -1: all
    searchmethod: String,
) -> Result<HashMap<i64, Clip>, String> {
    let config = app.state::<ConfigMutex>();
    let config = config.config.lock().await;
    let limit = config.search_clip_per_batch;
    drop(config);

    // if data is empty, return all clips
    if data.is_empty() {
        let res = empty_search(&app, minid, maxid, limit, favourite).await;
        if let Err(err) = res {
            return Err(err.message());
        }
        return Ok(res.unwrap());
    }

    match searchmethod.as_str() {
        "fuzzy" => {
            let res = fuzzy_search(&app, minid, maxid, limit, data, favourite).await;
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "fast" => {
            let res = fast_search(&app, minid, maxid, limit, data, favourite).await;
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "normal" => {
            let res = normal_search(&app, minid, maxid, limit, data, favourite).await;
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "regexp" => {
            let res = regexp_search(&app, minid, maxid, limit, data, favourite).await;
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        _ => Err("invalid search method".to_string()),
    }
}
