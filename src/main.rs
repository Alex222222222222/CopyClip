mod components;
mod i18n;
mod invoke;
mod pages;
mod route;

// Load I18n macro, for allow you use `t!` macro in anywhere.
#[macro_use]
extern crate rust_i18n;

// Init translations for current crate.
// You must keep this path is same as the path you set `load-path` in [package.metadata.i18n] in Cargo.toml.
i18n!("locales");

fn main() {
    yew::Renderer::<route::Main>::new().render();
}
