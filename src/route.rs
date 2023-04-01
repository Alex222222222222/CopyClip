use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::preferences::language_config::LanguagesConfigState,
    invoke::invoke,
    pages::{home::Home, preferences::Preferences, search::Search},
};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/preferences")]
    Preferences,
    #[at("/search")]
    Search,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {
           <Home />
        },
        Route::Preferences => html! {
            <Preferences />
        },
        Route::Search => html! {
            <Search />
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Main)]
pub fn app() -> Html {
    // TODO fix the bug that change the language will force the dark mode config to disappear
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

    let (language_config, _) = yewdux::prelude::use_store::<LanguagesConfigState>();
    rust_i18n::set_locale(&language_config.config);

    html! {
        <div
            class="bg-white dark:bg-gray-800 text-black dark:text-white min-h-screen"
        >
            <BrowserRouter>
                    <Switch<Route> render={switch}/> // <- must be child of <BrowserRouter>
            </BrowserRouter>
        </div>
    }
}
