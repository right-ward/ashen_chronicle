//! Bevy lifecycle flow: start, load, character creation, quit, and death.
//!
//! This module owns only frontend orchestration. Game creation, loading,
//! validation, inheritance, and save-path rules remain in the existing model,
//! persistence, and game modules.

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use std::path::PathBuf;

use crate::bevy_presentation::{self, BevyScreenRoot, NavigationState, ScreenId, SemanticInputQueue};
use crate::game::validate_loaded_state;
use crate::input::InputEvent;
use crate::model::{create_inherited_state, create_new_state, GameState, WorldMode};
use crate::persistence::{character_save_path, find_save_files, legacy_save_path, load_game};
use crate::presentation::{DeathView, FactionView, ItemView, ScreenView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LifecyclePhase {
    Start,
    Load,
    CreateCharacter,
    QuitConfirm,
    Death,
    Complete,
}

#[derive(Resource)]
pub(crate) struct LifecycleState {
    pub(crate) phase: LifecyclePhase,
    pub(crate) save_files: Vec<PathBuf>,
    pub(crate) selected: usize,
    pub(crate) world_name: String,
    pub(crate) character_name: String,
    pub(crate) character_title: String,
    pub(crate) pending_state: Option<(GameState, PathBuf)>,
    pub(crate) session: Option<GameSession>,
    pub(crate) message: Option<String>,
    dirty: bool,
}

impl Default for LifecycleState {
    fn default() -> Self {
        Self {
            phase: LifecyclePhase::Start,
            save_files: Vec::new(),
            selected: 0,
            world_name: "The Ashen Crown".to_string(),
            character_name: String::new(),
            character_title: "Ash Walker".to_string(),
            pending_state: None,
            session: None,
            message: None,
            dirty: true,
        }
    }
}

#[derive(Resource)]
pub(crate) struct GameSession {
    pub(crate) state: GameState,
    pub(crate) save_path: PathBuf,
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<LifecycleState>()
        .add_systems(Startup, initialize)
        .add_systems(Update, (text_input, lifecycle_input, render_if_dirty).chain());
}

fn initialize(mut lifecycle: ResMut<LifecycleState>) {
    lifecycle.refresh_saves();
}

fn text_input(
    mut keyboard: MessageReader<KeyboardInput>,
    mut lifecycle: ResMut<LifecycleState>,
) {
    if lifecycle.phase != LifecyclePhase::CreateCharacter || lifecycle.selected > 2 {
        return;
    }

    for event in keyboard.read() {
        if let Some(text) = &event.text {
            if !text.chars().any(char::is_control) {
                lifecycle.active_field_mut().push_str(text);
                lifecycle.dirty = true;
            }
        }
    }
}

fn lifecycle_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut input_queue: ResMut<SemanticInputQueue>,
    mut commands: Commands,
) {
    if input_queue.0.is_empty() {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up => move_selection(&mut lifecycle, &mut navigation, -1),
            InputEvent::Down => move_selection(&mut lifecycle, &mut navigation, 1),
            InputEvent::Cancel => handle_cancel(&mut lifecycle, &mut navigation),
            InputEvent::Backspace => {
                if lifecycle.phase == LifecyclePhase::CreateCharacter && lifecycle.selected <= 2 {
                    lifecycle.active_field_mut().pop();
                    lifecycle.dirty = true;
                }
            }
            InputEvent::Confirm => {
                if let Some(exit) = activate_selection(&mut lifecycle, &mut navigation) {
                    if exit {
                        commands.write_message(AppExit::Success);
                        return;
                    }
                }
            }
            _ => {}
        }
    }
}

fn move_selection(
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    direction: isize,
) {
    let max = match lifecycle.phase {
        LifecyclePhase::Start => start_option_count(lifecycle).saturating_sub(1),
        LifecyclePhase::Load => lifecycle.save_files.len().saturating_sub(1),
        LifecyclePhase::CreateCharacter => 3,
        LifecyclePhase::QuitConfirm => 1,
        LifecyclePhase::Death => 2,
        LifecyclePhase::Complete => 0,
    };
    lifecycle.selected = (lifecycle.selected as isize + direction).clamp(0, max as isize) as usize;
    navigation.selected = lifecycle.selected;
    lifecycle.dirty = true;
}

fn activate_selection(
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
) -> Option<bool> {
    match lifecycle.phase {
        LifecyclePhase::Start => match lifecycle.selected {
            0 => {
                lifecycle.phase = LifecyclePhase::CreateCharacter;
                lifecycle.selected = 0;
                lifecycle.message = None;
                lifecycle.dirty = true;
            }
            1 if start_has_load(lifecycle) => {
                lifecycle.phase = LifecyclePhase::Load;
                lifecycle.selected = 0;
                lifecycle.refresh_saves();
            }
            _ => return Some(true),
        },
        LifecyclePhase::Load => {
            if let Some(path) = lifecycle.save_files.get(lifecycle.selected).cloned() {
                match load_game(&path) {
                    Ok(state) => {
                        let warnings = validate_loaded_state(&state);
                        let save_path = character_save_path(PathBuf::from(".").as_path(), &state.character.name);
                        lifecycle.pending_state = Some((state, save_path));
                        lifecycle.message = if warnings.is_empty() {
                            None
                        } else {
                            Some(format!(
                                "Save loaded with {} warning(s). Confirm to continue.",
                                warnings.len()
                            ))
                        };
                        if warnings.is_empty() {
                            lifecycle.finish_loading();
                        } else {
                            lifecycle.phase = LifecyclePhase::Complete;
                            lifecycle.selected = 0;
                        }
                        lifecycle.dirty = true;
                    }
                    Err(err) => {
                        lifecycle.message = Some(format!("Could not load save: {err}"));
                        lifecycle.dirty = true;
                    }
                }
            }
        }
        LifecyclePhase::CreateCharacter => match lifecycle.selected {
            0..=2 => lifecycle.selected = (lifecycle.selected + 1).min(3),
            3 => lifecycle.create_character(),
            _ => {}
        },
        LifecyclePhase::QuitConfirm => return Some(lifecycle.selected == 0),
        LifecyclePhase::Death => match lifecycle.selected {
            0 => {
                lifecycle.create_character();
                lifecycle.message = Some("A new world begins. The old world remains behind.".to_string());
            }
            1 => lifecycle.inherit_character(),
            2 => return Some(true),
            _ => {}
        },
        LifecyclePhase::Complete => {
            if lifecycle.pending_state.is_some() {
                lifecycle.finish_loading();
            } else {
                return Some(true);
            }
        }
    }
    navigation.current_screen = Some(ScreenId::Lifecycle);
    navigation.selected = lifecycle.selected;
    None
}

fn handle_cancel(lifecycle: &mut LifecycleState, navigation: &mut NavigationState) {
    match lifecycle.phase {
        LifecyclePhase::Start => lifecycle.phase = LifecyclePhase::QuitConfirm,
        LifecyclePhase::Load | LifecyclePhase::CreateCharacter => {
            lifecycle.phase = LifecyclePhase::Start;
            lifecycle.selected = 0;
            lifecycle.message = None;
        }
        LifecyclePhase::QuitConfirm | LifecyclePhase::Death => {
            lifecycle.phase = LifecyclePhase::Start;
            lifecycle.selected = 0;
            lifecycle.message = None;
        }
        LifecyclePhase::Complete => {}
    }
    navigation.current_screen = Some(ScreenId::Lifecycle);
    navigation.selected = lifecycle.selected;
    lifecycle.dirty = true;
}

fn start_option_count(lifecycle: &LifecycleState) -> usize {
    if start_has_load(lifecycle) { 3 } else { 2 }
}

fn start_has_load(lifecycle: &LifecycleState) -> bool {
    !lifecycle.save_files.is_empty()
}

impl LifecycleState {
    fn refresh_saves(&mut self) {
        let current_dir = PathBuf::from(".");
        self.save_files = find_save_files(&current_dir).unwrap_or_default();
        let legacy = legacy_save_path(&current_dir);
        if legacy.exists() && !self.save_files.iter().any(|path| path == &legacy) {
            self.save_files.push(legacy);
        }
        self.selected = 0;
        self.dirty = true;
    }

    fn active_field_mut(&mut self) -> &mut String {
        match self.selected {
            0 => &mut self.world_name,
            1 => &mut self.character_name,
            _ => &mut self.character_title,
        }
    }

    fn create_character(&mut self) {
        let world_name = if self.world_name.trim().is_empty() {
            "The Ashen Crown"
        } else {
            self.world_name.trim()
        };
        let character_name = if self.character_name.trim().is_empty() {
            "Wanderer"
        } else {
            self.character_name.trim()
        };
        let title = if self.character_title.trim().is_empty() {
            "Ash Walker"
        } else {
            self.character_title.trim()
        };
        let mut state = create_new_state(
            world_name,
            WorldMode::New,
            character_name.to_string(),
            title.to_string(),
        );
        crate::game::world::bootstrap_campaign_content(&mut state);
        let save_path = character_save_path(PathBuf::from(".").as_path(), character_name);
        self.session = Some(GameSession { state, save_path });
        self.pending_state = None;
        self.phase = LifecyclePhase::Complete;
        self.selected = 0;
        self.message = None;
        self.dirty = true;
    }

    fn finish_loading(&mut self) {
        if let Some((mut state, save_path)) = self.pending_state.take() {
            crate::game::world::bootstrap_campaign_content(&mut state);
            if state.character.alive {
                self.session = Some(GameSession { state, save_path });
                self.phase = LifecyclePhase::Complete;
            } else {
                self.session = Some(GameSession { state, save_path });
                self.phase = LifecyclePhase::Death;
            }
            self.selected = 0;
            self.dirty = true;
        }
    }

    fn inherit_character(&mut self) {
        let Some(session) = self.session.as_ref() else { return };
        let name = if self.character_name.trim().is_empty() {
            "Heir"
        } else {
            self.character_name.trim()
        };
        let title = if self.character_title.trim().is_empty() {
            "Ash Walker"
        } else {
            self.character_title.trim()
        };
        let state = create_inherited_state(&session.state, name.to_string(), title.to_string());
        let save_path = character_save_path(PathBuf::from(".").as_path(), name);
        self.session = Some(GameSession { state, save_path });
        self.phase = LifecyclePhase::Complete;
        self.selected = 0;
        self.message = None;
        self.dirty = true;
    }
}

fn render_if_dirty(
    mut commands: Commands,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    if !lifecycle.dirty {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    render(&mut commands, &lifecycle);
    navigation.current_screen = Some(ScreenId::Lifecycle);
    navigation.selected = lifecycle.selected;
    lifecycle.dirty = false;
}

fn render(commands: &mut Commands, lifecycle: &LifecycleState) {
    let root = bevy_presentation::spawn_screen(commands, "THE ASHEN CHRONICLE");
    let panel = bevy_presentation::spawn_panel(commands, root);

    match lifecycle.phase {
        LifecyclePhase::Start => render_start(commands, panel, lifecycle),
        LifecyclePhase::Load => render_load(commands, panel, lifecycle),
        LifecyclePhase::CreateCharacter => render_creation(commands, panel, lifecycle),
        LifecyclePhase::QuitConfirm => render_quit(commands, panel, lifecycle),
        LifecyclePhase::Death => render_death(commands, panel, lifecycle),
        LifecyclePhase::Complete => render_complete(commands, panel, lifecycle),
    }
}

fn render_start(commands: &mut Commands, panel: Entity, lifecycle: &LifecycleState) {
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "The road is quiet. Something is listening.",
    );
    if let Some(message) = &lifecycle.message {
        bevy_presentation::spawn_muted_label(commands, panel, message);
    }
    let labels = if start_has_load(lifecycle) {
        vec!["New Game", "Load Game", "Quit"]
    } else {
        vec!["New Game", "Quit"]
    };
    for (index, label) in labels.into_iter().enumerate() {
        bevy_presentation::spawn_choice_button(commands, panel, index, label);
    }
}

fn render_load(commands: &mut Commands, panel: Entity, lifecycle: &LifecycleState) {
    bevy_presentation::spawn_muted_label(commands, panel, "Choose a life to continue.");
    if let Some(message) = &lifecycle.message {
        bevy_presentation::spawn_muted_label(commands, panel, message);
    }
    for (index, path) in lifecycle.save_files.iter().enumerate() {
        let label = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown save");
        bevy_presentation::spawn_choice_button(commands, panel, index, label);
    }
    bevy_presentation::spawn_choice_button(commands, panel, lifecycle.save_files.len(), "Back");
}

fn render_creation(commands: &mut Commands, panel: Entity, lifecycle: &LifecycleState) {
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "Enter the names below. Enter advances to the next field.",
    );
    let fields = [
        ("World", &lifecycle.world_name),
        ("Character", &lifecycle.character_name),
        ("Title", &lifecycle.character_title),
    ];
    for (index, (label, value)) in fields.into_iter().enumerate() {
        let marker = if lifecycle.selected == index { ">" } else { " " };
        bevy_presentation::spawn_label(commands, panel, format!("{marker} {label}: {value}"));
    }
    bevy_presentation::spawn_choice_button(commands, panel, 3, "Begin Life");
}

fn render_quit(commands: &mut Commands, panel: Entity, lifecycle: &LifecycleState) {
    bevy_presentation::spawn_muted_label(commands, panel, "Leave the chronicle?");
    for (index, label) in ["Leave", "Stay"].into_iter().enumerate() {
        bevy_presentation::spawn_choice_button(commands, panel, index, label);
    }
    if lifecycle.message.is_some() {
        bevy_presentation::spawn_muted_label(commands, panel, "Escape returns to the start screen.");
    }
}

fn render_complete(commands: &mut Commands, panel: Entity, lifecycle: &LifecycleState) {
    let Some(session) = lifecycle.session.as_ref() else {
        bevy_presentation::spawn_muted_label(commands, panel, "No active life.");
        bevy_presentation::spawn_choice_button(commands, panel, 0, "Quit");
        return;
    };
    bevy_presentation::spawn_label(
        commands,
        panel,
        format!("{} the {} is ready.", session.state.character.name, session.state.character.title),
    );
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "The Bevy gameplay flow will take over this session in the next migration step.",
    );
    if let Some(message) = &lifecycle.message {
        bevy_presentation::spawn_muted_label(commands, panel, message);
    }
    bevy_presentation::spawn_choice_button(commands, panel, 0, "Quit");
}

fn render_death(commands: &mut Commands, panel: Entity, lifecycle: &LifecycleState) {
    let Some(session) = lifecycle.session.as_ref() else { return };
    let view = build_death_view(&session.state);
    bevy_presentation::spawn_label(commands, panel, view.screen.title);
    if let Some(subtitle) = view.screen.subtitle {
        bevy_presentation::spawn_muted_label(commands, panel, subtitle);
    }
    for line in view.screen.body {
        bevy_presentation::spawn_label(commands, panel, line);
    }
    bevy_presentation::spawn_muted_label(commands, panel, view.memory_note);
    for (index, label) in [
        "Create a new world",
        "Inherit this world with a new character",
        "Quit",
    ]
    .into_iter()
    .enumerate()
    {
        bevy_presentation::spawn_choice_button(commands, panel, index, label);
    }
}

fn build_death_view(state: &GameState) -> DeathView {
    let character = crate::presentation::CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    };
    let location_name = state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| location.name.clone())
        .unwrap_or_else(|| "an unknown place".to_string());
    let deeds = state
        .world
        .history
        .iter()
        .filter(|entry| {
            entry.text.contains(&character.display_name()) && entry.text.contains("completed ")
        })
        .map(|entry| entry.text.clone())
        .take(5)
        .collect::<Vec<_>>();
    let faction_standing = state
        .factions
        .iter()
        .map(|faction| FactionView {
            name: faction.name.clone(),
            reputation: faction.reputation,
            memories: Vec::new(),
        })
        .collect::<Vec<_>>();
    let dropped_items = state
        .corpses
        .last()
        .map(|corpse| {
            corpse
                .inventory
                .iter()
                .map(|item| ItemView {
                    id: item.id,
                    name: item.name.clone(),
                    description: item.description.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut body = vec![format!(
        "{} died at {} on turn {}.",
        character.display_name(), location_name, state.character.turn
    )];
    body.push(String::new());
    body.push("Deeds remembered:".to_string());
    if deeds.is_empty() {
        body.push("  None recorded.".to_string());
    } else {
        body.extend(deeds.iter().map(|deed| format!("  - {deed}")));
    }
    body.push(String::new());
    body.push("Faction standing at death:".to_string());
    if faction_standing.is_empty() {
        body.push("  None recorded.".to_string());
    } else {
        body.extend(
            faction_standing
                .iter()
                .map(|faction| format!("  - {} {:+}", faction.name, faction.reputation)),
        );
    }
    body.push(String::new());
    body.push("What remains on the body:".to_string());
    if dropped_items.is_empty() {
        body.push("  Nothing worth carrying.".to_string());
    } else {
        body.push(format!(
            "  {}",
            dropped_items.iter().map(|item| item.name.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }
    DeathView {
        screen: ScreenView {
            title: "DEATH".to_string(),
            subtitle: Some("The body is still. The world is not.".to_string()),
            art: None,
            body,
        },
        character,
        location_name,
        turn: state.character.turn,
        deeds,
        faction_standing,
        dropped_items,
        memory_note: "The next life will know none of this as memory. It can only be discovered.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_screen_has_expected_options_without_saves() {
        let state = LifecycleState::default();
        assert_eq!(start_option_count(&state), 2);
        assert!(!start_has_load(&state));
    }

    #[test]
    fn default_creation_values_match_legacy_flow() {
        let state = LifecycleState::default();
        assert_eq!(state.world_name, "The Ashen Crown");
        assert!(state.character_name.is_empty());
        assert_eq!(state.character_title, "Ash Walker");
    }
}
