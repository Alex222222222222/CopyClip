use std::collections::HashMap;

use serde::Serialize;
use serde_wasm_bindgen::to_value;

/// TODO because of I cannot find a component for the date-picker, so I do not implement the date-picker
use crate::{
    invoke::invoke,
    pages::search::clip::{Clip, ClipRes},
};

use super::{clip::SearchRes, SearchFullArgs};

/// search args
#[derive(Serialize)]
struct SearchArgs {
    pub data: String,
    pub minid: i64,
    /// -1 means no limit
    pub maxid: i64,
    /// fuzzy, fast, normal
    pub searchmethod: String,
    /// favourite filter
    pub favourite: bool,
}

/// search for a clip in the database
///
/// the method is decide by the input, and whenever the method is changed, the search will be reset
///
/// the search method is fuzzy, fast, normal
///
/// search state, if not finished, it will be None, if finished, it will be Some(Ok(())) or Some(Err(String))
pub async fn search_clips(
    search_res_dispatch: yewdux::dispatch::Dispatch<SearchRes>,
    search_full_args: SearchFullArgs,
    favourite_filter: bool,
) -> Result<(), String> {
    search_res_dispatch.reduce_mut(|state| {
        state.rebuild_num += 1;
        state.res = std::rc::Rc::new(std::sync::Mutex::new(Vec::new()));
    });

    let args = to_value(&()).unwrap();
    let mut max_id = search_full_args.user_id_limit.max;
    if max_id < 0 {
        let res = invoke("get_max_id", args).await;
        max_id = res.as_f64().unwrap() as i64 + 1;
    }
    let mut total_len = 0;

    gloo_console::log!(favourite_filter);

    while max_id > search_full_args.user_id_limit.min
        && total_len < search_full_args.total_search_res_limit
    {
        // the min_id is 0
        // data is the value
        // search_method is the search_method
        let args = to_value(&SearchArgs {
            data: search_full_args.search_data.clone().to_string(),
            minid: -1,
            maxid: max_id,
            searchmethod: search_full_args.search_method.clone().to_string(),
            favourite: favourite_filter,
        })
        .unwrap();

        let res = invoke("search_clips", args).await;
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

                search_res_dispatch.reduce_mut(|state| {
                    state.res.lock().unwrap().push(Clip::from_clip_res(
                        search_full_args.search_data.to_string(),
                        clip,
                    ));
                });
                total_len += 1;
            }

            search_res_dispatch.reduce_mut(|state| {
                state.rebuild_num += 1;
            });
        } else {
            let res = res.err().unwrap();
            let err = res.to_string();
            return Err(err);
        }
    }

    Ok(())
}
