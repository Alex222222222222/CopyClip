use std::collections::HashMap;

use gloo_console::log;
use serde_wasm_bindgen::to_value;
use yew::UseStateHandle;

use crate::pages::{search::{SearchArgs, clip::{ClipRes, Clip}}, invoke};

use super::{search_state::SearchState, clip::SearchRes, EmptyArg};

/// search for a clip in the database
///
/// the method is decide by the input, and whenever the method is changed, the search will be reset
///
/// the search method is fuzzy, fast, normal
///
/// search state, if not finished, it will be None, if finished, it will be Some(Ok(())) or Some(Err(String))
pub async fn search_clips(
    data: UseStateHandle<String>,
    search_method: UseStateHandle<String>,
    search_state: UseStateHandle<SearchState>,
    search_res: UseStateHandle<SearchRes>,
    search_res_num: UseStateHandle<usize>,
) -> Result<(), String> {
    let search_method_now = search_method.clone().to_string();
    let data_now = data.clone().to_string();
    let args = to_value(&EmptyArg {}).unwrap();
    let max_id = invoke("get_max_id", args).await;
    let mut max_id = max_id.as_f64().unwrap() as i64 + 1;

    // try get the search_res raw data
    let search_res_clone = search_res.clone();
    let search_res_clone = search_res_clone.get();
    let search_res_clone_clone = search_res_clone.clone();

    while max_id > 0
        && search_method_now == search_method.to_string()
        && data_now == data.to_string()
    {
        // the min_id is 0
        // data is the value
        // search_method is the search_method
        let args = to_value(&SearchArgs {
            data: data.clone().to_string(),
            minid: -1,
            maxid: max_id,
            searchmethod: search_method.clone().to_string(),
        })
        .unwrap();

        let res = invoke("search_clips", args).await;
        let search_res_clone = search_res_clone_clone.clone();
        let mut search_res_clone = search_res_clone.lock().unwrap();
        log!("res", res.clone());
        let res = serde_wasm_bindgen::from_value::<HashMap<String, ClipRes>>(res);
        if let Ok(res) = res {
            log!("current length ".to_owned() + &search_res_clone.len().to_string());
            if res.is_empty() {
                break;
            }
            for (id, clip) in res {
                log!("searching got id ".to_owned() + &id.to_string());
                let id = str::parse::<i64>(id.as_str()).unwrap();
                max_id -= 1;
                if id < max_id {
                    max_id = id;
                }

                search_res_clone.insert(id, Clip::from_clip_res(data.to_string(), clip));
            }
            log!("current length ".to_owned() + &search_res_clone.len().to_string());

            search_res_num.set(search_res_clone.len());
        } else {
            let res = res.err().unwrap();
            let err = res.to_string();
            log!("res", err.clone());
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
