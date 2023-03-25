use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::{
    function_component, html, use_effect_with_deps, use_state, Callback, Html, Properties,
    TargetCast, UseStateHandle,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct EmptyArg {}

#[derive(Serialize, Deserialize)]
struct SetPerPageDataArg {
    data: i64,
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
    let value_handle: UseStateHandle<i64> = use_state(|| props.default_value);
    let value = value_handle.clone();

    async fn handle_on_change(value: i64, set_value_invoke: String) {
        let args = to_value(&SetPerPageDataArg { data: value }).unwrap();
        invoke(&set_value_invoke, args).await;
    }

    let label = props.label.clone();
    let label1 = label.clone();

    let set_value_invoke = props.set_value_invoke.clone();
    let on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let value = value.parse::<i64>().unwrap();
        let res = handle_on_change(value, set_value_invoke.clone());
        spawn_local(res);
    });

    let get_value_invoke = props.get_value_invoke.clone();
    use_effect_with_deps(
        move |_| {
            spawn_local(async move {
                let args = to_value(&EmptyArg {}).unwrap();
                let res = invoke(&get_value_invoke, args).await.as_string().unwrap();
                let res = res.parse::<i64>().unwrap();
                value_handle.set(res);
            });
        },
        label1,
    );

    html! {
        <div class="flex flex-col">
            <label htmlFor="int-input-box" class=" text-xl">
                {label.clone()}
            </label>
            <input
                id={format!("{}-input-box", label)}
                type="number"
                class="border border-gray-200 rounded-md p-2"
                onchange={on_change}
                value={value.to_string()}
            />
        </div>
    }
}
