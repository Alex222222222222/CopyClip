use serde_wasm_bindgen::to_value;
use web_sys::{Event, HtmlInputElement};
use yew::{function_component, html, platform::spawn_local, Callback, Html, TargetCast};

use crate::invoke::invoke;

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    Eq,
    yewdux::prelude::Store,
)]
#[store(storage = "local")]
struct AddClipID {
    clip_id: i64,
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    Eq,
    yewdux::prelude::Store,
)]
#[store(storage = "local")]
struct RemoveClipID {
    clip_id: i64,
}

/// the struct to be passed to the add_remove_pinned_clip function
/// action: 0 for add, 1 for remove
#[derive(serde::Serialize, serde::Deserialize)]
struct AddRemovePinnedClipsPros {
    action: i64,
    id: i64,
}

#[function_component(AddPinnedClips)]
pub fn add_pinned_clips() -> Html {
    // yewdux to save clip id to be pinned
    let (add_clip_id, add_clip_id_dispatch) = yewdux::prelude::use_store::<AddClipID>();

    let on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let value = value.parse::<i64>().unwrap();
        add_clip_id_dispatch.set(AddClipID { clip_id: value });
    });

    let add_clip_id1 = add_clip_id.clone();
    let add_button_on_click = Callback::from(move |_| {
        let value = add_clip_id1.clip_id;
        let args = to_value(&AddRemovePinnedClipsPros {
            action: 0,
            id: value,
        })
        .unwrap();
        spawn_local(async {
            invoke("add_remove_pinned_clip", args).await;
        })
    });

    html! {
        <>
            <div class="flex flex-row justify-between">
                <label htmlFor={format!("{}-input-box", "preferences.pinned_clips_add")} class="text-xl my-2">
                    {t!("preferences.pinned_clips_add")}
                </label>
                <input
                    id={format!("{}-input-box", "preferences.pinned_clips_add")}
                    type="number"
                    class="border border-gray-200 rounded-md p-2 dark:text-black ml-5 flex-1"
                    onchange={on_change}
                    value={add_clip_id.clip_id.to_string()}
                />
            </div>
            // Add button
            <button
                class="search-button bg-black my-2"
                onclick={add_button_on_click}
            >
                <span
                    class="dark:bg-white dark:text-black text-white"
                > {t!("preferences.pinned_clips_add")} </span>
            </button>
        </>
    }
}

#[function_component(RemovePinnedClips)]
pub fn remove_pinned_clips() -> Html {
    // yewdux to save clip id to be pinned
    let (remove_clip_id, remove_clip_id_dispatch) = yewdux::prelude::use_store::<RemoveClipID>();

    let on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let value = value.parse::<i64>().unwrap();
        remove_clip_id_dispatch.set(RemoveClipID { clip_id: value });
    });

    let remove_clip_id1 = remove_clip_id.clone();
    let add_button_on_click = Callback::from(move |_| {
        let value = remove_clip_id1.clip_id;
        let args = to_value(&AddRemovePinnedClipsPros {
            action: 1,
            id: value,
        })
        .unwrap();
        spawn_local(async {
            invoke("add_remove_pinned_clip", args).await;
        })
    });

    html! {
        <>
            <div class="flex flex-row justify-between">
                <label htmlFor={format!("{}-input-box", "preferences.pinned_clips_remove")} class="text-xl my-2">
                    {t!("preferences.pinned_clips_remove")}
                </label>
                <input
                    id={format!("{}-input-box", "preferences.pinned_clips_remove")}
                    type="number"
                    class="border border-gray-200 rounded-md p-2 dark:text-black ml-5 flex-1"
                    onchange={on_change}
                    value={remove_clip_id.clip_id.to_string()}
                />
            </div>
            // Add button
            <button
                class="search-button bg-black my-2"
                onclick={add_button_on_click}
            >
                <span
                    class="dark:bg-white dark:text-black text-white"
                > {t!("preferences.pinned_clips_remove")} </span>
            </button>
        </>
    }
}
