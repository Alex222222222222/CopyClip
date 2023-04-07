/// invoke delete_clip_from_database
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, Callback, Html, Properties};
use yew_icons::{Icon, IconId};

use crate::invoke::invoke;

use super::clip::SearchRes;

#[derive(PartialEq, Properties)]
pub struct TrashClipButtonProps {
    pub id: i64,
    pub search_res_dispatch: yewdux::prelude::Dispatch<SearchRes>,
}

#[derive(Debug, Serialize)]
struct TrashClipArgs {
    pub id: i64,
}

#[function_component(TrashClipButton)]
pub fn trash_clip_button(props: &TrashClipButtonProps) -> Html {
    let id = props.id;
    let search_res_dispatch = props.search_res_dispatch.clone();
    let trash_clip_button_on_click = Callback::from(move |_| {
        let search_res_dispatch = search_res_dispatch.clone();
        spawn_local(async move {
            let args = TrashClipArgs { id };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            invoke("delete_clip_from_database", args).await;
            search_res_dispatch.reduce_mut(|state| {
                // try find the pos of the clip
                let mut res = state.res.lock().unwrap();
                let pos = res.iter().position(|x| x.id == id);
                if let Some(pos) = pos {
                    res.remove(pos);
                    state.rebuild_num += 1;
                }
            })
        });
    });

    html! {
        <td class="border border-gray-200">
            <button
                class="font-bold w-full"
                onclick={trash_clip_button_on_click}
            >
                <Icon icon_id={IconId::BootstrapTrash} class="mx-auto mt-0.5"/>
            </button>
        </td>
    }
}
