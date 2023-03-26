mod components;
mod invoke;
mod pages;
mod route;

// TODO add i18n support
// TODO add export database and config file

fn main() {
    yew::Renderer::<route::Main>::new().render();
}
