use std::rc::Rc;

use serde::Deserialize;
use serde::Serialize;
use yew::platform::spawn_local;
use yew::{function_component, html, Callback, Html, TargetCast};

use web_sys::{Event, HtmlInputElement};

use crate::pages::search::search_method::SearchMethod;
use crate::pages::search::search_res_table::SearchResTable;
use crate::{
    components::head_bar::HeadBar,
    pages::search::{
        clip::SearchRes,
        favourite_button::FavouriteFilter,
        order::OrderOrder,
        search_clip::search_clips,
        search_state::{SearchState, SearchStateHtml},
    },
};

use self::order::OrderMethod;

mod clip;
mod copy_clip_button;
mod favourite_button;
mod fuzzy_search_text;
mod order;
mod search_clip;
mod search_method;
mod search_res_table;
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
    pub search_method: SearchMethod,
    pub search_state: SearchState,
    pub search_data: Rc<String>,
    pub order_by: OrderMethod,
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
            search_method: SearchMethod::Fuzzy,
            search_state: SearchState::NotStarted,
            search_data: Rc::new("".to_string()),
            order_by: OrderMethod::Time,
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
        if state.search_state == SearchState::Finished {
            state.search_state = SearchState::NotStarted;
        }
    });

    let text_box_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            *Rc::make_mut(&mut state.search_data) = value;
        });

    let search_method_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            state.search_method = SearchMethod::from(value);
        });

    let order_method_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            state.order_by = OrderMethod::from(value);
        });

    let order_order_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            if value == "desc" {
                state.order_order = OrderOrder::Desc;
            } else {
                state.order_order = OrderOrder::Asc;
            }
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
        });

    let search_res_dispatch_1 = search_res_dispatch.clone();
    let search_args_dispatch_1 = search_args_dispatch.clone();
    let search_args_1 = search_args.clone();
    let search_button_on_click = Callback::from(move |_| {
        let search_args_dispatch = search_args_dispatch_1.clone();
        let search_args = search_args_1.clone();
        let search_res_dispatch = search_res_dispatch_1.clone();
        search_args_dispatch.reduce_mut(|state| {
            state.search_state = SearchState::Searching;
        });
        spawn_local(async move {
            let res = search_clips(search_res_dispatch, search_args.self_copy());
            let res = res.await;
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
            let res = value.parse::<usize>();
            if let Err(err) = res {
                state.search_state =
                    SearchState::Error(format!("Total search res limit should be a int: {}", err));
                return;
            }
            state.total_search_res_limit = res.unwrap();
        });

    let user_id_limit_min_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            let res = value.parse::<i64>();
            if let Err(err) = res {
                state.search_state =
                    SearchState::Error(format!("User id limit should be a int: {}", err));
                return;
            }
            state.user_id_limit = state.user_id_limit.new_min(res.unwrap());
        });
    let user_id_limit_max_on_change =
        search_args_dispatch.reduce_mut_callback_with(|state, event: Event| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            let res = value.parse::<i64>();
            if let Err(err) = res {
                state.search_state =
                    SearchState::Error(format!("User id limit should be a int: {}", err));
                return;
            }
            state.user_id_limit = state.user_id_limit.new_max(res.unwrap());
        });

    html! {
        <div class="flex min-h-screen flex-col">
            <HeadBar></HeadBar>
            <h1 class="text-center text-6xl m-0">{ "Search" }</h1>
            <div class="mx-5 my-2">
                <div class="flex flex-col">
                    <div class="flex flex-row my-2 justify-between">
                        <label htmlFor="search-page-search-data-input-box" class="text-xl py-1">
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
                        <label htmlFor="search-page-search-method-input-box" class="text-xl">
                            {"Choose search method"}
                        </label>
                        // search method drop list
                        <select
                            class="border border-gray-200 rounded-md p-2 text-lg dark:text-black"
                            onchange={search_method_on_change}
                        >
                            <option value="fuzzy" selected={SearchMethod::Fuzzy == search_args.search_method}>{"Fuzzy"}</option>
                            <option value="fast" selected={SearchMethod::Fast == search_args.search_method}>{"Fast"}</option>
                            <option value="normal" selected={SearchMethod::Normal == search_args.search_method}>{"Normal"}</option>
                            <option value="regexp" selected={SearchMethod::Regexp == search_args.search_method}>{"Regexp"}</option>
                        </select>
                    </div>

                    <div class="flex flex-row my-2 justify-between">
                        <label htmlFor="search-page-order-method-input-box" class="text-xl">
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
                                <option value="time" selected={OrderMethod::Time == search_args.order_by}>{"Time"}</option>
                                <option value="fuzzy_score" selected={OrderMethod::FuzzyScore == search_args.order_by}>{"Score"}</option>
                                <option value="id" selected={OrderMethod::Id == search_args.order_by}>{"Id"}</option>
                                <option value="text" selected={OrderMethod::Text == search_args.order_by}>{"Text"}</option>
                                <option value="size" selected={OrderMethod::Size == search_args.order_by}>{"Length"}</option>
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
                        <label htmlFor="search-page-favourite-filter-input-box" class=" text-xl">
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
                        <label htmlFor="search-page-search-total-res-limit-input-box" class="text-xl py-1">
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
                        <label htmlFor="search-page-id-limit-min-input-box" class="text-xl py-1">
                            {"Min ID"}
                        </label>
                        <input
                            id="user-id-limit-min-input-box"
                            type="number"
                            class="border border-gray-200 rounded-md px-2 py-1 ml-5 flex-1 dark:text-black"
                            onchange={user_id_limit_min_on_change}
                            value={search_args.user_id_limit.min.to_string()}
                        />
                        <label htmlFor="search-page-id-limit-max-input-box" class="text-xl py-1 ml-5">
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
                    <SearchStateHtml state={search_args.clone()}></SearchStateHtml>

                    // search res
                    <SearchResTable
                        search_args={search_args}
                        search_res={search_res}
                        search_res_dispatch={search_res_dispatch}
                    ></SearchResTable>
                </div>
            </div>
        </div>
    }
}
