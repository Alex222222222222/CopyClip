use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, Callback, Html, TargetCast};

use web_sys::{Event, HtmlInputElement};
use yew_icons::{Icon, IconId};

use crate::{
    components::head_bar::HeadBar,
    pages::search::{
        clip::{Clip, SearchRes},
        copy_clip_button::CopyClipButton,
        favourite_button::{FavouriteClipButton, FavouriteFilter},
        fuzzy_search_text::SearchText,
        order::{sort_search_res, OrderOrder},
        search_clip::search_clips,
        search_state::{SearchState, SearchStateHtml},
        time_display::TimeDisplay,
        trash_clip_button::TrashClipButton,
    },
};

mod clip;
mod copy_clip_button;
mod favourite_button;
mod fuzzy_search_text;
mod order;
mod search_clip;
mod search_state;
mod time_display;
mod trash_clip_button;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserIdLimit {
    pub min: i64, // 0 is always the min for all
    pub max: i64, // -1 means no limit
}

impl Default for UserIdLimit {
    fn default() -> Self {
        Self { min: 0, max: -1 }
    }
}

impl UserIdLimit {
    pub fn new_min(&self, min: i64) -> UserIdLimit {
        let mut new = self.clone();
        new.min = min;
        new
    }

    pub fn new_max(&self, max: i64) -> UserIdLimit {
        let mut new = self.clone();
        new.max = max;
        new
    }
}

#[derive(yewdux::prelude::Store, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[store(storage = "local")]
pub struct SearchFullArgs {
    pub search_method: String,
    pub search_state: SearchState,
    pub search_data: String,
    pub order_by: String,
    pub order_order: OrderOrder,
    pub favourite_filter: FavouriteFilter,
    pub total_search_res_limit: usize,
    pub user_id_limit: UserIdLimit,
}

impl SearchFullArgs {
    pub fn self_copy(&self) -> Self {
        self.clone()
    }
}

impl Default for SearchFullArgs {
    fn default() -> Self {
        Self {
            search_method: "fuzzy".to_string(),
            search_state: SearchState::NotStarted,
            search_data: "".to_string(),
            order_by: "time".to_string(),
            order_order: OrderOrder::Desc,
            favourite_filter: FavouriteFilter::default(),
            total_search_res_limit: 100,
            user_id_limit: UserIdLimit::default(),
        }
    }
}

#[function_component(Search)]
pub fn search() -> Html {
    let (search_res, search_res_dispatch) = yewdux::prelude::use_store::<SearchRes>();

    let (search_args, search_args_dispatch) = yewdux::prelude::use_store::<SearchFullArgs>();
    search_args_dispatch.reduce_mut(|state| {
        state.search_state = SearchState::NotStarted;
    });

    let text_box_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();

            state.search_data = value;

            // TODO if the text box is different from previous state, then try to clear the search
            // res data
        });

    let search_method_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            state.search_method = value;

            // TODO if the search method is different from previous state, then try to clear the
            // search res data
        });

    let order_method_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            state.order_by = value;

            // TODO if the order method is different from previous state, then try to rebuild the
            // table
        });

    let order_order_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            if value == "desc" {
                state.order_order = OrderOrder::Desc;
            } else {
                state.order_order = OrderOrder::Asc;
            }

            // TODO if the order order change then try to rebuild the table
        });

    let favourite_filter_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            if value == "all" {
                state.favourite_filter = FavouriteFilter::All;
            } else if value == "favourite" {
                state.favourite_filter = FavouriteFilter::Favourite;
            } else {
                state.favourite_filter = FavouriteFilter::NotFavourite;
            }

            // TODO if the favourite filter change then try to clear rhe search res data
        });

    let search_res_dispatch_1 = search_res_dispatch;
    let search_args_dispatch_1 = search_args_dispatch.clone();
    let search_args_1 = search_args.clone();
    let search_button_on_click = Callback::from(move |_| {
        let search_args_dispatch = search_args_dispatch_1.clone();
        let search_args = search_args_1.clone();
        let search_res_dispatch = search_res_dispatch_1.clone();
        spawn_local(async move {
            search_args_dispatch.reduce_mut(|state| {
                state.search_state = SearchState::Searching;
            });
            let res = search_clips(search_res_dispatch, search_args.self_copy()).await;
            if let Err(err) = res {
                search_args_dispatch.reduce_mut(|state| {
                    state.search_state = SearchState::Error(err);
                });
            } else {
                search_args_dispatch.reduce_mut(|state| {
                    state.search_state = SearchState::Finished;
                });
            }
        });
    });

    let total_search_res_limit_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            // TODO raise an Error if parse failed
            state.total_search_res_limit = value.parse().unwrap();
        });

    let user_id_limit_min_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            // TODO raise an Error if parse failed
            state.user_id_limit = state.user_id_limit.new_min(value.parse().unwrap());
        });
    let user_id_limit_max_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            // TODO raise an Error if parse failed
            state.user_id_limit = state.user_id_limit.new_max(value.parse().unwrap());
        });

    html! {
        <div class="flex min-h-screen flex-col">
            <HeadBar></HeadBar>
            <h1 class="text-center text-6xl m-0">{ "Search" }</h1>
            <div class="mx-5 my-2">
                <div class="flex flex-col">
                    <div class="flex flex-row my-2 justify-between">
                        // TODO change htmlFor
                        <label htmlFor="int-input-box" class="text-xl py-1">
                            {"Type to search"}
                        </label>
                        <input
                            id="search-data-text-input-box"
                            type="text"
                            class="border border-gray-200 rounded-md px-2 py-1 ml-5 flex-1 dark:text-black"
                            onchange={text_box_on_change}
                            placeholder={"Search"}
                            value={search_args.search_data.to_string()}
                        />
                    </div>

                    <div class="flex flex-row my-2 justify-between">
                        // TODO change htmlFor
                        <label htmlFor="int-input-box" class="text-xl">
                            {"Choose search method"}
                        </label>
                        // search method drop list
                        <select
                            class="border border-gray-200 rounded-md p-2 text-lg dark:text-black"
                            onchange={search_method_on_change}
                        >
                            <option value="fuzzy" selected={"fuzzy" == search_args.search_method.as_str()}>{"Fuzzy"}</option>
                            <option value="fast" selected={"fast" == search_args.search_method.as_str()}>{"Fast"}</option>
                            <option value="normal" selected={"normal" == search_args.search_method.as_str()}>{"Normal"}</option>
                            <option value="regexp" selected={"regexp" == search_args.search_method.as_str()}>{"Regexp"}</option>
                        </select>
                    </div>

                    <div class="flex flex-row my-2 justify-between">
                        // TODO change htmlFor
                        <label htmlFor="int-input-box" class="text-xl">
                            {"Choose order method"}
                        </label>
                        // order method drop list
                        <div
                            class="flex flex-row"
                        >
                            <select
                                class="border border-gray-200 rounded-md p-2 mr-2 text-lg dark:text-black"
                                onchange={order_method_on_change}
                            >
                                <option value="time" selected={"time" == search_args.order_by.as_str()}>{"Time"}</option>
                                <option value="score" selected={"score" == search_args.order_by.as_str()}>{"Score"}</option>
                                <option value="id" selected={"id" == search_args.order_by.as_str()}>{"Id"}</option>
                                <option value="text" selected={"text" == search_args.order_by.as_str()}>{"Text"}</option>
                            </select>
                            // order order drop list
                            <select
                                class="border border-gray-200 rounded-md p-2 ml-2 text-lg dark:text-black"
                                onchange={order_order_on_change}
                            >
                                <option value="desc" selected={OrderOrder::Desc == search_args.order_order}>{"Desc"}</option>
                                <option value="asc" selected={OrderOrder::Asc == search_args.order_order}>{"Asc"}</option>
                            </select>
                        </div>
                    </div>

                    <div class="flex flex-row my-2 justify-between">
                        // favourite filter
                        // TODO change htmlFor
                        <label htmlFor="int-input-box" class=" text-xl">
                            {"Favourite filter"}
                        </label>
                        <select
                            class="border border-gray-200 rounded-md p-2 text-lg dark:text-black"
                            onchange={favourite_filter_on_change}
                        >
                            <option value="all" selected={FavouriteFilter::All == search_args.favourite_filter}>{"All"}</option>
                            <option value="favourite" selected={FavouriteFilter::Favourite == search_args.favourite_filter}>{"Favourite"}</option>
                            <option value="not_favourite" selected={FavouriteFilter::NotFavourite == search_args.favourite_filter}>{"NotFavourite"}</option>
                        </select>
                    </div>

                    // total search res num limit
                    <div class="flex flex-row my-2 justify-between">
                        // TODO change htmlFor
                        <label htmlFor="int-input-box" class="text-xl py-1">
                            {"Total search res num limit"}
                        </label>
                        <input
                            id="totally-search-res-limit-input-box"
                            type="number"
                            class="border border-gray-200 rounded-md px-2 py-1 flex-1 ml-5 dark:text-black"
                            onchange={total_search_res_limit_on_change}
                            value={search_args.total_search_res_limit.to_string()}
                        />
                    </div>

                    // id min and id max
                    // total search res num limit
                    <div class="flex flex-row my-2 justify-between">
                        // TODO change htmlFor
                        <label htmlFor="int-input-box" class="text-xl py-1">
                            {"Min ID"}
                        </label>
                        <input
                            id="user-id-limit-min-input-box"
                            type="number"
                            class="border border-gray-200 rounded-md px-2 py-1 ml-5 flex-1 dark:text-black"
                            onchange={user_id_limit_min_on_change}
                            value={search_args.user_id_limit.min.to_string()}
                        />
                        // TODO change htmlFor
                        <label htmlFor="int-input-box" class="text-xl py-1 ml-5">
                            {"Max ID"}
                        </label>
                        <input
                            id="user-id-limit-max-input-box"
                            type="number"
                            class="border border-gray-200 rounded-md px-2 py-1 ml-5 flex-1 dark:text-black"
                            onchange={user_id_limit_max_on_change}
                            value={search_args.user_id_limit.max.to_string()}
                        />
                    </div>

                    // search button
                    <button
                        class="search-button bg-black my-2"
                        onclick={search_button_on_click}
                    >
                        <span
                            class="dark:bg-white dark:text-black text-white"
                        > {"Press To Search"} </span>
                    </button>

                    // search state
                    <SearchStateHtml state={search_args.search_state.state()}></SearchStateHtml>

                    // search res
                    {
                        search_res_table_html(
                            search_args.search_data.to_string(),
                            search_args.order_by.clone(),
                            search_args.order_order.clone(),
                            search_args.search_method.to_string(),
                            search_res.res.clone(),
                        )
                    }
                </div>
            </div>
        </div>
    }
}

fn search_res_table_html(
    data: String,
    order_by: String,
    order_order: OrderOrder, // asc or desc
    search_method: String,
    res: std::sync::Arc<std::sync::Mutex<HashMap<i64, Clip>>>,
) -> Html {
    let res = res.lock().unwrap();
    let mut res = res.clone();

    let res: Vec<(i64, Clip)> = res.drain().collect();
    let res = sort_search_res(res, order_by, order_order.to_bool());

    html! {
        <div class="flex flex-col">
            <table class="table-auto">
                <thead>
                    <tr>
                        // the id of the clip
                        <th class="border border-gray-200">{ "ID" }</th>
                        // the time of the clip
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::LucideTimer} class="mx-auto mt-0.5"/>
                        </th>
                        // favourite or not, use heart icon
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::BootstrapHeartHalf} class="mx-auto mt-0.5"/>
                        </th>
                        // the fuzzy score of the clip
                        <th class="border border-gray-200">{ "Score" }</th>
                        // copy the clip button icon
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::HeroiconsOutlineClipboardDocumentList} class="mx-auto mt-0.5"/>
                        </th>
                        // delete the clip button icon
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::BootstrapTrash} class="mx-auto mt-0.5"/>
                        </th>
                        // only part of the clip, if the user want to see the whole clip, he can click the link which will lead to the clip page
                        <th class="border border-gray-200">{ "Clip" }</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        res.into_iter().map(|(id, clip)| {
                            let search_method_1 = search_method.clone();
                            html! {
                                <tr>
                                    <td class="border border-gray-200 text-center">{clip.id}</td>
                                    <TimeDisplay time={clip.timestamp}></TimeDisplay>
                                    <FavouriteClipButton id={id} is_favourite={clip.favourite}></FavouriteClipButton>
                                    <td class="border border-gray-200 text-center">{clip.score}</td>
                                    <CopyClipButton id = {id}></CopyClipButton>
                                    <TrashClipButton id = {id}></TrashClipButton>
                                    <SearchText text={clip.text} data={data.clone()} search_method={search_method_1}></SearchText>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    }
}
