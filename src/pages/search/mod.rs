use std::collections::HashMap;

use gloo_console::log;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use wasm_bindgen_futures::spawn_local;
use yew::{
    function_component, html, use_state, use_state_eq, Callback, Html, Properties, TargetCast,
    UseStateHandle,
};

use web_sys::{Event, HtmlInputElement};

use crate::{
    components::head_bar::HeadBar,
    pages::search::{
        clip::{Clip, ClipRes, SearchRes},
        search::search_clips,
        search_state::{SearchState, SearchStateHtml},
    },
};

mod clip;
mod search;
mod search_state;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// empty args
#[derive(Clone, Debug, Default, PartialEq, Properties, Serialize, Deserialize)]
struct EmptyArg {}

/// search args
#[derive(Serialize, Deserialize)]
struct SearchArgs {
    pub data: String,
    pub minid: i64,
    /// -1 means no limit
    pub maxid: i64,
    /// fuzzy, fast, normal
    pub searchmethod: String,
    // limit get from the use config by the backend
}

#[function_component(Search)]
pub fn search() -> Html {
    let search_res: UseStateHandle<SearchRes> = use_state(SearchRes::new);
    let search_method: UseStateHandle<String> = use_state_eq(|| "fuzzy".to_string());
    let search_state: UseStateHandle<SearchState> = use_state(|| SearchState::NotStarted);
    let text_data: UseStateHandle<String> = use_state_eq(|| "".to_string());
    let search_res_num: UseStateHandle<usize> = use_state_eq(|| 0);
    let order_by: UseStateHandle<String> = use_state_eq(|| "id".to_string());
    let order_order: UseStateHandle<bool> = use_state_eq(|| true); // true is desc false is asc

    let text_data_1 = text_data.clone();
    let text_box_on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        text_data_1.set(value);
    });

    let search_method_1 = search_method.clone();
    let search_res_1 = search_res.clone();
    let search_state_1 = search_state.clone();
    let search_method_on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        if value != search_method_1.to_string() {
            search_method_1.set(value);
            search_state_1.set(SearchState::NotStarted);
            search_res_1.set(SearchRes::new());
        }
    });

    let search_res_1 = search_res.clone();
    let search_method_1 = search_method.clone();
    let search_state_1 = search_state.clone();
    let text_data_1 = text_data.clone();
    let search_res_num_1 = search_res_num.clone();
    let search_button_on_click = Callback::from(move |_| {
        let search_res_clone = search_res_1.clone();
        let search_method_clone = search_method_1.clone();
        let search_state_clone = search_state_1.clone();
        let search_state_clone_clone = search_state_1.clone();
        let text_data_clone = text_data_1.clone();
        let search_res_num_clone = search_res_num_1.clone();
        spawn_local(async move {
            search_state_clone.set(SearchState::Searching);
            search_res_num_clone.set(0);
            search_res_clone.set(SearchRes::new());
            let res = search_clips(
                text_data_clone,
                search_method_clone,
                search_state_clone_clone,
                search_res_clone,
                search_res_num_clone,
            )
            .await;
            if let Err(err) = res {
                search_state_clone.set(SearchState::Error(err));
            }
        });
    });

    html! {
        <div class="flex min-h-screen flex-col bg-white">
            <HeadBar></HeadBar>
            <h1 class="text-center text-6xl m-0">{ "Search" }</h1>
            <div class="mx-5 my-2">
                <div class="flex flex-col">
                    <label htmlFor="int-input-box" class=" text-xl">
                        {"Type to search"}
                    </label>
                    <input
                        id="text-input-box"
                        type="text"
                        class="border border-gray-200 rounded-md p-2"
                        onchange={text_box_on_change}
                        // value={"test"}
                        placeholder={"Search"}
                    />
                    <br/>
                    <label htmlFor="int-input-box" class=" text-xl">
                        {"Choose search method"}
                    </label>
                    // search method drop list
                    <select
                        class="border border-gray-200 rounded-md p-2"
                        onchange={search_method_on_change}
                    >
                        <option value="fuzzy">{"Fuzzy"}</option>
                        <option value="fast">{"Fast"}</option>
                        <option value="normal">{"Normal"}</option>
                    </select>
                    <br/>
                    // search button
                    <button
                        class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                        onclick={search_button_on_click}
                    >
                        {"Search"}
                    </button>

                    // search state
                    <SearchStateHtml state={search_state.state()}></SearchStateHtml>

                    // search res
                    {search_res_table_html(search_res, order_by,order_order)}
                </div>
            </div>
        </div>
    }
}

fn search_res_table_html(
    res: UseStateHandle<SearchRes>,
    order_by: UseStateHandle<String>,
    order_order: UseStateHandle<bool>, // asc or desc
) -> Html {
    let res = res.get();
    let mut res = res.lock().unwrap();
    log!("searching len got".to_owned() + &res.len().to_string());
    let mut res: Vec<(i64, Clip)> = res.drain().collect();
    res.sort_by(|a, b| {
        if order_by.to_string() == "time" {
            a.1.timestamp.cmp(&b.1.timestamp)
        } else {
            a.1.id.cmp(&b.1.id)
        }
    });

    html! {
        <div class="flex flex-col">
            <label htmlFor="int-input-box" class=" text-xl">
                {"Search result"}
            </label>
            <table class="table-auto">
                <thead>
                    <tr>
                        // select box, a icon
                        <th class="border border-gray-200">{ "Box" }</th>
                        // the time of the clip
                        <th class="border border-gray-200">{ "Time" }</th>
                        // favorite or not, use heart icon
                        <th class="border border-gray-200">{ "Favorite" }</th>
                        // the fuzzy score of the clip
                        <th class="border border-gray-200">{ "Score" }</th>
                        // copy the clip button icon
                        <th class="border border-gray-200">{ "Copy" }</th>
                        // only part of the clip, if the user want to see the whole clip, he can click the link which will lead to the clip page
                        <th class="border border-gray-200 w-8/12">{ "Clip" }</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        res.into_iter().map(|(id, clip)| {
                            log!("searching".to_owned() + &id.to_string());
                            html! {
                                <tr>
                                    <td class="border border-gray-200">{"Tick Box"}</td>
                                    <td class="border border-gray-200">{clip.timestamp}</td>
                                    <td class="border border-gray-200">{clip.favorite}</td>
                                    <td class="border border-gray-200">{clip.score}</td>
                                    <td class="border border-gray-200">{"Copy Button"}</td>
                                    <td class="border border-gray-200">{clip.text}</td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    }
}
