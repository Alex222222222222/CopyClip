use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use yew::Properties;

/// clip data
#[derive(Clone, PartialEq, Properties, Serialize, Deserialize)]
pub struct Clip {
    pub id: i64,
    pub text: String,
    pub timestamp: i64,
    pub favorite: bool,
    pub score: i64,
}

impl Clip {
    /// create a new clip from the search data and the clip data
    pub fn from_clip_res(search_data: String, clip_res: ClipRes) -> Self {
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
            favorite: clip_res.favorite,
            score,
        }
    }
}

/// clip data get from the backend
#[derive(Clone, PartialEq, Properties, Serialize, Deserialize)]
pub struct ClipRes {
    pub id: i64,
    pub text: String,
    pub timestamp: i64,
    pub favorite: bool,
}

/// search result
#[derive(Clone)]
pub struct SearchRes {
    pub res: Arc<Mutex<HashMap<i64, Clip>>>,
}

impl SearchRes {
    pub fn new() -> Self {
        Self {
            res: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self) -> Arc<Mutex<HashMap<i64, Clip>>> {
        self.res.clone()
    }
}
