use std::collections::HashMap;

use regex::Regex;
use sublime_fuzzy::best_match;
use tauri::{AppHandle, Manager};

use crate::{config::ConfigMutex, error};

use super::{Clip, ClipDataMutex};

use sqlx::{sqlite::SqliteRow, Row};

/// search for a clip in the database
/// SELECT * FROM enrondata1 WHERE content MATCH 'linux';

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

    let favorite = row.try_get::<i64, _>("favorite");
    if let Err(err) = favorite {
        return Err(error::Error::GetClipDataFromDatabaseErr(
            id,
            err.to_string(),
        ));
    }
    let favorite = favorite.unwrap();
    let favorite = favorite == 1;

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
        text,
        favorite,
        timestamp,
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
    favorite: i64,
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().await;
    let res = if favorite == -1 {
        sqlx::query(
            "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text MATCH ? ORDER BY id DESC LIMIT ?",
            )
            .bind(&min_id)
            .bind(&max_id)
            .bind(&data)
            .bind(&limit)
            .fetch_all(clip_data.database_connection.as_ref().unwrap()).await
    } else {
        sqlx::query("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favorite = ? AND text MATCH ? ORDER BY id DESC LIMIT ?").bind(&min_id).bind(&max_id).bind(&favorite).bind(&data).bind(&limit)
            .fetch_all(clip_data.database_connection.as_ref().unwrap()).await
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

/// normal search for a clip in the database
/// SELECT count(*) FROM enrondata2 WHERE content LIKE '%linux%'
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
    favorite: i64,
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().await;
    let res = if favorite == -1 {
        sqlx::query(
            "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text Like ? ORDER BY id DESC LIMIT ?",
        )
        .bind(&min_id)
        .bind(&max_id)
        .bind("%".to_string() + data.as_str() + "%")
        .bind(&limit)
        .fetch_all(clip_data.database_connection.as_ref().unwrap())
        .await
    } else {
        sqlx::query(            "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favorite = ? AND text Like ? ORDER BY id DESC LIMIT ?",
).bind(&min_id).bind(&max_id).bind(&favorite).bind("%".to_string() + data.as_str() + "%").bind(&limit)
            .fetch_all(clip_data.database_connection.as_ref().unwrap()).await
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
    favorite: i64, // 0: not favorite, 1: favorite, -1: all
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().await;

    let mut max_id = max_id;
    let mut count = 0;
    let mut clips = HashMap::new();

    while max_id >= min_id && count < limit {
        let res = if favorite == -1 {
            let res = sqlx::query(
                "SELECT * FROM clips WHERE id BETWEEN ? AND ? ORDER BY id DESC LIMIT 1",
            )
            .bind(&min_id)
            .bind(&max_id)
            .fetch_all(clip_data.database_connection.as_ref().unwrap())
            .await;

            res
        } else {
            let res = sqlx::query(
                "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favorite = ? ORDER BY id DESC LIMIT 1",
            )
            .bind(&min_id)
            .bind(&max_id)
            .bind(&favorite)
            .fetch_all(clip_data.database_connection.as_ref().unwrap())
            .await;

            res
        };

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }
        let res = res.unwrap();
        if res.len() == 0 {
            break;
        }

        for row in res {
            let clip = clip_from_row(row)?;

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
    favorite: i64, // 0: not favorite, 1: favorite, -1: all
) -> Result<HashMap<i64, Clip>, error::Error> {
    let re = Regex::new(&data);
    if let Err(err) = re {
        return Err(error::Error::RegexpErr(err.to_string()));
    }
    let re = re.unwrap();

    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().await;

    let mut max_id = max_id;
    let mut count = 0;
    let mut clips = HashMap::new();

    while max_id >= min_id && count < limit {
        let res = if favorite == -1 {
            let res = sqlx::query(
                "SELECT * FROM clips WHERE id BETWEEN ? AND ? ORDER BY id DESC LIMIT 1",
            )
            .bind(&min_id)
            .bind(&max_id)
            .fetch_all(clip_data.database_connection.as_ref().unwrap())
            .await;

            res
        } else {
            let res = sqlx::query(
                "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favorite = ? ORDER BY id DESC LIMIT 1",
            )
            .bind(&max_id)
            .bind(&min_id)
            .bind(&favorite)
            .fetch_all(clip_data.database_connection.as_ref().unwrap())
            .await;

            res
        };

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }
        let res = res.unwrap();
        if res.len() == 0 {
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

/// get the max id of the clip in the database,
/// if no clip in the database, return 0
#[tauri::command]
pub async fn get_max_id(clip_data: tauri::State<'_, ClipDataMutex>) -> Result<i64, error::Error> {
    let clip_data = clip_data.clip_data.lock().await;
    let res = clip_data.clips.whole_list_of_ids.last();
    if res.is_none() {
        return Ok(0);
    }
    let res = res.unwrap();
    Ok(*res)
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
    favorite: i64, // 0: not favorite, 1: favorite, -1: all
    searchmethod: String,
) -> Result<HashMap<i64, Clip>, String> {
    let config = app.state::<ConfigMutex>();
    let config = config.config.lock().await;
    let limit = config.search_clip_per_batch;
    drop(config);

    match searchmethod.as_str() {
        "fuzzy" => {
            let res = fuzzy_search(&app, minid, maxid, limit, data, favorite).await;
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "fast" => {
            let res = fast_search(&app, minid, maxid, limit, data, favorite).await;
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "normal" => {
            let res = normal_search(&app, minid, maxid, limit, data, favorite).await;
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "regexp" => {
            let res = regexp_search(&app, minid, maxid, limit, data, favorite).await;
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        _ => Err("invalid search method".to_string()),
    }
}
