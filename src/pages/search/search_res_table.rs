use std::rc::Rc;

use yew::{function_component, html, Html, Properties};
use yew_icons::{Icon, IconId};

use crate::pages::search::{
    copy_clip_button::CopyClipButton, favourite_button::FavouriteClipButton,
    fuzzy_search_text::SearchText, order::sort_search_res, time_display::TimeDisplay,
    trash_clip_button::TrashClipButton,
};

use super::{clip::SearchRes, SearchFullArgs};

/// the search res table
#[derive(Clone, PartialEq, Properties)]
pub struct SearchResTableProps {
    pub search_args: Rc<SearchFullArgs>,
    pub search_res: Rc<SearchRes>,
    pub search_res_dispatch: yewdux::prelude::Dispatch<SearchRes>,
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
                        res.iter().map(|clip| {
                            html! {
                                <tr>
                                    <td class="border border-gray-200 text-center">{clip.id}</td>
                                    <td class="border border-gray-200 text-center">{clip.len}</td>
                                    <TimeDisplay time={clip.timestamp}></TimeDisplay>
                                    <FavouriteClipButton id={clip.id} is_favourite={clip.favourite}></FavouriteClipButton>
                                    <td class="border border-gray-200 text-center">{clip.score}</td>
                                    <CopyClipButton id={clip.id}></CopyClipButton>
                                    <TrashClipButton id={clip.id} search_res_dispatch={props.search_res_dispatch.clone()}></TrashClipButton>
                                    <SearchText text={clip.text.clone()} data={props.search_args.search_data.clone()} search_method={props.search_args.search_method.clone()}></SearchText>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    }
}
