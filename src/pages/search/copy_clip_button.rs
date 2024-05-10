use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, Callback, Html, Properties};
use yew_icons::{Icon, IconId};

use crate::invoke::invoke;

#[derive(Debug, PartialEq, Properties)]
pub struct CopyClipButtonProps {
    pub id: u64,
}

#[derive(Debug, Serialize)]
struct CopyClipToClipBoardArgs {
    pub id: u64,
}

#[function_component(CopyClipButton)]
pub fn copy_clip_button(props: &CopyClipButtonProps) -> Html {
    let id = props.id;
    let copy_clip_button_on_click = Callback::from(move |_| {
        spawn_local(async move {
            let args = CopyClipToClipBoardArgs { id };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            invoke("copy_clip_to_clipboard", args).await;
        });
    });

    html! {
        <td class="border border-gray-200">
            <button
                class="font-bold w-full"
                onclick={copy_clip_button_on_click}
            >
                <Icon icon_id={IconId::HeroiconsOutlineClipboardDocumentList} class="mx-auto mt-0.5"/>
            </button>
        </td>
    }
}
