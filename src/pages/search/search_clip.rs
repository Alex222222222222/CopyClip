use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use yew::UseStateHandle;

use crate::pages::{
    invoke,
    search::clip::{Clip, ClipRes},
};

use super::{clip::SearchRes, search_state::SearchState, EmptyArg};

/// search args
#[derive(Serialize, Deserialize)]
struct SearchArgs {
    pub data: String,
    pub minid: i64,
    /// -1 means no limit
    pub maxid: i64,
    /// fuzzy, fast, normal
    pub searchmethod: String,
    /// favorite filter
    pub favorite: i64,
}

/// search for a clip in the database
///
/// the method is decide by the input, and whenever the method is changed, the search will be reset
///
/// the search method is fuzzy, fast, normal
///
/// search state, if not finished, it will be None, if finished, it will be Some(Ok(())) or Some(Err(String))
pub async fn search_clips(
    data: String,
    search_method: String,
    search_state: UseStateHandle<SearchState>,
    search_res: UseStateHandle<SearchRes>,
    search_res_num: UseStateHandle<usize>,
    favorite_filter: i64,
    total_search_res_limit: usize,
) -> Result<(), String> {
    let args = to_value(&EmptyArg {}).unwrap();
    let max_id = invoke("get_max_id", args).await;
    let mut max_id = max_id.as_f64().unwrap() as i64 + 1;
    let mut total_len = 0;

    // try get the search_res raw data
    let search_res_clone = search_res.clone();
    let search_res_clone = search_res_clone.get();
    let search_res_clone_clone = search_res_clone.clone();

    while max_id > 0 && total_len < total_search_res_limit {
        // the min_id is 0
        // data is the value
        // search_method is the search_method
        let args = to_value(&SearchArgs {
            data: data.clone().to_string(),
            minid: -1,
            maxid: max_id,
            searchmethod: search_method.clone().to_string(),
            favorite: favorite_filter,
        })
        .unwrap();

        let res = invoke("search_clips", args).await;
        let search_res_clone = search_res_clone_clone.clone();
        let mut search_res_clone = search_res_clone.lock().unwrap();
        let res = serde_wasm_bindgen::from_value::<HashMap<String, ClipRes>>(res);
        if let Ok(res) = res {
            if res.is_empty() {
                break;
            }
            for (id, clip) in res {
                let id = str::parse::<i64>(id.as_str()).unwrap();
                max_id -= 1;
                if id < max_id {
                    max_id = id;
                }

                search_res_clone.insert(id, Clip::from_clip_res(data.to_string(), clip));
            }

            search_res_num.set(search_res_clone.len());
            total_len = search_res_clone.len();
        } else {
            let res = res.err().unwrap();
            let err = res.to_string();
            search_state.set(SearchState::Error(err.clone()));
            return Err(err);
        }
    }

    if !search_state.is_err() {
        search_res.set(SearchRes {
            res: search_res_clone_clone,
        });
        search_state.set(SearchState::Finished);
    }

    Ok(())
}
