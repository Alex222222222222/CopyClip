use std::{rc::Rc, sync::Arc};

use serde_wasm_bindgen::{from_value, to_value};
use tauri_plugin_logging::error;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, use_effect, Html, Properties};

use crate::{invoke::invoke, search::SearchConstraintStruct};

/// A list of all labels except "favourite" and "pinned"
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, yewdux::Store)]
#[store(storage = "local", storage_tab_sync)]
struct AllLabels {
    label: Vec<String>,
}

impl Default for AllLabels {
    fn default() -> Self {
        Self { label: Vec::new() }
    }
}

#[derive(Properties, PartialEq)]
struct LabelButtonProps {
    label: String,
}

#[function_component(SearchLabelsBarButton)]
fn search_labels_bar_button(LabelButtonProps { label }: &LabelButtonProps) -> Html {
    let (search_constraint, search_constraint_dispatch) =
        yewdux::use_store::<SearchConstraintStruct>();

    let label = Arc::new(label.clone());
    html! {
        <>{
            if search_constraint.neutral_label.contains(label.as_ref()) {
                let label_new = label.clone();
                html! {
                    <button class="py-1 px-2" onclick={move |_| {
                        search_constraint_dispatch.reduce_mut( |state| {
                            state.neutral_label.remove(label_new.as_ref());
                            state.exclude_label.insert(label_new.as_ref().clone());
                        });}}>
                        { label }
                    </button>
                }
            } else if search_constraint.exclude_label.contains(label.as_ref()) {
                let label_new = label.clone();
                html! {
                    <button class="py-1 px-2 bg-red-200" onclick={move |_| {
                        search_constraint_dispatch.reduce_mut( |state| {
                            state.exclude_label.remove(label_new.as_ref());
                        });}}>
                        {label}
                    </button>
                }
            } else {
                let label_new = label.clone();
                html! {
                    <button class="py-1 px-2 bg-green-200" onclick={move |_| {
                        search_constraint_dispatch.reduce_mut( |state| {
                            state.neutral_label.insert(label_new.as_ref().clone());
                        });}}>
                        {label}
                    </button>
                }
            }
        }</>
    }
}

#[yew::function_component(SearchLabelsBar)]
pub fn search() -> yew::Html {
    let (all_labels, all_labels_dispatch) = yewdux::use_store::<AllLabels>();

    // get all labels from the frontend
    use_effect(move || {
        spawn_local(async move {
            // call `get_all_labels` from the frontend
            let args = to_value(&()).unwrap();
            let res = invoke("get_all_labels", args).await;
            let res: Vec<String> = match from_value(res) {
                Ok(res) => res,
                Err(err) => {
                    error(format!("failed to convert res: {}", err));
                    return;
                }
            };

            all_labels_dispatch.reduce(move |_| Rc::new(AllLabels { label: res }));
        });
    });

    yew::html! {
        // a flex bar to display all labels with gray background color with a line separator
        <div class="flex flex-nowrap overflow-x-auto bg-gray-200 dark:bg-gray-800 text-black dark:text-white divide-x-2 divide-gray-600 dark:divide-gray-600 no-scrollbar">
            <SearchLabelsBarButton label="favorite" />
            <SearchLabelsBarButton label="pinned" />
            { for all_labels.label.iter().map(|label| {
                yew::html! {
                    <SearchLabelsBarButton label={label.clone()} />
                }
            }) }
        </div>
    }
}
