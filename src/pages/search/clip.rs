use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::Deserialize;
use serde::Serialize;
use yew::Properties;

/// clip data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Clip {
    pub id: i64,
    pub text: String,
    pub timestamp: i64,
    pub favourite: bool,
    pub score: i64,
}

impl Clip {
    /// create a new clip from the search data and the clip data
    pub fn from_clip_res(search_data: String, clip_res: ClipRes) -> Self {
        // if the text is too long, we skip the fuzzy check.
        if clip_res.text.len() > 2000 {
            return Self {
                id: clip_res.id,
                text: clip_res.text,
                timestamp: clip_res.timestamp,
                favourite: clip_res.favourite,
                score: 0,
            };
        }

        let res = sublime_fuzzy::best_match(&search_data, &clip_res.text);
        let score = if let Some(score) = res {
            score.score() as i64
        } else {
            0
        };

        Self {
            id: clip_res.id,
            text: clip_res.text,
            timestamp: clip_res.timestamp,
            favourite: clip_res.favourite,
            score,
        }
    }
}

/// clip data get from the backend
#[derive(PartialEq, Properties, Deserialize)]
pub struct ClipRes {
    pub id: i64,
    pub text: String,
    pub timestamp: i64,
    pub favourite: bool,
}

/// search Result
#[derive(Default, Deserialize, Serialize, Clone)]
pub struct SearchRes {
    pub rebuild_num: u64,
    pub res: Arc<Mutex<HashMap<i64, Clip>>>,
}

impl yewdux::store::Store for SearchRes {
    fn new() -> Self {
        Self::default()
    }

    fn should_notify(&self, old: &Self) -> bool {
        self.rebuild_num != old.rebuild_num
    }
}
