use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::{
    function_component, html, use_effect_with_deps, use_state, Callback, Html, Properties,
    TargetCast, UseStateHandle,
};

use crate::invoke::invoke;

#[derive(Serialize, Deserialize)]
struct SetDataArg {
    data: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct SwitcherConfigTemplateProps {
    pub label: String,
    pub default_value: bool,
    pub set_value_invoke: String,
    pub get_value_invoke: String,
}

#[function_component(SwitchConfigTemplate)]
pub fn switch_config_template(props: &SwitcherConfigTemplateProps) -> Html {
    let value_handle: UseStateHandle<bool> = use_state(|| props.default_value);
    let value = value_handle.clone();

    async fn handle_on_change(value: bool, set_value_invoke: String) {
        let args = to_value(&SetDataArg { data: value }).unwrap();
        invoke(&set_value_invoke, args).await;
    }

    let label = props.label.clone();

    let set_value_invoke = props.set_value_invoke.clone();
    let on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let value = value.parse::<bool>().unwrap();
        let res = handle_on_change(value, set_value_invoke.clone());
        spawn_local(res);
    });

    let get_value_invoke = props.get_value_invoke.clone();
    use_effect_with_deps(
        move |_| {
            spawn_local(async move {
                let args = to_value(&()).unwrap();
                let res = invoke(&get_value_invoke, args).await.as_string().unwrap();
                let res = res.parse::<bool>().unwrap();
                value_handle.set(res);
            });
        },
        (),
    );

    html! {
        <div class="flex flex-col">
            <label htmlFor="int-input-box" class="text-xl">
                {label.clone()}
            </label>
            <label class="switch">
                <input
                    type="checkbox"
                    id={format!("{}-input-box", label)}
                    onchange={on_change}
                    value={value.to_string()}
                />
                <span class="slider"></span>
            </label>
        </div>
    }
}
