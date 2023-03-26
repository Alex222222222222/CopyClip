mod components;
mod invoke;
mod pages;
mod route;

// TODO add i18n support

fn main() {
    yew::Renderer::<route::Main>::new().render();
}
