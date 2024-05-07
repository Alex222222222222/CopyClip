/*
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
*/

pub mod backward;
pub mod clip;
pub mod config;
pub mod error;
pub mod event;
pub mod export;
pub mod systray;

#[macro_use]
extern crate rust_i18n;
i18n!("../locales");
