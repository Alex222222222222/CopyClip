use std::collections::HashMap;

use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::{
    function_component, html, use_effect_with_deps, use_state_eq, Callback, Html, Properties,
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

#[function_component(OptionChooseConfigTemplate)]
pub fn option_choose_config_template_html(props: &OptionChooseConfigTemplateProps) -> Html {
    let selected = use_state_eq(|| props.default_value.clone());

    let set_value_invoke = props.set_value_invoke.clone();
    let select_on_change = Callback::from(move |event: Event| {
        let set_value_invoke_1 = set_value_invoke.clone();
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let args = SetValueArgs { data: value };
        let args = to_value(&args).unwrap();
        spawn_local(async move {
            invoke(&set_value_invoke_1, args).await;
        })
    });

    let get_value_invoke = props.get_value_invoke.clone();
    let selected_1 = selected.clone();
    use_effect_with_deps(
        move |_| {
            spawn_local(async move {
                let args = to_value(&()).unwrap();
                let get_value_invoke = get_value_invoke.clone();
                let res = invoke(&get_value_invoke, args).await.as_string().unwrap();
                selected_1.set(res);
            });
        },
        (),
    );

    let options = props.option.clone();
    html! {
        <>
            <label htmlFor="int-input-box" class="text-xl">
                {props.label.clone()}
            </label>
            // search method drop list
            <select
                class="border border-gray-200 rounded-md p-2 mx-2 text-lg"
                onchange={select_on_change}
            >
                {
                    options.into_iter().map(|(key,value)| {
                        html! {
                            <option value={key.clone()} selected={key == selected.to_string()}>
                                {value}
                            </option>
                        }
                    }).collect::<Html>()
                }
            </select>
        </>
    }
}
