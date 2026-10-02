//! Bevy application bootstrap for the graphical frontend.
//!
//! Engine setup stays here; lifecycle and gameplay behavior remain in their
//! frontend-independent systems and Bevy adapters.

#[cfg(not(target_os = "android"))]
use crate::desktop_display;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::prelude::*;
use bevy_picking::events::{Pointer, Release};

use crate::{
    bevy_combat, bevy_console, bevy_feedback, bevy_gameplay, bevy_input_diagnostics,
    bevy_interactions, bevy_lifecycle, bevy_navigation_bridge, bevy_options, bevy_presentation,
    bevy_records, bevy_runtime, bevy_touch,
};

#[cfg(target_os = "android")]
const WINDOW_WIDTH: u32 = 1280;
#[cfg(target_os = "android")]
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &str = "The Ashen Chronicle";

pub fn run() {
    #[cfg(not(target_os = "android"))]
    let display_mode = desktop_display::load();
    #[cfg(not(target_os = "android"))]
    let window_resolution = desktop_display::window_resolution(display_mode);
    #[cfg(target_os = "android")]
    let window_resolution = (WINDOW_WIDTH, WINDOW_HEIGHT);

    let mut app = App::new();
    // Bevy 0.19.1's EditableText widget installs a Pointer<Release> reader even
    // without the picking plugins. Register its message storage without enabling
    // Bevy picking, which would conflict with our custom touch interaction layer.
    app.add_message::<Pointer<Release>>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE.to_owned(),
            resolution: window_resolution.into(),
            #[cfg(not(target_os = "android"))]
            mode: desktop_display::window_mode(display_mode),
            resizable: true,
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(TabNavigationPlugin);
    bevy_presentation::install(&mut app);
    bevy_touch::install(&mut app);
    bevy_input_diagnostics::install(&mut app);
    bevy_runtime::install(&mut app);
    bevy_lifecycle::install(&mut app);
    bevy_gameplay::install(&mut app);
    bevy_combat::install(&mut app);
    bevy_console::install(&mut app);
    bevy_navigation_bridge::install(&mut app);
    bevy_options::install(&mut app);
    bevy_records::install(&mut app);
    bevy_interactions::install(&mut app);
    bevy_feedback::install(&mut app);
    #[cfg(not(target_os = "android"))]
    app.add_systems(Update, desktop_display::apply_window_icon);
    app.add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[cfg(test)]
#[cfg(target_os = "android")]
mod tests {
    use super::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};

    #[test]
    fn foundation_window_configuration_is_stable() {
        assert_eq!(WINDOW_TITLE, "The Ashen Chronicle");
        assert_eq!(WINDOW_WIDTH, 1280);
        assert_eq!(WINDOW_HEIGHT, 720);
    }
}
