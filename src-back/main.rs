/*
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::single_call_fn, clippy::str_to_string)]
*/

mod components;
mod i18n;
mod invoke;
mod pages;
/// Route table for the app
mod route;
/// Main search page
mod search;

// Load I18n macro, for allow you use `t!` macro in anywhere.
#[macro_use]
extern crate rust_i18n;

// Init translations for current crate.
// You must keep this path is same as the path you set `load-path` in [package.metadata.i18n] in Cargo.toml.
i18n!("locales", fallback = "en-GB");

fn main() {
    yew::Renderer::<route::Main>::new().render();
}
