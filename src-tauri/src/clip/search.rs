use std::collections::HashMap;

use sublime_fuzzy::best_match;
use tauri::{AppHandle, Manager};

use crate::{config::ConfigMutex, error};

use super::{Clip, ClipDataMutex};

/// search for a clip in the database
/// SELECT * FROM enrondata1 WHERE content MATCH 'linux';

/// fast search for a clip in the database
/// using the fts4 table search, will be faster however, for install data = "install" and a clip = "installapp"
/// then this clip will not be in the search result.
/// fts4 only searches for as a discrete token, not as a substring
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
/// will return a list of clips
pub fn fast_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
    favorite: i64,
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().unwrap();
    let res = if favorite == -1 {
        let mut statement = clip_data
            .database_connection
            .as_ref()
            .unwrap()
            .prepare("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text MATCH ? ORDER BY id DESC LIMIT ?")
            .unwrap();
        let res = statement.query_map(
            [
                min_id.to_string(),
                max_id.to_string(),
                data,
                limit.to_string(),
            ],
            |row| {
                let favorite: i64 = row.get(3)?;
                Ok(Clip {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    favorite: favorite == 1,
                })
            },
        );

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }

        let res = res.unwrap();
        let mut clips = HashMap::new();

        for clip in res {
            if let Err(err) = clip {
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.to_string(),
                ));
            }
            let clip = clip.unwrap();
            clips.insert(clip.id, clip);
        }

        clips
    } else {
        let mut statement = clip_data
            .database_connection
            .as_ref()
            .unwrap()
            .prepare("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favorite = ? AND text MATCH ? ORDER BY id DESC LIMIT ?")
            .unwrap();
        let res = statement.query_map(
            [
                min_id.to_string(),
                max_id.to_string(),
                favorite.to_string(),
                data,
                limit.to_string(),
            ],
            |row| {
                let favorite: i64 = row.get(3)?;
                Ok(Clip {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    favorite: favorite == 1,
                })
            },
        );

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }

        let res = res.unwrap();
        let mut clips = HashMap::new();

        for clip in res {
            if let Err(err) = clip {
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.to_string(),
                ));
            }
            let clip = clip.unwrap();
            clips.insert(clip.id, clip);
        }

        clips
    };

    println!("fast search result: {:?}", res);

    Ok(res)
}

/// normal search for a clip in the database
/// SELECT count(*) FROM enrondata2 WHERE content LIKE '%linux%'
/// will search for substring
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
/// will return a list of clips
pub fn normal_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
    favorite: i64,
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().unwrap();
    let res = if favorite == -1 {
        let mut statement = clip_data
            .database_connection
            .as_ref()
            .unwrap()
            .prepare(
                "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text Like ? ORDER BY id DESC LIMIT ?",
            )
            .unwrap();
        let res = statement.query_map(
            [
                min_id.to_string(),
                max_id.to_string(),
                "%".to_string() + data.as_str() + "%",
                limit.to_string(),
            ],
            |row| {
                let favorite: i64 = row.get(3)?;
                Ok(Clip {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    favorite: favorite == 1,
                })
            },
        );

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }

        let res = res.unwrap();

        let mut clips = HashMap::new();

        for clip in res {
            if let Err(err) = clip {
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.to_string(),
                ));
            }
            let clip = clip.unwrap();
            clips.insert(clip.id, clip);
        }

        clips
    } else {
        let mut statement = clip_data
            .database_connection
            .as_ref()
            .unwrap()
            .prepare(
                "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favorite = ? AND text Like ? ORDER BY id DESC LIMIT ?",
            )
            .unwrap();

        let res = statement.query_map(
            [
                min_id.to_string(),
                max_id.to_string(),
                favorite.to_string(),
                "%".to_string() + data.as_str() + "%",
                limit.to_string(),
            ],
            |row| {
                let favorite: i64 = row.get(3)?;
                Ok(Clip {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    favorite: favorite == 1,
                })
            },
        );

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }

        let res = res.unwrap();

        let mut clips = HashMap::new();

        for clip in res {
            if let Err(err) = clip {
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.to_string(),
                ));
            }
            let clip = clip.unwrap();
            clips.insert(clip.id, clip);
        }

        clips
    };

    Ok(res)
}

/// fuzzy search for a clip in the database
/// try iterate with each clip in the database such that the id is between min_id and max_id and maximum limit clips
/// will return a list of clips
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
pub fn fuzzy_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
    favorite: i64, // 0: not favorite, 1: favorite, -1: all
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().unwrap();

    let mut max_id = max_id;
    let mut count = 0;
    let mut clips = HashMap::new();

    while max_id >= min_id && count < limit {
        let clip = if favorite == -1 {
            let mut statement = clip_data
                .database_connection
                .as_ref()
                .unwrap()
                .prepare("SELECT * FROM clips WHERE id BETWEEN ? AND ? ORDER BY id DESC LIMIT 1")
                .unwrap();
            let res = statement.query_row([min_id.to_string(), max_id.to_string()], |row| {
                let favorite: i64 = row.get(3)?;
                Ok(Clip {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    favorite: favorite == 1,
                })
            });

            if let Err(err) = res {
                if err == rusqlite::Error::QueryReturnedNoRows {
                    max_id -= 1;
                    continue;
                }
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.to_string(),
                ));
            }

            res.unwrap()
        } else {
            let mut statement = clip_data
                .database_connection
                .as_ref()
                .unwrap()
                .prepare(
                    "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favorite = ? ORDER BY id DESC LIMIT 1",
                )
                .unwrap();

            let res = statement.query_row(
                [min_id.to_string(), max_id.to_string(), favorite.to_string()],
                |row| {
                    let favorite: i64 = row.get(3)?;
                    Ok(Clip {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        timestamp: row.get(2)?,
                        favorite: favorite == 1,
                    })
                },
            );

            if let Err(err) = res {
                if err == rusqlite::Error::QueryReturnedNoRows {
                    max_id -= 1;
                    continue;
                }
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.to_string(),
                ));
            }

            res.unwrap()
        };

        let id = clip.id;

        let result = best_match(&data, &clip.text);
        if result.is_none() {
            max_id = id - 1;
            continue;
        }
        let result = result.unwrap();
        if result.score() <= 0 {
            max_id = id - 1;
            continue;
        }
        clips.insert(clip.id, clip);
        max_id = id - 1;
        count += 1;
    }

    Ok(clips)
}

/// regexp search for a clip in the database
/// SELECT count(*) FROM enrondata2 WHERE content LIKE '%linux%'
/// will search for substring
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
/// will return a list of clips
pub fn regexp_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
    favorite: i64,
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().unwrap();
    let res = if favorite == -1 {
        let mut statement = clip_data
            .database_connection
            .as_ref()
            .unwrap()
            .prepare("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text REGEXP ? ORDER BY id DESC LIMIT ?")
            .unwrap();
        let res = statement.query_map(
            [
                min_id.to_string(),
                max_id.to_string(),
                data,
                limit.to_string(),
            ],
            |row| {
                let favorite: i64 = row.get(3)?;
                Ok(Clip {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    favorite: favorite == 1,
                })
            },
        );

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }

        let res = res.unwrap();
        let mut clips = HashMap::new();

        for clip in res {
            if let Err(err) = clip {
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.to_string(),
                ));
            }
            let clip = clip.unwrap();
            clips.insert(clip.id, clip);
        }

        clips
    } else {
        let mut statement = clip_data
            .database_connection
            .as_ref()
            .unwrap()
            .prepare("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND favorite = ? AND text REGEXP ? ORDER BY id DESC LIMIT ?")
            .unwrap();
        let res = statement.query_map(
            [
                min_id.to_string(),
                max_id.to_string(),
                favorite.to_string(),
                data,
                limit.to_string(),
            ],
            |row| {
                let favorite: i64 = row.get(3)?;
                Ok(Clip {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    favorite: favorite == 1,
                })
            },
        );

        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                -1,
                err.to_string(),
            ));
        }

        let res = res.unwrap();
        let mut clips = HashMap::new();

        for clip in res {
            if let Err(err) = clip {
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.to_string(),
                ));
            }
            let clip = clip.unwrap();
            clips.insert(clip.id, clip);
        }

        clips
    };

    Ok(res)
}

#[tauri::command]
pub fn get_max_id(clip_data: tauri::State<ClipDataMutex>) -> Result<i64, error::Error> {
    let clip_data = clip_data.clip_data.lock().unwrap();
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
pub fn search_clips(
    app: AppHandle,
    data: String,
    minid: i64,
    maxid: i64,
    favorite: i64, // 0: not favorite, 1: favorite, -1: all
    searchmethod: String,
) -> Result<HashMap<i64, Clip>, String> {
    let config = app.state::<ConfigMutex>();
    let config = config.config.lock().unwrap();
    let limit = config.search_clip_per_page;
    drop(config);

    match searchmethod.as_str() {
        "fuzzy" => {
            let res = fuzzy_search(&app, minid, maxid, limit, data, favorite);
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "fast" => {
            let res = fast_search(&app, minid, maxid, limit, data, favorite);
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "normal" => {
            let res = normal_search(&app, minid, maxid, limit, data, favorite);
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "regexp" => {
            let res = regexp_search(&app, minid, maxid, limit, data, favorite);
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        _ => Err("invalid search method".to_string()),
    }
}

// TODO add regexp search
