use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::{function_component, html, use_effect_with, Callback, Html, Properties, TargetCast};

use crate::invoke::invoke;

#[derive(Serialize, Deserialize)]
struct SetPerPageDataArg {
    data: i64,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, Eq, yewdux::prelude::Store)]
#[store(storage = "local")]
struct ConfigState {
    config: HashMap<String, i64>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct IntConfigTemplateProps {
    pub label: String,
    pub default_value: i64,
    pub set_value_invoke: String,
    pub get_value_invoke: String,
}

#[function_component(IntConfigTemplate)]
pub fn int_config_template(props: &IntConfigTemplateProps) -> Html {
    let (config, config_dispatch) = yewdux::prelude::use_store::<ConfigState>();

    async fn handle_on_change(value: i64, set_value_invoke: String) {
        let args = to_value(&SetPerPageDataArg { data: value }).unwrap();
        invoke(&set_value_invoke, args).await;
    }

    let label = props.label.clone();

    let set_value_invoke = props.set_value_invoke.clone();
    let config_1 = config.clone();
    let config_dispatch_1 = config_dispatch.clone();
    let label_1 = label.clone();
    let on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let value = value.parse::<i64>().unwrap();
        let res = handle_on_change(value, set_value_invoke.clone());
        let mut config = config_1.config.clone();
        config.insert(label_1.clone(), value);
        config_dispatch_1.set(ConfigState { config });
        spawn_local(res);
    });

    let default_value = props.default_value;
    let label_1 = label.clone();
    if !config.config.contains_key(&label_1) {
        config_dispatch.reduce_mut(|state| {
            state.config.insert(label_1, default_value);
        })
    }

    let label_1 = label.clone();
    let get_value_invoke = props.get_value_invoke.clone();
    let config_1 = config.clone();
    use_effect_with((), move |_| {
        spawn_local(async move {
            let args = to_value(&()).unwrap();
            gloo_console::log!("get_value_invoke: {}", get_value_invoke.clone());
            let res = invoke(&get_value_invoke, args).await;
            let res = res.as_f64().unwrap();
            let res = res as i64;
            let mut config = config_1.config.clone();
            config.insert(label_1, res);
            config_dispatch.set(ConfigState { config });
        });
    });

    html! {
        <div class="flex flex-row justify-between">
            <label htmlFor={format!("{}-input-box", label)} class="text-xl my-2">
                {t!{&label}}
            </label>
            <input
                id={format!("{}-input-box", label)}
                type="number"
                class="border border-gray-200 rounded-md p-2 dark:text-black ml-5 flex-1"
                onchange={on_change}
                value={config.config.get(&label).unwrap().to_string()}
            />
        </div>
    }
}
