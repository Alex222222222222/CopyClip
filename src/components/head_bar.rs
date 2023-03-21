/// the is the head bar of the application
/// it contains the menu bar and the search bar
/// it is the first component to be loaded
/// it is the parent of the menu bar and the search bar
use crate::route::Route;

use yew::{function_component, html, Html};
use yew_router::prelude::Link;

#[function_component(HeadBar)]
pub fn head_bar() -> Html {
    html! {
        <div class="m-2 p-2">
            <div class="grid grid-cols-4 content-center gap-4 text-xl bg-gray-300 rounded-md">
                <div class="mx-2 text-center">
                    <a class="text-blue-500 hover:text-blue-800">
                        <Link<Route> to={Route::Home}>
                            {"Home"}
                        </Link<Route>>
                    </a>
                </div>
                <div class="mx-2 text-center">
                    <a class="text-blue-500 hover:text-blue-800">
                        <Link<Route> to={Route::Preferences}>
                            {"Preferences"}
                        </Link<Route>>
                    </a>
                </div>
                <div class="mx-2 text-center">
                    <a class="text-blue-500 hover:text-blue-800">
                        <Link<Route> to={Route::Search}>
                            {"Search"}
                        </Link<Route>>
                    </a>
                </div>
                <div class="mx-2 text-center">
                    <a class="text-blue-500 hover:text-blue-800" href="https://github.com/Alex222222222222/CopyClip" target="_blank">{"Github"}</a>
                </div>
            </div>
        </div>
    }
}

// Path: src/components/menu_bar.rs
