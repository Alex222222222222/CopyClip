use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::{
    function_component, html, use_state, use_state_eq, Callback, Html, Properties, TargetCast,
    UseStateHandle,
};

use web_sys::{Event, HtmlInputElement};
use yew_icons::{Icon, IconId};

use crate::{
    components::head_bar::HeadBar,
    pages::search::{
        clip::{Clip, SearchRes},
        copy_clip_button::CopyClipButton,
        favorite_button::{FavoriteClipButton, FavoriteFilter},
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
mod favorite_button;
mod fuzzy_search_text;
mod order;
mod search_clip;
mod search_state;
mod time_display;
mod trash_clip_button;

/// empty args
#[derive(Clone, Debug, Default, PartialEq, Properties, Serialize, Deserialize)]
struct EmptyArg {}

#[function_component(Search)]
pub fn search() -> Html {
    let search_res: UseStateHandle<SearchRes> = use_state(SearchRes::new);
    let search_method: UseStateHandle<String> = use_state_eq(|| "fuzzy".to_string());
    let search_state: UseStateHandle<SearchState> = use_state(|| SearchState::NotStarted);
    let text_data: UseStateHandle<String> = use_state_eq(|| "".to_string());
    let search_res_num: UseStateHandle<usize> = use_state_eq(|| 0);
    let order_by: UseStateHandle<String> = use_state_eq(|| "time".to_string());
    let order_order: UseStateHandle<OrderOrder> = use_state_eq(|| OrderOrder::Desc); // true is desc false is asc
    let favorite_filter: UseStateHandle<FavoriteFilter> = use_state_eq(FavoriteFilter::default);
    let total_search_res_limit: UseStateHandle<usize> = use_state_eq(|| 100);

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

    let order_by_1 = order_by.clone();
    let order_method_on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        order_by_1.set(value);
    });

    let order_order_1 = order_order.clone();
    let order_order_on_change = Callback::from(move |event: Event| {
        let value = event
            .target_unchecked_into::<HtmlInputElement>()
            .value()
            .to_lowercase();
        if value == "desc" {
            order_order_1.set(OrderOrder::Desc);
        } else {
            order_order_1.set(OrderOrder::Asc);
        }
    });

    let favorite_filter_1 = favorite_filter.clone();
    let favorite_filter_on_change = Callback::from(move |event: Event| {
        let value = event
            .target_unchecked_into::<HtmlInputElement>()
            .value()
            .to_lowercase();
        if value == "all" {
            favorite_filter_1.set(FavoriteFilter::All);
        } else if value == "favorite" {
            favorite_filter_1.set(FavoriteFilter::Favorite);
        } else {
            favorite_filter_1.set(FavoriteFilter::NotFavorite);
        }
    });

    let search_res_1 = search_res.clone();
    let search_method_1 = search_method.clone();
    let search_state_1 = search_state.clone();
    let text_data_1 = text_data.clone();
    let search_res_num_1 = search_res_num;
    let total_search_res_limit_1 = total_search_res_limit.clone();
    let search_button_on_click = Callback::from(move |_| {
        let search_res_clone = search_res_1.clone();
        let search_method_clone = search_method_1.clone();
        let search_state_clone = search_state_1.clone();
        let search_state_clone_clone = search_state_1.clone();
        let text_data_clone = text_data_1.clone();
        let search_res_num_clone = search_res_num_1.clone();
        let favorite_filter_1 = favorite_filter.clone();
        let total_search_res_limit = total_search_res_limit_1.clone();
        spawn_local(async move {
            search_state_clone.set(SearchState::Searching);
            search_res_num_clone.set(0);
            search_res_clone.set(SearchRes::new());
            let res = search_clips(
                text_data_clone.to_string(),
                search_method_clone.to_string(),
                search_state_clone_clone,
                search_res_clone,
                search_res_num_clone,
                favorite_filter_1.to_int(),
                total_search_res_limit.to_string().parse::<usize>().unwrap(),
            )
            .await;
            if let Err(err) = res {
                search_state_clone.set(SearchState::Error(err));
            }
        });
    });

    let total_search_res_limit_1 = total_search_res_limit.clone();
    let total_search_res_limit_on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        total_search_res_limit_1.set(value.parse().unwrap());
    });

    html! {
        <div class="flex min-h-screen flex-col bg-white">
            <HeadBar></HeadBar>
            <h1 class="text-center text-6xl m-0">{ "Search" }</h1>
            <div class="mx-5 my-2">
                <div class="flex flex-col">
                    <div class="flex flex-row my-2 w-11/12">
                        <label htmlFor="int-input-box" class="text-xl py-1">
                            {"Type to search"}
                        </label>
                        <input
                            id="text-input-box"
                            type="text"
                            class="border border-gray-200 rounded-md px-2 py-1 mx-2 w-8/12"
                            onchange={text_box_on_change}
                            placeholder={"Search"}
                        />
                    </div>

                    <div class="flex flex-row my-2">
                        <label htmlFor="int-input-box" class="text-xl">
                            {"Choose search method"}
                        </label>
                        // search method drop list
                        <select
                            class="border border-gray-200 rounded-md p-2 mx-2 text-lg"
                            onchange={search_method_on_change}
                        >
                            <option value="fuzzy">{"Fuzzy"}</option>
                            <option value="fast">{"Fast"}</option>
                            <option value="normal">{"Normal"}</option>
                            <option value="regexp">{"Regexp"}</option>
                        </select>
                    </div>

                    <div class="flex flex-row my-2">
                        <label htmlFor="int-input-box" class="text-xl">
                            {"Choose order method"}
                        </label>
                        // order method drop list
                        <select
                            class="border border-gray-200 rounded-md p-2 mx-2 text-lg"
                            onchange={order_method_on_change}
                        >
                            <option value="time">{"Time"}</option>
                            <option value="score">{"Score"}</option>
                            <option value="id">{"Id"}</option>
                            <option value="text">{"Text"}</option>
                        </select>
                        // order order drop list
                        <select
                            class="border border-gray-200 rounded-md p-2 mx-2 text-lg"
                            onchange={order_order_on_change}
                        >
                            <option value="desc">{"Desc"}</option>
                            <option value="asc">{"Asc"}</option>
                        </select>
                    </div>

                    <div class="flex flex-row my-2">
                        // favorite filter
                        <label htmlFor="int-input-box" class=" text-xl">
                            {"Favorite filter"}
                        </label>
                        <select
                            class="border border-gray-200 rounded-md p-2 mx-2 text-lg"
                            onchange={favorite_filter_on_change}
                        >
                            <option value="all">{"All"}</option>
                            <option value="favorite">{"Favorite"}</option>
                            <option value="not_favorite">{"NotFavorite"}</option>
                        </select>
                        <br/>
                    </div>

                    // total search res num limit
                    <div class="flex flex-row my-2 w-11/12">
                        <label htmlFor="int-input-box" class="text-xl py-1">
                            {"Total search res num limit"}
                        </label>
                        <input
                            id="text-input-box"
                            type="number"
                            class="border border-gray-200 rounded-md px-2 py-1 mx-2 w-6/12"
                            onchange={total_search_res_limit_on_change}
                            value={total_search_res_limit.to_string()}
                        />
                    </div>

                    // search button
                    <button
                        class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded my-2"
                        onclick={search_button_on_click}
                    >
                        {"Search"}
                    </button>

                    // search state
                    <SearchStateHtml state={search_state.state()}></SearchStateHtml>

                    // search res
                    {
                        search_res_table_html(
                            text_data.to_string(),
                            search_res,
                            order_by,
                            order_order,
                            search_method.to_string(),
                        )
                    }
                </div>
            </div>
        </div>
    }
}

fn search_res_table_html(
    data: String,
    res: UseStateHandle<SearchRes>,
    order_by: UseStateHandle<String>,
    order_order: UseStateHandle<OrderOrder>, // asc or desc
    search_method: String,
) -> Html {
    let res = res.get();
    let mut res = res.lock().unwrap();

    let res: Vec<(i64, Clip)> = res.drain().collect();
    let res = sort_search_res(res, order_by.to_string(), order_order.to_bool());

    html! {
        <div class="flex flex-col">
            <table class="table-auto">
                <thead>
                    <tr>
                        // the id of the clip
                        <th class="border border-gray-200">{ "ID" }</th>
                        // the time of the clip
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::LucideTimer}/>
                        </th>
                        // favorite or not, use heart icon
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::BootstrapHeartHalf}/>
                        </th>
                        // the fuzzy score of the clip
                        <th class="border border-gray-200">{ "Score" }</th>
                        // copy the clip button icon
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::HeroiconsOutlineClipboardDocumentList}/>
                        </th>
                        // delete the clip button icon
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::BootstrapTrash}/>
                        </th>
                        // only part of the clip, if the user want to see the whole clip, he can click the link which will lead to the clip page
                        <th class="border border-gray-200 w-8/12">{ "Clip" }</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        res.into_iter().map(|(id, clip)| {
                            let search_method_1 = search_method.clone();
                            html! {
                                <tr>
                                    <td class="border border-gray-200">{clip.id}</td>
                                    <TimeDisplay time={clip.timestamp}></TimeDisplay>
                                    <FavoriteClipButton id={id} is_favorite={clip.favorite}></FavoriteClipButton>
                                    <td class="border border-gray-200">{clip.score}</td>
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
