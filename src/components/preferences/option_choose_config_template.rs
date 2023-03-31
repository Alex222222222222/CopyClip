use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::{
    function_component, html, use_effect_with_deps, Callback, Html, Properties,
    TargetCast,
};

use crate::invoke::invoke;

#[derive(PartialEq, Properties)]
pub struct OptionChooseConfigTemplateProps {
    /// key, display value
    pub option: HashMap<String, String>,
    pub label: String,
    /// key of default value
    pub default_value: String,
    pub set_value_invoke: String,
    pub get_value_invoke: String,
}

#[derive(Serialize)]
pub struct SetValueArgs {
    pub data: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, Eq, yewdux::prelude::Store)]
#[store(storage = "local")]
struct ChooseOptionState {
    config: HashMap<String, String>,
}

#[function_component(OptionChooseConfigTemplate)]
pub fn option_choose_config_template_html(props: &OptionChooseConfigTemplateProps) -> Html {
    let (config, config_dispatch) = yewdux::prelude::use_store::<ChooseOptionState>();
    if !config.config.contains_key(&props.label) {
        let mut config = config.config.clone();
        config.insert(props.label.clone(), props.default_value.clone());
        config_dispatch.set(ChooseOptionState { config });
    }

    let set_value_invoke = props.set_value_invoke.clone();
    let config_1 = config.clone();
    let config_dispatch_1 = config_dispatch.clone();
    let label = props.label.clone();
    let select_on_change = Callback::from(move |event: Event| {
        let set_value_invoke_1 = set_value_invoke.clone();
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let args = SetValueArgs { data: value.clone() };
        let args = to_value(&args).unwrap();
        let mut config = config_1.config.clone();
        config.insert(label.clone(), value);
        config_dispatch_1.set(ChooseOptionState { config });
        spawn_local(async move {
            invoke(&set_value_invoke_1, args).await;
        })
    });

    let get_value_invoke = props.get_value_invoke.clone();
    let config_1 = config.clone();
    let config_dispatch_1 = config_dispatch;
    let label = props.label.clone();
    use_effect_with_deps(
        
        move |_| {
            spawn_local(async move {
                let args = to_value(&()).unwrap();
                let get_value_invoke = get_value_invoke.clone();
                let res = invoke(&get_value_invoke, args).await.as_string().unwrap();
                let mut config = config_1.config.clone();
                 config.insert(label, res);
                config_dispatch_1.set(ChooseOptionState { config });
            });
        },
        (),
    );

    let options = props.option.clone();
    html! {
        <div
            class="flex flex-row justify-between"
        >
            <label class="text-xl">
                {props.label.clone()}
            </label>
            // search method drop list
            <select
                class="border border-gray-200 rounded-md p-2 text-lg dark:text-black"
                onchange={select_on_change}
            >
                {
                    options.into_iter().map(|(key,value)| {
                        html! {
                            <option
                                value={key.clone()}
                                selected={key == *config.config.get(&props.label).unwrap()}
                            >
                                {value}
                            </option>
                        }
                    }).collect::<Html>()
                }
            </select>
        </div>
    }
}
