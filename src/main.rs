#[cfg(feature = "bevy")]
mod bevy_app;
#[cfg(feature = "bevy")]
mod bevy_combat;
#[cfg(feature = "bevy")]
mod bevy_console;
#[cfg(feature = "bevy")]
mod bevy_gameplay;
#[cfg(feature = "bevy")]
mod bevy_interactions;
#[cfg(feature = "bevy")]
mod bevy_lifecycle;
#[cfg(feature = "bevy")]
mod bevy_navigation_bridge;
#[cfg(feature = "bevy")]
mod bevy_presentation;
#[cfg(feature = "bevy")]
mod bevy_records;
mod content;
// Retained renderer-neutral compatibility surfaces are intentionally linted separately
// while their Bevy adapters become the sole shipped frontend.
#[allow(dead_code)]
mod events;
#[allow(dead_code)]
mod game;
mod input;
mod model;
mod persistence;
#[allow(dead_code)]
mod presentation;
pub mod procedural;
pub mod procedural_authored;
pub mod procedural_characteristics;
pub mod procedural_entities;
pub mod procedural_opportunities;
pub mod procedural_relationships;
mod rng;
#[allow(dead_code)]
pub mod ui;

fn main() {
    bevy_app::run();
}
