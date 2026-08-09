/// the button used to clear the clip history, except pinned clips
///
/// Native `window.confirm` dialogs are not implemented by wry's WebKitGTK
/// backend on Linux, so a native confirm would silently do nothing.
/// Instead the button requires two clicks: the first arms it, the second
/// (while armed) performs the deletion.
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, use_state, Callback, Html};

use crate::invoke::invoke;

#[function_component(ClearHistoryButton)]
pub fn clear_history_button() -> Html {
    let armed = use_state(|| false);

    let clear_history_button_on_click = {
        let armed = armed.clone();
        Callback::from(move |_| {
            if !*armed {
                armed.set(true);
                return;
            }

            armed.set(false);
            spawn_local(async move {
                let args = to_value(&()).unwrap();
                invoke("clear_clip_history", args).await;
            });
        })
    };

    let label = if *armed {
        t!("preferences.clear_history_confirm_button")
    } else {
        t!("preferences.clear_history_button")
    };

    html! (
        <button
            class="search-button bg-black my-2"
            onclick={clear_history_button_on_click}
        >
            <span
                class="dark:bg-white dark:text-black text-white"
            > {label} </span>
        </button>
    )
}
