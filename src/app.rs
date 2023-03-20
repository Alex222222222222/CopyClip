use yew::prelude::*;
use yew_router::prelude::*;

use crate::preferences::Preferences;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/preferences")]
    Preferences,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <h1>{ "Home" }</h1> },
        Route::Preferences => html! {
            <Preferences />
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
