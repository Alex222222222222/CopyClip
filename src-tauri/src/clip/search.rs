use sqlite::{State, Value};
use sublime_fuzzy::best_match;
use tauri::{AppHandle, Manager};

use crate::error;

use super::{Clip, ClipDataMutex};

/// search for a clip in the database
/// SELECT * FROM enrondata1 WHERE content MATCH 'linux';

/// fast search for a clip in the database
/// using the fts4 table search, will be faster however, for install data = "install" and a clip = "installapp"
/// then this clip will not be in the search result.
/// fts4 only searches for as a discrete token, not as a substring
///
/// this will try select clips match data wil id between min_id and max_id and maximum limit clips
/// will return a list of clips
pub fn fast_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
) -> Result<Vec<Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().unwrap();
    let mut statement = clip_data
        .database_connection
        .as_ref()
        .unwrap()
        .prepare("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text MATCH ? LIMIT ?")
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
    let mut clips = Vec::new();
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
                clips.push(clip);
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

/// normal search for a clip in the database
/// SELECT count(*) FROM enrondata2 WHERE content LIKE '%linux%'
/// will search for substring
///
/// this will try select clips match data wil id between min_id and max_id and maximum limit clips
/// will return a list of clips
pub fn normal_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
) -> Result<Vec<Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let clip_data = clip_data.clip_data.lock().unwrap();
    let mut statement = clip_data
        .database_connection
        .as_ref()
        .unwrap()
        .prepare("SELECT * FROM clips WHERE id BETWEEN ? AND ? AND text Like ? LIMIT ?")
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
    let mut clips = Vec::new();
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
                clips.push(clip);
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
pub fn fuzzy_search(
    app: &AppHandle,
    min_id: i64,
    max_id: i64,
    limit: i64,
    data: String,
) -> Result<Vec<Clip>, error::Error> {
    let clip_data = app.state::<ClipDataMutex>();
    let mut clip_data = clip_data.clip_data.lock().unwrap();
    let min_id_pos = clip_data.get_id_pos_in_whole_list_of_ids(min_id);
    let max_id_pos = clip_data.get_id_pos_in_whole_list_of_ids(max_id);
    if min_id_pos.is_none() {
        return Ok(vec![]);
    }
    if max_id_pos.is_none() {
        return Ok(vec![]);
    }
    let min_id_pos = min_id_pos.unwrap();
    let max_id_pos = max_id_pos.unwrap();

    let mut clips = Vec::new();
    let mut count = 0;
    let mut pos = max_id_pos - 1;
    while pos >= min_id_pos && count < limit {
        let id = clip_data.clips.whole_list_of_ids.get(pos as usize);
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
        clips.push(clip);
        count += 1;
        pos -= 1;
    }

    Ok(clips)
}
