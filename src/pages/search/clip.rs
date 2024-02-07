use std::{rc::Rc, sync::Mutex};

use serde::Deserialize;
use serde::Serialize;
use yew::Properties;

/// max len of the clip to do fuzzy search
const MAX_LEN: usize = 2000;

/// clip data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Clip {
    pub id: i64,
    pub text: Rc<String>,
    pub timestamp: i64,
    pub favourite: bool,
    pub score: i64,
    pub len: u64,
    pub pinned: bool,
}

impl Clip {
    /// create a new clip from the search data and the clip data
    pub fn from_clip_res(search_data: String, clip_res: ClipRes) -> Self {
        // if the text is too long, we skip the fuzzy check.
        if clip_res.text.len() > MAX_LEN {
            return Self {
                len: clip_res.text.len() as u64,
                id: clip_res.id,
                text: Rc::new(clip_res.text),
                timestamp: clip_res.timestamp,
                favourite: clip_res.favourite,
                score: 0,
                pinned: clip_res.pinned,
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
            len: clip_res.text.len() as u64,
            text: Rc::new(clip_res.text),
            timestamp: clip_res.timestamp,
            favourite: clip_res.favourite,
            pinned: clip_res.pinned,
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
    pub pinned: bool,
}

/// search Result
#[derive(Default, Deserialize, Serialize, Clone)]
pub struct SearchRes {
    pub rebuild_num: u64,
    pub res: std::rc::Rc<Mutex<Vec<Clip>>>,
}

impl yewdux::store::Store for SearchRes {
    fn new(_: &yewdux::Context) -> Self {
        Self::default()
    }

    fn should_notify(&self, old: &Self) -> bool {
        self.rebuild_num != old.rebuild_num
    }
}

impl PartialEq for SearchRes {
    fn eq(&self, other: &Self) -> bool {
        self.rebuild_num == other.rebuild_num
    }
}
