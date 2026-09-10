//! Bevy presentation and interaction for the primary gameplay/world screen.
//!
//! This module adapts the existing gameplay state and world-navigation rules
//! into Bevy UI. It does not reimplement gameplay simulation.

use bevy::prelude::*;

use crate::bevy_lifecycle::{GameSession, LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::game::{actions, menu, navigation};
use crate::input::InputEvent;
use crate::presentation::{HistoryEntryViewType, NavigationView, WorldView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameplayScreen {
    Dashboard,
    Navigation,
}

#[derive(Resource)]
pub(crate) struct GameplayState {
    pub(crate) screen: GameplayScreen,
    pub(crate) selected: usize,
    pub(crate) message: Option<String>,
    dirty: bool,
}

impl Default for GameplayState {
    fn default() -> Self {
        Self {
            screen: GameplayScreen::Dashboard,
            selected: 0,
            message: None,
            dirty: true,
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<GameplayState>()
        .add_systems(Update, (gameplay_input, render_if_active).chain());
}

fn gameplay_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut gameplay: ResMut<GameplayState>,
    mut navigation_state: ResMut<NavigationState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if lifecycle.phase != LifecyclePhase::Complete || lifecycle.session.is_none() {
        return;
    }
    if input_queue.0.is_empty() {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up => move_selection(&mut gameplay, &mut navigation_state, -1, &lifecycle),
            InputEvent::Down => move_selection(&mut gameplay, &mut navigation_state, 1, &lifecycle),
            InputEvent::Cancel => {
                if gameplay.screen == GameplayScreen::Navigation {
                    gameplay.screen = GameplayScreen::Dashboard;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.dirty = true;
                } else {
                    lifecycle.phase = LifecyclePhase::QuitConfirm;
                    lifecycle.selected = 1;
                    lifecycle.dirty = true;
                }
                navigation_state.current_screen = Some(ScreenId::Gameplay);
                navigation_state.selected = gameplay.selected;
            }
            InputEvent::Confirm => {
                activate_selection(&mut lifecycle, &mut gameplay, &mut navigation_state);
            }
            _ => {}
        }
    }
}

fn menu_entries(session: &GameSession) -> Vec<menu::MenuEntry> {
    menu::build_main_menu(&session.state)
}

fn move_selection(
    gameplay: &mut GameplayState,
    navigation_state: &mut NavigationState,
    direction: isize,
    lifecycle: &LifecycleState,
) {
    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let count = match gameplay.screen {
        GameplayScreen::Dashboard => menu_entries(session).len(),
        GameplayScreen::Navigation => navigation::build_view(&session.state).destinations.len() + 1,
    };
    if count == 0 {
        return;
    }
    gameplay.selected =
        (gameplay.selected as isize + direction).rem_euclid(count as isize) as usize;
    navigation_state.current_screen = Some(ScreenId::Gameplay);
    navigation_state.selected = gameplay.selected;
    gameplay.dirty = true;
}

fn activate_selection(
    lifecycle: &mut LifecycleState,
    gameplay: &mut GameplayState,
    navigation_state: &mut NavigationState,
) {
    let Some(session) = lifecycle.session.as_mut() else {
        return;
    };

    match gameplay.screen {
        GameplayScreen::Dashboard => {
            let entries = menu_entries(session);
            let Some(entry) = entries.get(gameplay.selected) else {
                return;
            };
            match entry.action {
                menu::GameAction::Travel => {
                    gameplay.screen = GameplayScreen::Navigation;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.dirty = true;
                }
                menu::GameAction::Quit => {
                    lifecycle.phase = LifecyclePhase::QuitConfirm;
                    lifecycle.selected = 1;
                    lifecycle.dirty = true;
                }
                _ => {
                    gameplay.message = Some(format!(
                        "{} remains available from the gameplay flow and will receive its Bevy screen in a later migration step.",
                        entry.label
                    ));
                    gameplay.dirty = true;
                }
            }
        }
        GameplayScreen::Navigation => {
            let view = navigation::build_view(&session.state);
            if gameplay.selected >= view.destinations.len() {
                gameplay.screen = GameplayScreen::Dashboard;
                gameplay.selected = 0;
                gameplay.message = None;
                gameplay.dirty = true;
                return;
            }
            if let Some(destination) = view.destinations.get(gameplay.selected) {
                let old_turn = session.state.character.turn;
                let target_id = destination.id;
                if actions::travel_to(&mut session.state, target_id).is_ok() {
                    let mut message = session
                        .state
                        .world
                        .history
                        .iter()
                        .rev()
                        .find(|entry| entry.turn > old_turn)
                        .map(|entry| entry.text.clone());
                    if !session.state.character.alive {
                        lifecycle.phase = LifecyclePhase::Death;
                        lifecycle.selected = 0;
                        lifecycle.dirty = true;
                    }
                    gameplay.screen = GameplayScreen::Dashboard;
                    gameplay.selected = 0;
                    gameplay.message = message.take();
                    gameplay.dirty = true;
                }
            }
        }
    }
    navigation_state.current_screen = Some(ScreenId::Gameplay);
    navigation_state.selected = gameplay.selected;
}

fn render_if_active(
    mut commands: Commands,
    lifecycle: ResMut<LifecycleState>,
    mut gameplay: ResMut<GameplayState>,
    mut navigation_state: ResMut<NavigationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete || lifecycle.session.is_none() {
        return;
    }
    if !gameplay.dirty {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }

    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let view = build_world_view(&session.state);
    match gameplay.screen {
        GameplayScreen::Dashboard => render_dashboard(
            &mut commands,
            &view,
            &menu_entries(session),
            gameplay.selected,
            gameplay.message.as_deref(),
        ),
        GameplayScreen::Navigation => render_navigation(
            &mut commands,
            &navigation::build_view(&session.state),
            gameplay.selected,
        ),
    }
    navigation_state.current_screen = Some(ScreenId::Gameplay);
    navigation_state.selected = gameplay.selected;
    gameplay.dirty = false;
}

fn render_dashboard(
    commands: &mut Commands,
    view: &WorldView,
    actions_list: &[menu::MenuEntry],
    selected: usize,
    message: Option<&str>,
) {
    let root = bevy_presentation::spawn_screen(commands, "THE ASHEN CHRONICLE");
    let header = bevy_presentation::spawn_panel(commands, root);
    let location = view
        .location
        .as_ref()
        .map(|location| location.name.as_str())
        .unwrap_or("Unknown");
    bevy_presentation::spawn_label(
        commands,
        header,
        format!("{} · {}", view.character.display_name(), view.time),
    );
    bevy_presentation::spawn_muted_label(
        commands,
        header,
        format!("World: {} · Location: {}", view.world_name, location),
    );
    bevy_presentation::spawn_gauge(commands, header, view.character.hp, view.character.max_hp);

    let context = bevy_presentation::spawn_panel(commands, root);
    render_world_context(commands, context, view);

    if let Some(message) = message {
        bevy_presentation::spawn_muted_label(commands, context, message);
    }

    let history = bevy_presentation::spawn_panel(commands, root);
    render_history(commands, history, view);

    let choices = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_muted_label(
        commands,
        choices,
        "Choose an action. Arrow keys and Enter also work.",
    );
    for (index, entry) in actions_list.iter().enumerate() {
        let label = if index == selected {
            format!("▶ {}", entry.label)
        } else {
            entry.label.clone()
        };
        bevy_presentation::spawn_choice_button(commands, choices, index, label);
    }
}

fn render_navigation(commands: &mut Commands, view: &NavigationView, selected: usize) {
    let root = bevy_presentation::spawn_screen(commands, "WORLD NAVIGATION");
    let panel = bevy_presentation::spawn_panel(commands, root);
    if let Some(current) = &view.current_location {
        bevy_presentation::spawn_label(commands, panel, current.name.clone());
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            format!("Region: {}", current.region_name),
        );
        if current.dangerous {
            bevy_presentation::spawn_muted_label(
                commands,
                panel,
                "Danger: this location is unsafe.",
            );
        }
        if !current.description.trim().is_empty() {
            bevy_presentation::spawn_muted_label(commands, panel, current.description.clone());
        }
    } else {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "Your current location can no longer be resolved.",
        );
    }

    if view.destinations.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "No known routes lead onward.");
    } else {
        for (index, destination) in view.destinations.iter().enumerate() {
            let label = if index == selected {
                format!("▶ {}", destination.name)
            } else {
                destination.name.clone()
            };
            bevy_presentation::spawn_choice_button(commands, panel, index, label);
        }
    }
    let back_index = view.destinations.len();
    let back_label = if selected == back_index {
        "▶ Back"
    } else {
        "Back"
    };
    bevy_presentation::spawn_choice_button(commands, panel, back_index, back_label);
}

fn render_world_context(commands: &mut Commands, parent: Entity, view: &WorldView) {
    let Some(location) = &view.location else {
        bevy_presentation::spawn_muted_label(commands, parent, "You are lost in an unknown place.");
        return;
    };
    bevy_presentation::spawn_label(commands, parent, location.name.clone());
    bevy_presentation::spawn_muted_label(
        commands,
        parent,
        format!("Region: {}", location.region_name),
    );
    if !location.description.trim().is_empty() {
        bevy_presentation::spawn_muted_label(commands, parent, location.description.clone());
    }
    if location.dangerous {
        bevy_presentation::spawn_muted_label(commands, parent, "Danger: this location is unsafe.");
    }
    match &view.threat {
        Some(threat) => {
            bevy_presentation::spawn_label(commands, parent, format!("Threat: {}", threat.label));
            if !threat.description.trim().is_empty() {
                bevy_presentation::spawn_muted_label(commands, parent, threat.description.clone());
            }
        }
        None => bevy_presentation::spawn_muted_label(commands, parent, "Threat: none active."),
    }
}

fn render_history(commands: &mut Commands, parent: Entity, view: &WorldView) {
    bevy_presentation::spawn_label(commands, parent, "Recent Events");
    if view.history.is_empty() {
        bevy_presentation::spawn_muted_label(commands, parent, "Nothing has been recorded yet.");
        return;
    }
    for entry in &view.history {
        let marker = match entry.entry_type {
            HistoryEntryViewType::Event => "[EVENT]",
            HistoryEntryViewType::Narrative => "[NOTE]",
        };
        bevy_presentation::spawn_muted_label(
            commands,
            parent,
            format!("Day {} {} {}", entry.day, marker, entry.text),
        );
    }
}

fn build_world_view(state: &crate::model::GameState) -> WorldView {
    let location = state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| {
            let region_name = state
                .world
                .regions
                .iter()
                .find(|region| region.id == location.region_id)
                .map(|region| region.name.clone())
                .unwrap_or_else(|| "Unknown region".to_string());
            crate::presentation::LocationView {
                id: location.id,
                name: location.name.clone(),
                description: location.description.clone(),
                region_name,
                dangerous: location.dangerous,
            }
        });
    let threat = state
        .threat
        .active
        .then(|| crate::presentation::ThreatView {
            label: state.threat.label.clone(),
            description: state.threat.description.clone(),
        });
    let history = state
        .world
        .history
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|entry| crate::presentation::HistoryEntryView {
            day: entry.turn,
            entry_type: match entry.entry_type {
                crate::model::HistoryEntryType::Event => HistoryEntryViewType::Event,
                crate::model::HistoryEntryType::Narrative => HistoryEntryViewType::Narrative,
            },
            text: entry.text.clone(),
            event_id: entry.event_id.clone(),
            location_name: entry.location_name.clone(),
            outcome: entry.outcome.clone(),
        })
        .collect();
    WorldView {
        world_name: state.world.name.clone(),
        time: crate::game::time::time_display(state.world.time_points, state.world.day),
        character: crate::presentation::CharacterView {
            name: state.character.name.clone(),
            title: state.character.title.clone(),
            hp: state.character.hp,
            max_hp: state.character.max_hp,
        },
        location,
        threat,
        history,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{create_new_state, WorldMode};

    #[test]
    fn gameplay_starts_on_dashboard() {
        let state = GameplayState::default();
        assert_eq!(state.screen, GameplayScreen::Dashboard);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn world_view_contains_current_location() {
        let state = create_new_state(
            "Test World",
            WorldMode::New,
            "Ash".to_string(),
            "Wanderer".to_string(),
        );
        let view = build_world_view(&state);
        assert_eq!(view.world_name, "Test World");
        assert!(view.location.is_some());
    }
}
