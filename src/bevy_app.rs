//! Bevy application foundation for the graphical frontend.
//!
//! This module owns only engine/bootstrap concerns. Gameplay state and rules
//! remain in the existing frontend-independent game modules and will be
//! integrated here incrementally by later v0.49.x migration issues.

use bevy::prelude::*;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const WINDOW_TITLE: &str = "The Ashen Chronicle";

pub(crate) fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_owned(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.08, 0.07, 0.09), Vec2::new(960.0, 540.0)),
        Transform::default(),
    ));
}

#[cfg(test)]
mod tests {
    use super::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};

    #[test]
    fn foundation_window_configuration_is_stable() {
        assert_eq!(WINDOW_TITLE, "The Ashen Chronicle");
        assert_eq!(WINDOW_WIDTH, 1280.0);
        assert_eq!(WINDOW_HEIGHT, 720.0);
    }
}
