use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{home::Home, index::Index, preferences::Preferences};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/preferences")]
    Preferences,
    #[at("/search")]
    Search,
    #[at("/index")]
    Index,
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
        Route::Search => html! { <h1>{ "Search" }</h1> },
        Route::Index => html! {
            <Index />
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Main)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} /> // <- must be child of <BrowserRouter>
        </BrowserRouter>
    }
}
