use std::collections::HashMap;

use sqlite::{State, Value};
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
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().unwrap();
    let mut statement = clip_data
        .database_connection
        .as_ref()
        .unwrap()
        .prepare("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text MATCH ? ORDER BY id DESC LIMIT ?")
        .unwrap();
    statement
        .bind::<&[(_, Value)]>(
            &[
                (1, min_id.into()),
                (2, max_id.into()),
                (3, data.into()),
                (4, limit.into()),
            ][..],
        )
        .unwrap();
    let mut clips = HashMap::new();
    loop {
        let state = statement.next();
        match state {
            Ok(State::Done) => {
                break;
            }
            Ok(State::Row) => {
                let id = statement.read::<i64, _>("id");
                if let Err(err) = id {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let id = id.unwrap();

                let text = statement.read::<String, _>("text");
                if let Err(err) = text {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let text = text.unwrap();

                let timestamp = statement.read::<i64, _>("timestamp");
                if let Err(err) = timestamp {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let timestamp = timestamp.unwrap();

                let favorite = statement.read::<i64, _>("favorite");
                if let Err(err) = favorite {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let favorite = favorite.unwrap() == 1;

                let clip = Clip {
                    id,
                    text,
                    timestamp,
                    favorite,
                };
                clips.insert(clip.id, clip);
            }
            Err(err) => {
                println!("{}", err.message.clone().unwrap());
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.message.unwrap(),
                ));
            }
        }
    }

    Ok(clips)
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
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().unwrap();
    let mut statement = clip_data
        .database_connection
        .as_ref()
        .unwrap()
        .prepare(
            "SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text Like ? ORDER BY id DESC LIMIT ?",
        )
        .unwrap();
    statement
        .bind::<&[(_, Value)]>(
            &[
                (1, min_id.into()),
                (2, max_id.into()),
                (3, ("%".to_string() + data.as_str() + "%").into()),
                (4, limit.into()),
            ][..],
        )
        .unwrap();
    let mut clips = HashMap::new();
    loop {
        let state = statement.next();
        match state {
            Ok(State::Done) => {
                break;
            }
            Ok(State::Row) => {
                let id = statement.read::<i64, _>("id");
                if let Err(err) = id {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let id = id.unwrap();

                let text = statement.read::<String, _>("text");
                if let Err(err) = text {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let text = text.unwrap();

                let timestamp = statement.read::<i64, _>("timestamp");
                if let Err(err) = timestamp {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let timestamp = timestamp.unwrap();

                let favorite = statement.read::<i64, _>("favorite");
                if let Err(err) = favorite {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let favorite = favorite.unwrap() == 1;

                let clip = Clip {
                    id,
                    text,
                    timestamp,
                    favorite,
                };
                clips.insert(clip.id, clip);
            }
            Err(err) => {
                return Err(error::Error::GetClipDataFromDatabaseErr(
                    -1,
                    err.message.unwrap(),
                ));
            }
        }
    }

    Ok(clips)
}

/// fuzzy search for a clip in the database
/// try iterate with each clip in the database such that the id is between min_id and max_id and maximum limit clips
/// will return a list of clips
///
/// this will try select clips match data clip and min_id <= id <= max_id and maximum limit clips
/// TODO fix fuzzy search return none problem
pub fn fuzzy_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
) -> Result<HashMap<i64, Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let mut clip_data = clip_data.clip_data.lock().unwrap();

    // get min_id_pos and max_id_pos
    let min_id_pos_res = clip_data.clips.whole_list_of_ids.binary_search(&min_id);
    let max_id_pos_res = clip_data.clips.whole_list_of_ids.binary_search(&max_id);
    let mut min_id_pos = 0;
    let mut max_id_pos = 0;
    if let Ok(pos) = max_id_pos_res {
        max_id_pos = pos;
    }
    if let Ok(pos) = min_id_pos_res {
        min_id_pos = pos;
    }
    if let Err(err) = max_id_pos_res {
        max_id_pos = err - 1;
    }
    if let Err(err) = min_id_pos_res {
        min_id_pos = err;
    }

    let mut clips = HashMap::new();
    let mut count = 0;
    let mut pos = max_id_pos + 1;
    while pos > min_id_pos && count < limit {
        pos -= 1;
        println!("pos: {}", pos);
        let id = clip_data.clips.whole_list_of_ids.get(pos);
        if id.is_none() {
            continue;
        }
        let id = id.unwrap();
        let id = *id;
        let clip = clip_data.get_clip(id)?;
        let result = best_match(&data, &clip.text);
        if result.is_none() {
            continue;
        }
        let result = result.unwrap();
        if result.score() <= 0 {
            continue;
        }
        clips.insert(clip.id, clip);
        count += 1;
    }

    Ok(clips)
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
    searchmethod: String,
) -> Result<HashMap<i64, Clip>, String> {
    let config = app.state::<ConfigMutex>();
    let config = config.config.lock().unwrap();
    let limit = config.search_clip_per_page;
    drop(config);

    match searchmethod.as_str() {
        "fuzzy" => {
            let res = fuzzy_search(&app, minid, maxid, limit, data);
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "fast" => {
            let res = fast_search(&app, minid, maxid, limit, data);
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        "normal" => {
            let res = normal_search(&app, minid, maxid, limit, data);
            if let Err(err) = res {
                return Err(err.message());
            }
            Ok(res.unwrap())
        }
        _ => Err("invalid search method".to_string()),
    }
}

// TODO add regexp search
