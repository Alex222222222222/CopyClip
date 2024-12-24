use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
/// the button used to export the user data
use yew::{function_component, html, Callback, Html};

use crate::invoke::invoke;
#[function_component(ExportButton)]
pub fn export_button() -> Html {
    let export_button_on_click = Callback::from(|_| {
        spawn_local(async move {
            let args = to_value(&()).unwrap();
            invoke("export_data_invoke", args).await;
        });
    });

    html! (
        // search button
        <button
            class="search-button bg-black my-2"
            onclick={export_button_on_click}
        >
            <span
                class="dark:bg-white dark:text-black text-white"
            > {t!("export.export_button")} </span>
        </button>
    )
}
