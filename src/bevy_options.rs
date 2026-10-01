//! Dedicated Options screen for filesystem/game-data configuration.
//!
//! The screen owns only presentation and configuration requests. Runtime path
//! resolution remains centralized in game_paths, and Android shared-storage
//! access remains owned by MainActivity.

use bevy::prelude::*;

use crate::bevy_gameplay::GameplayState;
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId, UiSelected,
};
use crate::input::InputEvent;

#[derive(Resource)]
pub(crate) struct OptionsState {
    selected: usize,
    message: Option<String>,
    active: bool,
    dirty: bool,
    #[cfg(target_os = "android")]
    shared_storage_name: Option<String>,
    #[cfg(target_os = "android")]
    last_storage_poll: f32,
}

impl Default for OptionsState {
    fn default() -> Self {
        Self {
            selected: 0,
            message: None,
            active: false,
            dirty: true,
            #[cfg(target_os = "android")]
            shared_storage_name: None,
            #[cfg(target_os = "android")]
            last_storage_poll: 0.0,
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<OptionsState>().add_systems(
        Update,
        (
            sync_screen,
            refresh_storage,
            options_input,
            render_if_active,
        )
            .chain(),
    );
}

fn sync_screen(
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut options: ResMut<OptionsState>,
) {
    let active = matches!(
        lifecycle.phase,
        LifecyclePhase::Start | LifecyclePhase::Complete
    ) && navigation.current_screen == Some(ScreenId::Options);

    if active && !options.active {
        options.active = true;
        options.selected = 0;
        options.message = None;
        options.dirty = true;

        #[cfg(target_os = "android")]
        {
            options.shared_storage_name = android_shared_storage_name();
            options.last_storage_poll = 0.0;
        }
    } else if !active {
        options.active = false;
    }
}

#[cfg(target_os = "android")]
fn refresh_storage(
    time: Res<Time>,
    navigation: Res<NavigationState>,
    mut options: ResMut<OptionsState>,
) {
    if navigation.current_screen != Some(ScreenId::Options) {
        return;
    }

    let elapsed = time.elapsed_secs();
    if elapsed - options.last_storage_poll < 0.5 {
        return;
    }
    options.last_storage_poll = elapsed;

    let current = android_shared_storage_name();
    if current != options.shared_storage_name {
        options.shared_storage_name = current;
        options.dirty = true;
    }
}

#[cfg(not(target_os = "android"))]
fn refresh_storage(
    _time: Res<Time>,
    _navigation: Res<NavigationState>,
    _options: ResMut<OptionsState>,
) {
}

fn options_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut options: ResMut<OptionsState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if !matches!(
        lifecycle.phase,
        LifecyclePhase::Start | LifecyclePhase::Complete
    ) || navigation.current_screen != Some(ScreenId::Options)
        || input_queue.0.is_empty()
    {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up => options.selected = options.selected.saturating_sub(1),
            InputEvent::Down => options.selected = (options.selected + 1).min(1),
            InputEvent::Home => options.selected = 0,
            InputEvent::End => options.selected = 1,
            InputEvent::Confirm => match options.selected {
                0 => request_game_data_root_change(&mut options),
                1 => close_options(&mut lifecycle, &mut navigation, &mut options),
                _ => {}
            },
            InputEvent::Cancel => close_options(&mut lifecycle, &mut navigation, &mut options),
            _ => {}
        }
        options.dirty = true;
    }
    navigation.selected = options.selected;
}

fn close_options(
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    options: &mut OptionsState,
) {
    let return_screen = navigation.return_screen.take();
    options.active = false;
    options.selected = 0;
    options.message = None;
    options.dirty = true;

    match return_screen {
        Some(ScreenId::Lifecycle) => {
            navigation.current_screen = Some(ScreenId::Lifecycle);
            navigation.selected = lifecycle.selected;
            lifecycle.mark_dirty();
        }
        _ => {
            navigation.current_screen = Some(ScreenId::Gameplay);
            navigation.selected = 0;
        }
    }
}

#[cfg(not(target_os = "android"))]
fn request_game_data_root_change(options: &mut OptionsState) {
    let Some(folder) = rfd::FileDialog::new()
        .set_title("Choose The Ashen Chronicle game data root")
        .pick_folder()
    else {
        options.message = Some("Folder selection was cancelled.".to_string());
        return;
    };

    match crate::game_paths::set_configured_game_root(&folder) {
        Ok(()) => {
            options.message = Some(format!(
                "Game data root set to {}. Restart the game to use the new root.",
                folder.display()
            ));
        }
        Err(error) => {
            options.message = Some(format!("Could not save the game data root: {error}"));
        }
    }
}

#[cfg(target_os = "android")]
fn request_game_data_root_change(options: &mut OptionsState) {
    if android_request_storage_picker() {
        options.message =
            Some("Choose or create the shared game folder in the Android picker.".to_string());
    } else {
        options.message = Some("The Android storage picker is unavailable.".to_string());
    }
}

fn render_if_active(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut options: ResMut<OptionsState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    if !matches!(
        lifecycle.phase,
        LifecyclePhase::Start | LifecyclePhase::Complete
    ) || navigation.current_screen != Some(ScreenId::Options)
        || !options.active
        || !options.dirty
    {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }

    let root = bevy_presentation::spawn_screen(&mut commands, "OPTIONS");
    let panel = bevy_presentation::spawn_surface(
        &mut commands,
        root,
        bevy_presentation::SurfaceTone::Strong,
    );

    bevy_presentation::spawn_muted_label(
        &mut commands,
        panel,
        "Settings are grouped by purpose. Only Game Data is configured here.",
    );

    let tab_strip = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Row,
            column_gap: bevy_presentation::responsive_compact_gap(),
            ..default()
        })
        .id();
    commands.entity(panel).add_child(tab_strip);

    commands.entity(tab_strip).with_children(|tabs| {
        tabs.spawn((
            Node {
                min_width: px(120),
                min_height: bevy_presentation::touch_target_size(),
                padding: UiRect::axes(bevy_presentation::responsive_surface_padding(), px(6)),
                border: UiRect::all(px(1)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_ACCENT),
            BackgroundColor(bevy_presentation::THEME_SELECTED),
            children![(
                Text::new("Game Data"),
                bevy_presentation::TextContent,
                TextFont::from_font_size(bevy_presentation::muted_font_size()),
                TextColor(bevy_presentation::THEME_TEXT),
            )],
        ));
    });

    bevy_presentation::spawn_label(&mut commands, panel, "GAME DATA");

    #[cfg(not(target_os = "android"))]
    let root_label = crate::game_paths::configured_game_root()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "No configured desktop game data root.".to_string());

    #[cfg(target_os = "android")]
    let root_label = options
        .shared_storage_name
        .as_deref()
        .map(|name| format!("Shared folder: {name}"))
        .unwrap_or_else(|| "No shared game folder selected.".to_string());

    bevy_presentation::spawn_muted_label(&mut commands, panel, root_label);

    #[cfg(not(target_os = "android"))]
    bevy_presentation::spawn_muted_label(
        &mut commands,
        panel,
        "The configured desktop root is stored separately from the game data itself and takes effect on the next launch.",
    );

    #[cfg(target_os = "android")]
    bevy_presentation::spawn_muted_label(
        &mut commands,
        panel,
        "Android keeps the Rust working root app-scoped and mirrors editable mods and saves to this selected folder.",
    );

    if let Some(message) = &options.message {
        bevy_presentation::spawn_context_message(&mut commands, panel, message);
    }

    let change = bevy_presentation::spawn_action_button(
        &mut commands,
        panel,
        0,
        "⌂",
        "Change game data folder",
    );
    if options.selected == 0 {
        commands.entity(change).insert(UiSelected);
    }

    let back = bevy_presentation::spawn_action_button(&mut commands, panel, 1, "←", "Back");
    if options.selected == 1 {
        commands.entity(back).insert(UiSelected);
    }

    options.dirty = false;
}

#[cfg(target_os = "android")]
fn android_shared_storage_name() -> Option<String> {
    let app = bevy::android::ANDROID_APP.get()?.clone();
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) };
    vm.attach_current_thread(|env| -> jni::errors::Result<Option<String>> {
        let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
        let activity =
            unsafe { env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)? };
        let value = env.call_method(
            activity.as_ref(),
            jni::jni_str!("getSharedStorageDisplayName"),
            jni::jni_sig!("()Ljava/lang/String;"),
            &[],
        )?;
        let object = value.l()?;
        if object.as_raw().is_null() {
            return Ok(None);
        }
        let string = jni::objects::JString::cast_local(env, object)?;
        Ok(Some(string.try_to_string(env)?))
    })
    .ok()
    .flatten()
}

#[cfg(target_os = "android")]
fn android_request_storage_picker() -> bool {
    let Some(app) = bevy::android::ANDROID_APP.get().cloned() else {
        return false;
    };
    let java_app = app.clone();
    app.run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(java_app.vm_as_ptr().cast()) };
        if let Err(error) = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = java_app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            env.call_method(
                activity.as_ref(),
                jni::jni_str!("requestStorageTreeFromOptions"),
                jni::jni_sig!("()V"),
                &[],
            )?;
            Ok(())
        }) {
            bevy::log::warn!("Could not open Android storage picker: {error}");
        }
    }));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_options_from_start_returns_to_lifecycle() {
        let mut lifecycle = LifecycleState::default();
        lifecycle.selected = 1;
        let mut navigation = NavigationState {
            current_screen: Some(ScreenId::Options),
            return_screen: Some(ScreenId::Lifecycle),
            selected: 0,
        };
        let mut options = OptionsState {
            active: true,
            ..OptionsState::default()
        };

        close_options(&mut lifecycle, &mut navigation, &mut options);

        assert_eq!(navigation.current_screen, Some(ScreenId::Lifecycle));
        assert_eq!(navigation.selected, 1);
        assert_eq!(lifecycle.selected, 1);
        assert!(!options.active);
    }

    #[test]
    fn options_navigation_has_two_actions() {
        let mut options = OptionsState::default();
        options.selected = 0;
        options.selected = (options.selected + 1).min(1);
        assert_eq!(options.selected, 1);
        options.selected = options.selected.saturating_sub(1);
        assert_eq!(options.selected, 0);
    }
}
