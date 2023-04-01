use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::{function_component, html, use_effect_with_deps, Callback, Html, TargetCast};

use crate::invoke::invoke;

#[derive(Serialize)]
struct SetValueArgs {
    pub data: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, Eq, yewdux::prelude::Store)]
#[store(storage = "local")]
pub struct LanguagesConfigState {
    pub config: String,
}

#[function_component(LanguagesConfig)]
pub fn languages_config() -> Html {
    let get_value_invoke = "get_language";
    let set_value_invoke = "set_language";
    let default_value = "en-GB";
    let options = crate::i18n::I18N.clone();

    let (config, config_dispatch) = yewdux::prelude::use_store::<LanguagesConfigState>();
    if config.config.is_empty() {
        config_dispatch.set(LanguagesConfigState {
            config: default_value.to_string(),
        });
    }

    let config_dispatch_1 = config_dispatch.clone();
    let select_on_change = Callback::from(move |event: Event| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let args = SetValueArgs {
            data: value.clone(),
        };
        let args = to_value(&args).unwrap();
        rust_i18n::set_locale(&value);
        config_dispatch_1.set(LanguagesConfigState { config: value });

        let l = web_sys::window().unwrap().location();
        let res = l.reload();
        if let Err(e) = res {
            gloo_console::error!("Error reloading page: {}", e);
        }

        spawn_local(async move {
            invoke(set_value_invoke, args).await;
        });

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let element = document.document_element().unwrap();
        let class_list = element.class_list();
        spawn_local(async move {
            let res = invoke("get_dark_mode", to_value(&()).unwrap()).await;
            let is_dark = res.as_bool().unwrap();
            if is_dark {
                class_list.add_1("dark").unwrap();
            } else {
                class_list.remove_1("dark").unwrap();
            }
        });
    });

    let config_dispatch_1 = config_dispatch;
    use_effect_with_deps(
        move |_| {
            spawn_local(async move {
                let args = to_value(&()).unwrap();
                let res = invoke(get_value_invoke, args).await.as_string().unwrap();
                config_dispatch_1.set(LanguagesConfigState { config: res });
            });
        },
        (),
    );

    html! {
        <div
            class="flex flex-row justify-between"
        >
            <label class="text-xl">
                {t!("preferences.language")}
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
                                selected={key == config.config}
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
