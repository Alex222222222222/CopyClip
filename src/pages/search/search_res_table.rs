use std::rc::Rc;

use yew::{function_component, html, Html, Properties};
use yew_icons::{Icon, IconId};

use crate::pages::search::{
    copy_clip_button::CopyClipButton, favourite_button::FavouriteClipButton,
    favourite_clip_filter::FavouriteClipFilter, fuzzy_search_text::SearchText,
    order::sort_search_res, pin_clip_button::PinClipButton, pin_clip_filter::PinClipFilter,
    time_display::TimeDisplay, trash_clip_button::TrashClipButton,
};

use super::{clip::SearchRes, SearchFullArgs};

/// the search res table
#[derive(Clone, PartialEq, Properties)]
pub struct SearchResTableProps {
    pub search_args: Rc<SearchFullArgs>,
    pub search_res: Rc<SearchRes>,
    pub search_res_dispatch: yewdux::prelude::Dispatch<SearchRes>,
    pub favourite_filter_dispatch: yewdux::prelude::Dispatch<SearchFullArgs>,
    pub pin_filter_dispatch: yewdux::prelude::Dispatch<SearchFullArgs>,
}

#[function_component(SearchResTable)]
pub fn search_res_table_html(props: &SearchResTableProps) -> Html {
    sort_search_res(
        props.search_res.res.clone(),
        props.search_args.order_by.clone(),
        props.search_args.order_order.clone(),
    );
    let res = props.search_res.res.lock().unwrap();

    html! {
        <div class="flex flex-col">
            <table class="table-auto">
                <thead>
                    <tr>
                        // the id of the clip
                        <th class="border border-gray-200">{ "ID" }</th>
                        // the len of the clip
                        <th class="border border-gray-200">{ "Len" }</th>
                        // the time of the clip
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::LucideTimer} class="mx-auto mt-0.5"/>
                        </th>
                        // favourite or not, use heart icon
                        <th class="border border-gray-200">
                            /* <Icon icon_id={IconId::BootstrapHeartHalf} class="mx-auto mt-0.5"/> */
                            <FavouriteClipFilter
                            favourite_filter_dispatch={props.favourite_filter_dispatch.clone()}
                            search_args={props.search_args.clone()}
                            search_res_dispatch={props.search_res_dispatch.clone()}
                            ></FavouriteClipFilter>
                        </th>
                        // the fuzzy score of the clip
                        <th class="border border-gray-200">{ "Score" }</th>
                        // pin the clip button icon
                        <th class="border border-gray-200">
                            <PinClipFilter
                            pin_filter_dispatch={props.pin_filter_dispatch.clone()}
                            search_args={props.search_args.clone()}
                            search_res_dispatch={props.search_res_dispatch.clone()}
                            ></PinClipFilter>
                        </th>
                        // copy the clip button icon
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::HeroiconsOutlineClipboardDocumentList} class="mx-auto mt-0.5"/>
                        </th>
                        // only part of the clip, if the user want to see the whole clip, he can click the link which will lead to the clip page
                        <th class="border border-gray-200">{ "Clip" }</th>
                        // delete the clip button icon
                        <th class="border border-gray-200">
                            <Icon icon_id={IconId::BootstrapTrash} class="mx-auto mt-0.5"/>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {
                        res.iter().map(|clip| {
                            html! {
                                <tr>
                                    <td class="border border-gray-200 text-center">{clip.clip.id}</td>
                                    <td class="border border-gray-200 text-center">{clip.len}</td>
                                    <TimeDisplay time={clip.clip.timestamp}></TimeDisplay>
                                    <FavouriteClipButton id={clip.clip.id} is_favourite={clip.clip.labels.contains(&"favourite".to_string())}></FavouriteClipButton>
                                    <td class="border border-gray-200 text-center">{clip.score}</td>
                                    <PinClipButton id={clip.clip.id} pinned={clip.clip.labels.contains(&"pinned".to_string())}></PinClipButton>
                                    <CopyClipButton id={clip.clip.id}></CopyClipButton>
                                    <SearchText text={clip.clip.text.clone()} data={props.search_args.search_data.clone()} search_method={props.search_args.search_method.clone()}></SearchText>
                                    <TrashClipButton id={clip.clip.id} search_res_dispatch={props.search_res_dispatch.clone()}></TrashClipButton>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    }
}
