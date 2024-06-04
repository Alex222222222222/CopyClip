use serde::{Deserialize, Serialize};

use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::{functional::use_store, store::Store};

use crate::{
    components::preferences::language_config::LanguagesConfigState,
    pages::{home::Home, preferences::Preferences},
};

#[derive(Clone, Routable, PartialEq)]
/// Routes for the app
pub enum Route {
    #[at("/")]
    /// Home page
    Home,
    #[at("/preferences")]
    /// Preferences page
    Preferences,
    #[not_found]
    #[at("/404")]
    /// 404 page
    NotFound,
}

/// Renderer for the router
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {
            <Home />
        },
        Route::Preferences => html! {
            <Preferences />
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[derive(Clone, Debug, Store, PartialEq, Deserialize, Serialize, Default)]
#[store(storage = "local")]
/// Yewdux store for dark mode
pub struct DarkModeConfig {
    /// Whether dark mode is enabled
    pub is_dark: bool,
}

#[function_component(Main)]
pub fn app() -> Html {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let element = document.document_element().unwrap();
    let class_list = element.class_list();
    let (dark_mode_config, _) = use_store::<DarkModeConfig>();
    if dark_mode_config.is_dark {
        class_list.add_1("dark").unwrap();
    } else {
        class_list.remove_1("dark").unwrap();
    }

    let (language_config, _) = use_store::<LanguagesConfigState>();
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
