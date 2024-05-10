use std::rc::Rc;
use yew::{function_component, html, platform::spawn_local, Callback, Html, Properties};
use yew_icons::{Icon, IconId};
use yewdux::dispatch::Dispatch;

use crate::pages::search::{search_clip::search_clips, search_state::SearchState};

use super::{clip::SearchRes, SearchFullArgs};

#[derive(PartialEq, Properties, Clone)]
pub struct PinClipFilterProps {
    pub pin_filter_dispatch: Dispatch<SearchFullArgs>,
    pub search_args: Rc<SearchFullArgs>,
    pub search_res_dispatch: Dispatch<SearchRes>,
}

#[function_component(PinClipFilter)]
pub fn pin_clip_filter(props: &PinClipFilterProps) -> Html {
    let props1 = (*props).clone();
    let pin_filter_on_click = Callback::from(move |_| {
        /*
        props1.pin_filter_dispatch.reduce_mut(|state| {
            state.pin_filter = !state.pin_filter;
        });1
        */

        let search_args_dispatch = props1.pin_filter_dispatch.clone();
        let search_args = props1.search_args.clone();
        let search_res_dispatch = props1.search_res_dispatch.clone();
        search_args_dispatch.reduce_mut(|state| {
            state.pin_filter = !state.pin_filter;
            state.search_state = SearchState::Searching;
        });
        tauri_plugin_logging::debug(format!(
            "clicked pin filter, set default to {}",
            search_args.pin_filter
        ));
        spawn_local(async move {
            let res = search_clips(
                search_res_dispatch,
                search_args.self_copy(),
                search_args.favourite_filter,
                !search_args.pin_filter,
            );
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

    html! {

            <button
                class="mx-auto mt-2.5"
                onclick={pin_filter_on_click}
            >
                <Icon icon_id={IconId::BootstrapPinAngleFill} class=""/>
            </button>
    }
}
