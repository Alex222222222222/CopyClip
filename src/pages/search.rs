use yew::{function_component, html, Html};

use crate::components::head_bar::HeadBar;

#[function_component(Search)]
pub fn search() -> Html {
    html! {
        <div>
            <HeadBar></HeadBar>
            <div class="flex min-h-screen flex-col bg-white">
                <h1 class="text-center text-6xl m-0">{ "Search" }</h1>
            </div>
        </div>
    }
}
