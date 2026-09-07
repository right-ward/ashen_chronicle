mod content;
mod events;
mod game;
mod input;
mod model;
mod persistence;
mod presentation;
pub mod procedural;
pub mod procedural_authored;
pub mod procedural_characteristics;
pub mod procedural_entities;
pub mod procedural_opportunities;
pub mod procedural_relationships;
mod rng;
pub mod ui;
mod ui_components;

fn main() {
    if let Err(err) = game::run() {
        eprintln!("Fatal error: {err}");
        std::process::exit(1);
    }
}
