//! Bevy application bootstrap for the graphical frontend.
//!
//! Engine setup stays here; lifecycle and gameplay behavior remain in their
//! frontend-independent systems and Bevy adapters.

use bevy::prelude::*;

use crate::{bevy_gameplay, bevy_lifecycle, bevy_presentation};

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
    bevy_lifecycle::install(&mut app);
    bevy_gameplay::install(&mut app);
    app.add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
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
