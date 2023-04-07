use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::{function_component, html, use_effect_with_deps, Callback, Html, TargetCast};

use crate::{invoke::invoke, route::DarkModeConfig};

#[derive(Serialize, Deserialize)]
struct SetDataArg {
    data: bool,
}

#[function_component(DarkModeSwitch)]
pub fn dark_mode_switch() -> Html {
    let label = "Dark Mode";
    let set_value_invoke = "set_dark_mode";
    let get_value_invoke = "get_dark_mode";

    let (dark_mode_state, dark_mode_dispatch) = yewdux::prelude::use_store::<DarkModeConfig>();

    async fn handle_on_change(value: bool, set_value_invoke: &str) {
        let args = to_value(&SetDataArg { data: value }).unwrap();
        invoke(set_value_invoke, args).await;
    }

    let dark_mode_dispatch_1 = dark_mode_dispatch.clone();
    let on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().checked();
        let res = handle_on_change(value, set_value_invoke);
        dark_mode_dispatch_1.set(DarkModeConfig { is_dark: value });
        spawn_local(res);

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let element = document.document_element().unwrap();
        let class_list = element.class_list();
        if value {
            class_list.add_1("dark").unwrap();
        } else {
            class_list.remove_1("dark").unwrap();
        }
    });

    use_effect_with_deps(
        move |_| {
            spawn_local(async move {
                let args = to_value(&()).unwrap();
                let res = invoke(get_value_invoke, args).await;
                let res = res.as_bool().unwrap();
                dark_mode_dispatch.set(DarkModeConfig { is_dark: res });
            });
        },
        (),
    );

    html! {
        <div class="flex flex-row justify-between">
            <label htmlFor="int-input-box" class="text-xl">
                {t!{"preferences.dark_mode"}}
            </label>
            <label class="switch">
                <input
                    type="checkbox"
                    id={format!("{}-input-box", label)}
                    onchange={on_change}
                    checked={dark_mode_state.is_dark}
                />
                <span class="slider dark:bg-white bg-black"></span>
            </label>
        </div>
    }
}
