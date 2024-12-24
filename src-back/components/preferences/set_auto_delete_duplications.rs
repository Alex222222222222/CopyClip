use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::{function_component, html, use_effect_with, Callback, Html, TargetCast};
use yewdux::store::Store;

use crate::invoke::invoke;

#[derive(Serialize, Deserialize)]
struct SetDataArg {
    data: bool,
}

#[derive(Clone, Debug, Store, PartialEq, Deserialize, Serialize, Default)]
#[store(storage = "local")]
struct AutoDeleteDuplicateClipConfig {
    pub auto_delete_enabled: bool,
}

#[function_component(AutoDeleteDuplications)]
pub fn auto_delete_duplications() -> Html {
    let label = "Enable Auto Delete Duplications";
    let set_value_invoke = "set_auto_delete_duplicate_clip";
    let get_value_invoke = "get_auto_delete_duplicate_clip";

    let (auto_delete_duplicate_clip_state, auto_delete_duplicate_clip_dispatch) =
        yewdux::prelude::use_store::<AutoDeleteDuplicateClipConfig>();

    async fn handle_on_change(value: bool, set_value_invoke: &str) {
        let args = to_value(&SetDataArg { data: value }).unwrap();
        invoke(set_value_invoke, args).await;
    }

    let auto_delete_duplicate_clip_dispatch_1 = auto_delete_duplicate_clip_dispatch.clone();
    let on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().checked();
        let res = handle_on_change(value, set_value_invoke);
        auto_delete_duplicate_clip_dispatch_1.set(AutoDeleteDuplicateClipConfig {
            auto_delete_enabled: value,
        });
        spawn_local(res);
    });

    use_effect_with((), move |_| {
        spawn_local(async move {
            let args = to_value(&()).unwrap();
            let res = invoke(get_value_invoke, args).await;
            let res = res.as_bool().unwrap();
            auto_delete_duplicate_clip_dispatch.set(AutoDeleteDuplicateClipConfig {
                auto_delete_enabled: res,
            });
        });
    });

    html! {
        <div class="flex flex-row justify-between">
            <label htmlFor="int-input-box" class="text-xl">
                {t!{"preferences.enable_auto_delete_duplications"}}
            </label>
            <label class="switch">
                <input
                    type="checkbox"
                    id={format!("{}-input-box", label)}
                    checked={auto_delete_duplicate_clip_state.auto_delete_enabled}
                    onchange={on_change}
                />
                <span class={
                    if auto_delete_duplicate_clip_state.auto_delete_enabled {
                        "slider dark:bg-gray-500 bg-gray-400"
                    } else {
                        "slider dark:bg-white bg-black"
                    }
                }></span>
            </label>
        </div>
    }
}
