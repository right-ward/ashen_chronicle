//! Bevy application foundation for the graphical frontend.
//!
//! This module owns engine/bootstrap concerns. Gameplay state and rules remain
//! in the existing frontend-independent game modules and are integrated here
//! incrementally by later v0.49.x migration issues.

use bevy::prelude::*;

use crate::bevy_presentation;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &str = "The Ashen Chronicle";

pub(crate) fn run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE.to_owned(),
            resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
            resizable: true,
            ..default()
        }),
        ..default()
    }));
    bevy_presentation::install(&mut app);
    app.add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let screen = bevy_presentation::spawn_screen(&mut commands, WINDOW_TITLE);
    let panel = bevy_presentation::spawn_panel(&mut commands, screen);
    bevy_presentation::spawn_muted_label(
        &mut commands,
        panel,
        "The graphical frontend is ready for incremental screen migration.",
    );
    bevy_presentation::spawn_gauge(&mut commands, panel, 7, 10);
    bevy_presentation::spawn_choice_button(&mut commands, panel, 0, "Continue");
    bevy_presentation::spawn_choice_button(&mut commands, panel, 1, "Open Character");
    bevy_presentation::spawn_choice_button(&mut commands, panel, 2, "Open Inventory");
}

#[cfg(test)]
mod tests {
    use super::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};

    #[test]
    fn foundation_window_configuration_is_stable() {
        assert_eq!(WINDOW_TITLE, "The Ashen Chronicle");
        assert_eq!(WINDOW_WIDTH, 1280);
        assert_eq!(WINDOW_HEIGHT, 720);
    }
}
