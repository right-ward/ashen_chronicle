mod content;
mod events;
mod game;
mod model;
mod persistence;
mod presentation;
mod ui_components;
#[allow(dead_code)]
pub mod ui;

fn main() {
    if let Err(err) = game::run() {
        eprintln!("Fatal error: {err}");
        std::process::exit(1);
    }
}
