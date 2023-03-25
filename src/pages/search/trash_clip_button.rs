/// invoke delete_clip_from_database
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, Callback, Html, Properties};
use yew_icons::{Icon, IconId};

use crate::pages::invoke;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct TrashClipButtonProps {
    pub id: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct TrashClipArgs {
    pub id: i64,
}

#[function_component(TrashClipButton)]
pub fn trash_clip_button(props: &TrashClipButtonProps) -> Html {
    let id = props.id;
    let trash_clip_button_on_click = Callback::from(move |_| {
        spawn_local(async move {
            let args = TrashClipArgs { id };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            invoke("delete_clip_from_database", args).await;
        });
    });

    html! {
        <td class="border border-gray-200">
            <button
                class="font-bold"
                onclick={trash_clip_button_on_click}
            >
                <Icon icon_id={IconId::BootstrapTrash}/>
            </button>
        </td>
    }
}
