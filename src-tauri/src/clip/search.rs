use sqlite::{Value, State};
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
        let state = statement.next()
        match state{
            Ok(State::Done) => {
                break;
            }
            Ok(State::Row) => {
                let id = statement.read::<i64,_>("id");
                if let Err(err) = id  {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let id = id.unwrap();

                let text = statement.read::<String,_>("text");
                if let Err(err) = text  {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let text = text.unwrap();

                let timestamp = statement.read::<i64,_>("timestamp");
                if let Err(err) = timestamp  {
                    return Err(error::Error::GetClipDataFromDatabaseErr(
                        -1,
                        err.message.unwrap(),
                    ));
                }
                let timestamp = timestamp.unwrap();

                let favorite = statement.read::<i64,_>("favorite");
                if let Err(err) = favorite  {
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
