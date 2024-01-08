use yew::{function_component, html, Html};

use crate::components::head_bar::HeadBar;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <div class="h-full">
            <HeadBar></HeadBar>
            <div class="flex flex-col">
                <h1 class="text-center text-6xl m-0">{ "Home" }</h1>

                <div
                    class="mx-3 my-2 text-lg"
                >
                    {"For more information please visit the repo on "}
                    <a
                        class="text-blue-500"
                        href="https://github.com/Alex222222222222/CopyClip"
                        target="_blank"
                    >{"GitHub"}</a>
                    {"."}

                    <br />

                    {"更多详情请查看: "}
                    <a
                        class="text-blue-500"
                        href="https://github.com/Alex222222222222/CopyClip"
                        target="_blank"
                    >{"GitHub"}</a>
                    {"."}

                    <br />

                    {"Für weitere Informationen besuchen Sie bitte das Repo auf "}
                    <a
                        class="text-blue-500"
                        href="https://github.com/Alex222222222222/CopyClip"
                        target="_blank"
                    >{"GitHub"}</a>
                    {"."}
                </div>
            </div>
        </div>
    }
}
