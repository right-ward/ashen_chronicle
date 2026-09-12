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

fn main() {
    bevy_app::run();
}
