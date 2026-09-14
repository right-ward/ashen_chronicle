mod bevy_app;
mod bevy_combat;
mod bevy_console;
mod bevy_feedback;
mod bevy_gameplay;
mod bevy_interactions;
mod bevy_lifecycle;
mod bevy_navigation_bridge;
mod bevy_presentation;
mod bevy_records;
mod bevy_runtime;
mod content;
#[cfg(not(target_os = "android"))]
mod desktop_storage;
mod events;
mod game;
mod game_paths;
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
pub mod rng;

pub fn run() {
    if let Err(error) = game_paths::GamePaths::initialize() {
        panic!("failed to initialize The Ashen Chronicle game root: {error}");
    }
    bevy_app::run();
}
