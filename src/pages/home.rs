use yew::{function_component, html, Html};

use crate::components::head_bar::HeadBar;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <div class="h-full">
            <HeadBar></HeadBar>
            <div class="flex flex-col">
                <h1 class="text-center text-6xl m-0">{ "Home" }</h1>
            </div>
        </div>
    }
}
