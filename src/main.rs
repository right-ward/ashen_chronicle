mod content;
mod events;
mod game;
mod input;
mod model;
mod persistence;
mod presentation;
mod procedural;
pub mod ui;
mod ui_components;

fn main() {
    if let Err(err) = game::run() {
        eprintln!("Fatal error: {err}");
        std::process::exit(1);
    }
}
