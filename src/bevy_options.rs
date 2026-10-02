//! Dedicated Options screen for filesystem/game-data configuration.
//!
//! The screen owns only presentation and configuration requests. Runtime path
//! resolution remains centralized in game_paths, and Android shared-storage
//! access remains owned by MainActivity.

use bevy::prelude::*;

#[cfg(not(target_os = "android"))]
use crate::desktop_display;
use crate::bevy_gameplay::GameplayState;
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId, UiSelected,
};
use crate::input::InputEvent;

#[derive(Resource)]
pub(crate) struct OptionsState {
    selected: usize,
    tab: usize,
    message: Option<String>,
    active: bool,
    dirty: bool,
    #[cfg(not(target_os = "android"))]
    display_mode: desktop_display::DisplayMode,
    #[cfg(not(target_os = "android"))]
    pending_display_mode: Option<desktop_display::DisplayMode>,
    #[cfg(target_os = "android")]
    shared_storage_name: Option<String>,
    #[cfg(target_os = "android")]
    last_storage_poll: f32,
}

impl Default for OptionsState {
    fn default() -> Self {
        Self {
            selected: 0,
            tab: 0,
            message: None,
            active: false,
            dirty: true,
            #[cfg(not(target_os = "android"))]
            display_mode: desktop_display::load(),
            #[cfg(not(target_os = "android"))]
            pending_display_mode: None,
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
            options_tab_input,
            options_input,
            apply_display_mode,
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
        options.tab = 0;
        options.message = None;
        #[cfg(not(target_os = "android"))]
        {
            options.display_mode = desktop_display::load();
            options.pending_display_mode = None;
        }
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

#[derive(Component, Clone, Copy)]
struct OptionsTabButton(usize);

fn options_tab_input(
    mut interactions: Query<(&Interaction, &OptionsTabButton), Changed<Interaction>>,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut options: ResMut<OptionsState>,
) {
    if !matches!(
        lifecycle.phase,
        LifecyclePhase::Start | LifecyclePhase::Complete
    ) || navigation.current_screen != Some(ScreenId::Options)
    {
        return;
    }

    for (interaction, tab) in &mut interactions {
        if *interaction != Interaction::Pressed || tab.0 == options.tab {
            continue;
        }

        options.tab = tab.0;
        options.selected = 0;
        options.message = None;
        #[cfg(not(target_os = "android"))]
        {
            options.pending_display_mode = None;
        }
        options.dirty = true;
    }
}

fn options_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut options: ResMut<OptionsState>,
    gameplay: Res<GameplayState>,
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
            InputEvent::Up => {
                options.selected = options.selected.saturating_sub(1);
            }
            InputEvent::Down => {
                options.selected = (options.selected + 1).min(options_selection_end(options.tab));
            }
            InputEvent::Home => options.selected = 0,
            InputEvent::End => options.selected = options_selection_end(options.tab),
            InputEvent::Tab => {
                options.tab = (options.tab + 1) % 2;
                options.selected = 0;
                options.message = None;
                #[cfg(not(target_os = "android"))]
                {
                    options.pending_display_mode = None;
                }
            }
            InputEvent::Confirm => match options.tab {
                0 => match options.selected {
                    0 => request_game_data_root_change(&mut options),
                    1 => close_options(&mut lifecycle, &mut navigation, &mut options, &gameplay),
                    _ => {}
                },
                1 => {
                    #[cfg(not(target_os = "android"))]
                    match options.selected {
                        0 => select_display_mode(
                            &mut options,
                            desktop_display::DisplayMode::Fullscreen,
                        ),
                        1 => select_display_mode(
                            &mut options,
                            desktop_display::DisplayMode::Windowed1920x1080,
                        ),
                        2 => select_display_mode(
                            &mut options,
                            desktop_display::DisplayMode::Windowed1280x720,
                        ),
                        3 => close_options(
                            &mut lifecycle,
                            &mut navigation,
                            &mut options,
                            &gameplay,
                        ),
                        _ => {}
                    }

                    #[cfg(target_os = "android")]
                    if options.selected == 0 {
                        close_options(
                            &mut lifecycle,
                            &mut navigation,
                            &mut options,
                            &gameplay,
                        );
                    }
                }
                _ => {}
            },
            InputEvent::Cancel => {
                close_options(&mut lifecycle, &mut navigation, &mut options, &gameplay)
            }
            _ => {}
        }
        options.dirty = true;
    }
    navigation.selected = options.selected;
}

fn options_selection_end(tab: usize) -> usize {
    match tab {
        0 => 1,
        1 => {
            #[cfg(not(target_os = "android"))]
            {
                3
            }
            #[cfg(target_os = "android")]
            {
                0
            }
        }
        _ => 1,
    }
}

#[cfg(not(target_os = "android"))]
fn select_display_mode(options: &mut OptionsState, mode: desktop_display::DisplayMode) {
    options.display_mode = mode;
    options.pending_display_mode = Some(mode);
    match desktop_display::save(mode) {
        Ok(()) => {
            options.message = Some(format!("Display mode: {}.", mode.label()));
        }
        Err(error) => {
            options.message = Some(format!(
                "Display mode applied, but could not be saved: {error}"
            ));
        }
    }
}

#[cfg(not(target_os = "android"))]
fn apply_display_mode(
    mut options: ResMut<OptionsState>,
    mut window: Single<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    let Some(mode) = options.pending_display_mode.take() else {
        return;
    };
    desktop_display::apply(&mut window, mode);
}

#[cfg(target_os = "android")]
fn apply_display_mode() {}

fn close_options(
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    options: &mut OptionsState,
    gameplay: &GameplayState,
) {
    let return_screen = navigation.return_screen.take();
    options.active = false;
    options.selected = 0;
    options.tab = 0;
    options.message = None;
    options.dirty = true;
    #[cfg(not(target_os = "android"))]
    {
        options.pending_display_mode = None;
    }

    match return_screen {
        Some(ScreenId::Lifecycle) => {
            navigation.current_screen = Some(ScreenId::Lifecycle);
            navigation.selected = lifecycle.selected;
            lifecycle.mark_dirty();
        }
        _ => {
            navigation.current_screen = Some(ScreenId::Gameplay);
            navigation.selected = gameplay.navigation_selection();
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
        "Settings are grouped by purpose.",
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
        for (index, label) in [(0, "Game Data"), (1, "Display")] {
            let mut tab = tabs.spawn((
                Button,
                OptionsTabButton(index),
                bevy_presentation::UiStyledButton,
                Node {
                    min_width: px(120),
                    min_height: bevy_presentation::touch_target_size(),
                    padding: UiRect::axes(bevy_presentation::responsive_surface_padding(), px(6)),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(bevy_presentation::THEME_BORDER),
                BackgroundColor(bevy_presentation::THEME_SURFACE),
                children![(
                    Text::new(label),
                    bevy_presentation::TextContent,
                    TextFont::from_font_size(bevy_presentation::muted_font_size()),
                    TextColor(bevy_presentation::THEME_TEXT),
                )],
            ));
            if options.tab == index {
                tab.insert(UiSelected);
            }
        }
    });

    match options.tab {
        0 => render_game_data_tab(&mut commands, panel, &options),
        1 => render_display_tab(&mut commands, panel, &options),
        _ => {}
    }

    options.dirty = false;
}

fn render_game_data_tab(commands: &mut Commands, panel: Entity, options: &OptionsState) {
    bevy_presentation::spawn_label(commands, panel, "GAME DATA");

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

    bevy_presentation::spawn_muted_label(commands, panel, root_label);

    #[cfg(not(target_os = "android"))]
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "The configured desktop root is stored separately from the game data itself and takes effect on the next launch.",
    );

    #[cfg(target_os = "android")]
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "Android keeps the Rust working root app-scoped and mirrors editable mods and saves to this selected folder.",
    );

    if let Some(message) = &options.message {
        bevy_presentation::spawn_context_message(commands, panel, message);
    }

    let change = bevy_presentation::spawn_action_button(
        commands,
        panel,
        0,
        "⌂",
        "Change game data folder",
    );
    if options.selected == 0 {
        commands.entity(change).insert(UiSelected);
    }

    let back = bevy_presentation::spawn_action_button(commands, panel, 1, "←", "Back");
    if options.selected == 1 {
        commands.entity(back).insert(UiSelected);
    }
}

fn render_display_tab(commands: &mut Commands, panel: Entity, options: &OptionsState) {
    bevy_presentation::spawn_label(commands, panel, "DISPLAY");

    #[cfg(not(target_os = "android"))]
    {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "Choose how the desktop game window is displayed. Windowed presets remain freely resizable.",
        );

        let modes = [
            (desktop_display::DisplayMode::Fullscreen, "Fullscreen"),
            (
                desktop_display::DisplayMode::Windowed1920x1080,
                "Windowed — 1920 × 1080",
            ),
            (
                desktop_display::DisplayMode::Windowed1280x720,
                "Windowed — 1280 × 720",
            ),
        ];

        for (index, (mode, label)) in modes.into_iter().enumerate() {
            let button = bevy_presentation::spawn_action_button(
                commands,
                panel,
                index,
                "▣",
                if options.display_mode == mode {
                    format!("{label} (current)")
                } else {
                    label.to_string()
                },
            );
            if options.selected == index {
                commands.entity(button).insert(UiSelected);
            }
        }

        let back = bevy_presentation::spawn_action_button(commands, panel, 3, "←", "Back");
        if options.selected == 3 {
            commands.entity(back).insert(UiSelected);
        }
    }

    #[cfg(target_os = "android")]
    {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "Display configuration is not available on Android yet.",
        );
        let back = bevy_presentation::spawn_action_button(commands, panel, 0, "←", "Back");
        if options.selected == 0 {
            commands.entity(back).insert(UiSelected);
        }
    }

    if let Some(message) = &options.message {
        bevy_presentation::spawn_context_message(commands, panel, message);
    }
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
        let gameplay = GameplayState::default();
        let mut options = OptionsState {
            active: true,
            ..OptionsState::default()
        };

        close_options(&mut lifecycle, &mut navigation, &mut options, &gameplay);

        assert_eq!(navigation.current_screen, Some(ScreenId::Lifecycle));
        assert_eq!(navigation.selected, 1);
        assert_eq!(lifecycle.selected, 1);
        assert!(!options.active);
    }

    #[test]
    fn closing_options_from_pause_restores_pause_selection() {
        let mut lifecycle = LifecycleState::default();
        let mut navigation = NavigationState {
            current_screen: Some(ScreenId::Options),
            return_screen: Some(ScreenId::Gameplay),
            selected: 0,
        };
        let mut gameplay = GameplayState::default();
        gameplay.screen = crate::bevy_gameplay::GameplayScreen::Pause;
        gameplay.selected = 1;
        gameplay.pause_selected = 3;
        let mut options = OptionsState {
            active: true,
            ..OptionsState::default()
        };

        close_options(&mut lifecycle, &mut navigation, &mut options, &gameplay);

        assert_eq!(navigation.current_screen, Some(ScreenId::Gameplay));
        assert_eq!(navigation.selected, 3);
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
