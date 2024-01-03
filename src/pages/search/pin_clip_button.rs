use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, use_effect_with, Callback, Html, Properties};
use yew_icons::{Icon, IconId};
use yewdux::{functional::use_store, store::Store};

use crate::invoke::invoke;

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub enum PinnedFilter {
    All,
    Pinned,
    NotPinned,
}

impl Default for PinnedFilter {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct PinClipButtonProps {
    pub id: i64,
    pub pinned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
// represent the current clip
pub enum ClipID {
    /// the current clip is the clip with the given id
    Clip(i64),
    /// is the pinned clip with the given id
    PinnedClip(i64),
}

#[derive(Debug, Serialize)]
struct PinClipToTrayArgs {
    pub id: i64,
    pub pinned: bool,
}

#[derive(Debug, Serialize)]
struct GetIsPinnedProps {
    pub id: i64,
}

#[derive(PartialEq, Debug, Store, Clone, Eq, Default, Serialize, Deserialize)]
#[store(storage = "session")]
struct IsPinnedID {
    pub pinned: HashSet<i64>,
}

#[function_component(PinClipButton)]
pub fn pin_clip_button(props: &PinClipButtonProps) -> Html {
    let (pinned_statue, dispatch) = use_store::<IsPinnedID>();

    let id = props.id;

    let dispatch1: yewdux::prelude::Dispatch<IsPinnedID> = dispatch.clone();
    let pinned_statue1 = pinned_statue.clone();
    let pin_clip_button_on_click = Callback::from(move |_| {
        let dispatch1 = dispatch.clone();
        let pinned_statue = pinned_statue1.clone();
        spawn_local(async move {
            let args = PinClipToTrayArgs {
                id,
                pinned: pinned_statue.pinned.contains(&id),
            };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            let res = invoke("switch_pinned_status", args.clone()).await;
            let res = serde_wasm_bindgen::from_value::<()>(res);
            res.unwrap();
            let res = invoke("id_is_pinned", args).await;
            let res = serde_wasm_bindgen::from_value::<bool>(res);
            let res = res.unwrap();
            gloo_console::log!("res: {}", res);
            if res {
                dispatch1.reduce_mut(|state| {
                    state.pinned.insert(id);
                });
            } else {
                dispatch1.reduce_mut(|state| {
                    state.pinned.remove(&id);
                });
            }
        });
    });

    let _dispatch = dispatch1.clone();
    use_effect_with(id, move |_| {
        spawn_local(async move {
            let args = GetIsPinnedProps { id };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            let res = invoke("id_is_pinned", args).await;
            let res = serde_wasm_bindgen::from_value::<bool>(res);
            let res = res.unwrap();

            gloo_console::log!("first load res: {}", res, id);
            if res {
                dispatch1.reduce_mut(|state| {
                    state.pinned.insert(id);
                });
            } else {
                dispatch1.reduce_mut(|state| {
                    state.pinned.remove(&id);
                });
            }
        })
    });

    let pinned_1 = pinned_statue.pinned.contains(&id);
    gloo_console::log!(
        "id: {}, pinned: {}",
        id,
        pinned_1,
        pinned_statue.pinned.len()
    );

    let mut icon = IconId::BootstrapPinAngleFill;
    if !pinned_1 {
        icon = IconId::BootstrapPinAngle
    }

    html! {
        <td class="border border-gray-200">
            <button
                class="font-bold w-full"
                onclick={pin_clip_button_on_click}
            >
                <Icon icon_id={icon} class="mx-auto mt-0.5"/>
            </button>
        </td>
    }
}
