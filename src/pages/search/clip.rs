use std::sync::Mutex;

use clip::Clip;
use serde::Deserialize;
use serde::Serialize;

/// max len of the clip to do fuzzy search
const MAX_LEN: usize = 2000;

/// clip data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipWithSearchInfo {
    pub clip: Clip,
    pub score: i64,
    pub len: u64,
}

impl ClipWithSearchInfo {
    /// create a new clip from the search data and the clip data
    pub fn from_clip(search_data: String, clip_res: Clip) -> Self {
        // if the text is too long, we skip the fuzzy check.
        if clip_res.text.len() > MAX_LEN {
            return Self {
                len: clip_res.text.len() as u64,
                clip: clip_res,
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
            len: clip_res.text.len() as u64,
            clip: clip_res,
            score,
        }
    }
}

/// search Result
#[derive(Default, Deserialize, Serialize, Clone)]
pub struct SearchRes {
    pub rebuild_num: u64,
    pub res: std::rc::Rc<Mutex<Vec<ClipWithSearchInfo>>>,
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
