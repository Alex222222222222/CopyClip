mod app;
mod components;
mod index;
mod preferences;

fn main() {
    yew::Renderer::<app::Main>::new().render();
}
