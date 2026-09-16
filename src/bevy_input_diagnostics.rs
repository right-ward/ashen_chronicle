//! Android input diagnostics for device-side event delivery.

use bevy::prelude::*;

pub fn install(app: &mut App) {
    #[cfg(target_os = "android")]
    app.add_systems(Update, (log_ime_events, log_browser_back));
}

#[cfg(target_os = "android")]
fn log_ime_events(mut ime: MessageReader<Ime>) {
    for event in ime.read() {
        bevy::log::info!("Android IME event: {event:?}");
    }
}

#[cfg(target_os = "android")]
fn log_browser_back(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::BrowserBack) {
        bevy::log::info!("Android BrowserBack key event received");
    }
}
