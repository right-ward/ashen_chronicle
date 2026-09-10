//! Bevy presentation and navigation for non-combat gameplay records.
//!
//! Character, inventory, quest, meditation, history, and journal screens reuse
//! the existing frontend-neutral view models and authoritative game mutations.

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::bevy_lifecycle::{GameSession, LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::game::{actions, character, history_screen, records};
use crate::input::InputEvent;
use crate::presentation::{
    CharacterSheetView, HistoryEntryView, HistoryView, InventoryDetailView, InventoryView,
    QuestLogView, QuestView,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordScreen {
    CharacterGeneral,
    CharacterReputation,
    CharacterJournal,
    Inventory,
    InventoryDetail,
    QuestLog,
    QuestDetail,
    Meditation,
    MeditationResult,
    History,
    HistoryDetail,
    JournalEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JournalParent {
    Gameplay,
    Character,
}

#[derive(Resource)]
pub(crate) struct RecordScreenState {
    screen: RecordScreen,
    active_navigation: Option<ScreenId>,
    selected: usize,
    detail_index: usize,
    message: Option<String>,
    meditation_result: Option<crate::presentation::MeditationResultView>,
    journal_draft: String,
    journal_parent: JournalParent,
    dirty: bool,
}

impl Default for RecordScreenState {
    fn default() -> Self {
        Self {
            screen: RecordScreen::CharacterGeneral,
            active_navigation: None,
            selected: 0,
            detail_index: 0,
            message: None,
            meditation_result: None,
            journal_draft: String::new(),
            journal_parent: JournalParent::Gameplay,
            dirty: true,
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<RecordScreenState>().add_systems(
        Update,
        (sync_screen, text_input, record_input, render_if_active).chain(),
    );
}

fn is_record_screen(screen: Option<ScreenId>) -> bool {
    matches!(
        screen,
        Some(
            ScreenId::Character
                | ScreenId::Inventory
                | ScreenId::Quests
                | ScreenId::Meditation
                | ScreenId::History
                | ScreenId::Journal
        )
    )
}

fn sync_screen(
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut records_state: ResMut<RecordScreenState>,
) {
    if lifecycle.phase != LifecyclePhase::Complete || !is_record_screen(navigation.current_screen) {
        records_state.active_navigation = None;
        return;
    }

    let Some(screen) = navigation.current_screen else {
        return;
    };
    if records_state.active_navigation == Some(screen) {
        return;
    }

    records_state.active_navigation = Some(screen);
    records_state.screen = match screen {
        ScreenId::Character => RecordScreen::CharacterGeneral,
        ScreenId::Inventory => RecordScreen::Inventory,
        ScreenId::Quests => RecordScreen::QuestLog,
        ScreenId::Meditation => RecordScreen::Meditation,
        ScreenId::History => RecordScreen::History,
        ScreenId::Journal => RecordScreen::JournalEntry,
        _ => RecordScreen::CharacterGeneral,
    };
    records_state.selected = 0;
    records_state.detail_index = 0;
    records_state.message = None;
    records_state.meditation_result = None;
    records_state.journal_draft.clear();
    records_state.journal_parent = JournalParent::Gameplay;
    records_state.dirty = true;
}

fn text_input(
    mut keyboard: MessageReader<KeyboardInput>,
    navigation: Res<NavigationState>,
    mut records_state: ResMut<RecordScreenState>,
) {
    if !is_record_screen(navigation.current_screen)
        || records_state.screen != RecordScreen::JournalEntry
        || records_state.message.is_some()
    {
        return;
    }

    for event in keyboard.read() {
        if let Some(text) = &event.text {
            if !text.chars().any(char::is_control) {
                records_state.journal_draft.push_str(text);
                records_state.dirty = true;
            }
        }
    }
}

fn record_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut records_state: ResMut<RecordScreenState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || !is_record_screen(navigation.current_screen)
        || input_queue.0.is_empty()
    {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up => move_selection(&mut records_state, &lifecycle, -1),
            InputEvent::Down => move_selection(&mut records_state, &lifecycle, 1),
            InputEvent::Home => {
                records_state.selected = 0;
                records_state.dirty = true;
            }
            InputEvent::End => move_to_end(&mut records_state, &lifecycle),
            InputEvent::PageUp => move_selection(&mut records_state, &lifecycle, -5),
            InputEvent::PageDown => move_selection(&mut records_state, &lifecycle, 5),
            InputEvent::Backspace => {
                if records_state.screen == RecordScreen::JournalEntry
                    && records_state.message.is_none()
                {
                    records_state.journal_draft.pop();
                    records_state.dirty = true;
                }
            }
            InputEvent::Confirm => activate(&mut lifecycle, &mut navigation, &mut records_state),
            InputEvent::Cancel => cancel(&mut navigation, &mut records_state),
            _ => {}
        }
    }
}

fn selection_count(records_state: &RecordScreenState, lifecycle: &LifecycleState) -> usize {
    let Some(session) = lifecycle.session.as_ref() else {
        return 1;
    };
    match records_state.screen {
        RecordScreen::CharacterGeneral => 3,
        RecordScreen::CharacterReputation => 1,
        RecordScreen::CharacterJournal => {
            character::build_character_sheet_view(&session.state)
                .notes
                .len()
                + 2
        }
        RecordScreen::Inventory => records::build_inventory_view(&session.state).items.len() + 1,
        RecordScreen::InventoryDetail => 1,
        RecordScreen::QuestLog => records::build_quest_log_view(&session.state).quests.len() + 1,
        RecordScreen::QuestDetail => 1,
        RecordScreen::Meditation => {
            let view = actions::build_meditation_view(&session.state);
            if view.safe_to_meditate {
                view.targets.len() + 1
            } else {
                1
            }
        }
        RecordScreen::MeditationResult => 1,
        RecordScreen::History => history_screen::build_view(&session.state).entries.len() + 1,
        RecordScreen::HistoryDetail => 1,
        RecordScreen::JournalEntry => 2,
    }
}

fn move_selection(
    records_state: &mut RecordScreenState,
    lifecycle: &LifecycleState,
    direction: isize,
) {
    let count = selection_count(records_state, lifecycle);
    if count == 0 {
        return;
    }
    records_state.selected =
        (records_state.selected as isize + direction).rem_euclid(count as isize) as usize;
    records_state.dirty = true;
}

fn move_to_end(records_state: &mut RecordScreenState, lifecycle: &LifecycleState) {
    let count = selection_count(records_state, lifecycle);
    if count > 0 {
        records_state.selected = count - 1;
        records_state.dirty = true;
    }
}

fn close_to_gameplay(navigation: &mut NavigationState) {
    navigation.current_screen = Some(ScreenId::Gameplay);
    navigation.return_screen = None;
    navigation.selected = 0;
}

fn activate(
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    records_state: &mut RecordScreenState,
) {
    let Some(session) = lifecycle.session.as_mut() else {
        return;
    };

    match records_state.screen {
        RecordScreen::CharacterGeneral => match records_state.selected {
            0 => {
                records_state.screen = RecordScreen::CharacterReputation;
                records_state.selected = 0;
                records_state.dirty = true;
            }
            1 => {
                records_state.screen = RecordScreen::CharacterJournal;
                records_state.selected = 0;
                records_state.dirty = true;
            }
            _ => close_to_gameplay(navigation),
        },
        RecordScreen::CharacterReputation => {
            records_state.screen = RecordScreen::CharacterGeneral;
            records_state.selected = 0;
            records_state.dirty = true;
        }
        RecordScreen::CharacterJournal => {
            let note_count = character::build_character_sheet_view(&session.state)
                .notes
                .len();
            if records_state.selected < note_count {
                return;
            }
            if records_state.selected == note_count {
                records_state.journal_parent = JournalParent::Character;
                records_state.screen = RecordScreen::JournalEntry;
                records_state.selected = 0;
                records_state.message = None;
                records_state.journal_draft.clear();
                records_state.dirty = true;
            } else {
                records_state.screen = RecordScreen::CharacterGeneral;
                records_state.selected = 0;
                records_state.dirty = true;
            }
        }
        RecordScreen::Inventory => {
            let view = records::build_inventory_view(&session.state);
            if records_state.selected >= view.items.len() {
                close_to_gameplay(navigation);
            } else {
                records_state.detail_index = records_state.selected;
                records_state.screen = RecordScreen::InventoryDetail;
                records_state.selected = 0;
                records_state.dirty = true;
            }
        }
        RecordScreen::InventoryDetail => {
            records_state.screen = RecordScreen::Inventory;
            records_state.selected = records_state.detail_index;
            records_state.dirty = true;
        }
        RecordScreen::QuestLog => {
            let view = records::build_quest_log_view(&session.state);
            if records_state.selected >= view.quests.len() {
                close_to_gameplay(navigation);
            } else {
                records_state.detail_index = records_state.selected;
                records_state.screen = RecordScreen::QuestDetail;
                records_state.selected = 0;
                records_state.dirty = true;
            }
        }
        RecordScreen::QuestDetail => {
            records_state.screen = RecordScreen::QuestLog;
            records_state.selected = records_state.detail_index;
            records_state.dirty = true;
        }
        RecordScreen::Meditation => {
            let view = actions::build_meditation_view(&session.state);
            if !view.safe_to_meditate {
                close_to_gameplay(navigation);
            } else if records_state.selected >= view.targets.len() {
                close_to_gameplay(navigation);
            } else {
                match actions::meditate_to_target(
                    &mut session.state,
                    &session.save_path,
                    records_state.selected,
                ) {
                    Ok(result) => {
                        records_state.meditation_result = Some(result);
                        records_state.screen = RecordScreen::MeditationResult;
                        records_state.selected = 0;
                        records_state.message = None;
                        records_state.dirty = true;
                    }
                    Err(err) => {
                        records_state.message = Some(err.to_string());
                        records_state.dirty = true;
                    }
                }
            }
        }
        RecordScreen::MeditationResult => close_to_gameplay(navigation),
        RecordScreen::History => {
            let view = history_screen::build_view(&session.state);
            if records_state.selected >= view.entries.len() {
                close_to_gameplay(navigation);
            } else {
                records_state.detail_index = records_state.selected;
                records_state.screen = RecordScreen::HistoryDetail;
                records_state.selected = 0;
                records_state.dirty = true;
            }
        }
        RecordScreen::HistoryDetail => {
            records_state.screen = RecordScreen::History;
            records_state.selected = records_state.detail_index;
            records_state.dirty = true;
        }
        RecordScreen::JournalEntry => {
            if records_state.message.is_some() {
                if records_state.journal_parent == JournalParent::Character {
                    records_state.screen = RecordScreen::CharacterJournal;
                    records_state.selected = 0;
                    records_state.message = None;
                    records_state.dirty = true;
                } else {
                    close_to_gameplay(navigation);
                }
                return;
            }
            match records_state.selected {
                0 => {
                    if records::record_journal_note(
                        &mut session.state,
                        &records_state.journal_draft,
                    ) {
                        records_state.message = Some("The journal entry is recorded.".to_string());
                    } else {
                        records_state.message =
                            Some("Write a note before recording it.".to_string());
                    }
                    records_state.dirty = true;
                }
                _ => {
                    if records_state.journal_parent == JournalParent::Character {
                        records_state.screen = RecordScreen::CharacterJournal;
                        records_state.selected = 0;
                        records_state.journal_draft.clear();
                        records_state.dirty = true;
                    } else {
                        close_to_gameplay(navigation);
                    }
                }
            }
        }
    }
}

fn cancel(navigation: &mut NavigationState, records_state: &mut RecordScreenState) {
    match records_state.screen {
        RecordScreen::CharacterGeneral
        | RecordScreen::Inventory
        | RecordScreen::QuestLog
        | RecordScreen::Meditation
        | RecordScreen::History
        | RecordScreen::MeditationResult => close_to_gameplay(navigation),
        RecordScreen::CharacterReputation | RecordScreen::CharacterJournal => {
            records_state.screen = RecordScreen::CharacterGeneral;
            records_state.selected = 0;
            records_state.message = None;
            records_state.dirty = true;
        }
        RecordScreen::InventoryDetail => {
            records_state.screen = RecordScreen::Inventory;
            records_state.selected = records_state.detail_index;
            records_state.dirty = true;
        }
        RecordScreen::QuestDetail => {
            records_state.screen = RecordScreen::QuestLog;
            records_state.selected = records_state.detail_index;
            records_state.dirty = true;
        }
        RecordScreen::HistoryDetail => {
            records_state.screen = RecordScreen::History;
            records_state.selected = records_state.detail_index;
            records_state.dirty = true;
        }
        RecordScreen::JournalEntry => {
            if records_state.journal_parent == JournalParent::Character {
                records_state.screen = RecordScreen::CharacterJournal;
                records_state.selected = 0;
                records_state.message = None;
                records_state.dirty = true;
            } else {
                close_to_gameplay(navigation);
            }
        }
    }
}

fn render_if_active(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut records_state: ResMut<RecordScreenState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete || !is_record_screen(navigation.current_screen) {
        return;
    }
    if !records_state.dirty {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }

    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    match records_state.screen {
        RecordScreen::CharacterGeneral => render_character_general(
            &mut commands,
            &character::build_character_sheet_view(&session.state),
            records_state.selected,
        ),
        RecordScreen::CharacterReputation => render_character_reputation(
            &mut commands,
            &character::build_character_sheet_view(&session.state),
        ),
        RecordScreen::CharacterJournal => render_character_journal(
            &mut commands,
            &character::build_character_sheet_view(&session.state),
            records_state.selected,
        ),
        RecordScreen::Inventory => render_inventory(
            &mut commands,
            &records::build_inventory_view(&session.state),
            records_state.selected,
        ),
        RecordScreen::InventoryDetail => render_inventory_detail(
            &mut commands,
            &records::build_inventory_detail_view(&session.state, records_state.detail_index),
        ),
        RecordScreen::QuestLog => render_quests(
            &mut commands,
            &records::build_quest_log_view(&session.state),
            records_state.selected,
        ),
        RecordScreen::QuestDetail => render_quest_detail(
            &mut commands,
            records::build_quest_view_for(&session.state, records_state.detail_index).as_ref(),
        ),
        RecordScreen::Meditation => render_meditation(
            &mut commands,
            &actions::build_meditation_view(&session.state),
            records_state.selected,
            records_state.message.as_deref(),
        ),
        RecordScreen::MeditationResult => {
            render_meditation_result(&mut commands, records_state.meditation_result.as_ref())
        }
        RecordScreen::History => render_history(
            &mut commands,
            &history_screen::build_view(&session.state),
            records_state.selected,
        ),
        RecordScreen::HistoryDetail => render_history_detail(
            &mut commands,
            history_screen::build_view(&session.state)
                .entries
                .get(records_state.detail_index),
        ),
        RecordScreen::JournalEntry => render_journal_entry(
            &mut commands,
            &session.state,
            &records_state.journal_draft,
            records_state.selected,
            records_state.message.as_deref(),
            records_state.journal_parent,
        ),
    }
    records_state.dirty = false;
}

fn render_header(commands: &mut Commands, title: &str, subtitle: impl Into<String>) -> Entity {
    let root = bevy_presentation::spawn_screen(commands, title);
    let panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_muted_label(commands, panel, subtitle);
    panel
}

fn render_character_general(commands: &mut Commands, view: &CharacterSheetView, selected: usize) {
    let panel = render_header(
        commands,
        "CHARACTER",
        format!("{} · Level {}", view.character.display_name(), view.level),
    );
    bevy_presentation::spawn_label(
        commands,
        panel,
        format!(
            "Health: {}/{} · Experience: {}/{}",
            view.character.hp, view.character.max_hp, view.experience, view.next_level_experience
        ),
    );
    bevy_presentation::spawn_label(
        commands,
        panel,
        format!(
            "Might {} (effective {}) · Insight {} (effective {}) · Endurance {} (effective {})",
            view.attributes.might,
            view.attributes.effective_might,
            view.attributes.insight,
            view.attributes.effective_insight,
            view.attributes.endurance,
            view.attributes.effective_endurance,
        ),
    );
    if view.conditions.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "Conditions: none");
    } else {
        bevy_presentation::spawn_label(commands, panel, "Conditions");
        for condition in &view.conditions {
            let effect = match (condition.penalty, condition.bonus) {
                (penalty, bonus) if penalty < 0 && bonus > 0 => {
                    format!(" {:+} penalty, {:+} bonus", penalty, bonus)
                }
                (penalty, _) if penalty < 0 => format!(" {:+} penalty", penalty),
                (_, bonus) if bonus > 0 => format!(" {:+} bonus", bonus),
                _ => String::new(),
            };
            bevy_presentation::spawn_muted_label(
                commands,
                panel,
                format!(
                    "{} · {} portions{}",
                    condition.name, condition.remaining, effect
                ),
            );
        }
    }
    for (index, label) in ["Reputation", "Journal", "Back"].into_iter().enumerate() {
        let marker = if index == selected { "▶ " } else { "" };
        bevy_presentation::spawn_choice_button(commands, panel, index, format!("{marker}{label}"));
    }
}

fn render_character_reputation(commands: &mut Commands, view: &CharacterSheetView) {
    let panel = render_header(
        commands,
        "CHARACTER · REPUTATION",
        "Faction standing and remembered dealings.",
    );
    if view.factions.is_empty() {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "No faction reputations have been recorded.",
        );
    } else {
        for faction in &view.factions {
            bevy_presentation::spawn_label(
                commands,
                panel,
                format!("{} {:+}", faction.name, faction.reputation),
            );
            for memory in faction.memories.iter().rev() {
                bevy_presentation::spawn_muted_label(commands, panel, format!("  · {memory}"));
            }
        }
    }
    bevy_presentation::spawn_choice_button(commands, panel, 0, "Back to character");
}

fn render_character_journal(commands: &mut Commands, view: &CharacterSheetView, selected: usize) {
    let panel = render_header(commands, "CHARACTER · JOURNAL", "Recorded personal notes.");
    if view.notes.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "The journal is empty.");
    } else {
        for (index, note) in view.notes.iter().enumerate() {
            let marker = if selected == index { "▶ " } else { "" };
            bevy_presentation::spawn_muted_label(
                commands,
                panel,
                format!("{marker}{}. {}", index + 1, note),
            );
        }
    }
    let write_index = view.notes.len();
    let back_index = write_index + 1;
    let marker = if selected == write_index { "▶ " } else { "" };
    bevy_presentation::spawn_choice_button(
        commands,
        panel,
        write_index,
        format!("{marker}Write new note"),
    );
    let marker = if selected == back_index { "▶ " } else { "" };
    bevy_presentation::spawn_choice_button(commands, panel, back_index, format!("{marker}Back"));
}

fn render_inventory(commands: &mut Commands, view: &InventoryView, selected: usize) {
    let panel = render_header(commands, "INVENTORY", view.character.display_name());
    if view.items.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "Your pack is empty.");
    } else {
        for (index, item) in view.items.iter().enumerate() {
            let marker = if selected == index { "▶ " } else { "" };
            bevy_presentation::spawn_choice_button(
                commands,
                panel,
                index,
                format!("{marker}{}", item.name),
            );
        }
    }
    let back_index = view.items.len();
    let marker = if selected == back_index { "▶ " } else { "" };
    bevy_presentation::spawn_choice_button(commands, panel, back_index, format!("{marker}Back"));
}

fn render_inventory_detail(commands: &mut Commands, view: &Option<InventoryDetailView>) {
    let panel = render_header(commands, "INVENTORY · ITEM", "Inspect the selected item.");
    let Some(view) = view else {
        bevy_presentation::spawn_muted_label(commands, panel, "That item is no longer available.");
        bevy_presentation::spawn_choice_button(commands, panel, 0, "Back");
        return;
    };
    bevy_presentation::spawn_label(commands, panel, &view.item.name);
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        format!("Item {} of {}", view.position, view.total),
    );
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        if view.item.description.trim().is_empty() {
            "No description is available.".to_string()
        } else {
            view.item.description.clone()
        },
    );
    if view.art.is_some() {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "Item art is available in the campaign content.",
        );
    }
    bevy_presentation::spawn_choice_button(commands, panel, 0, "Back to inventory");
}

fn render_quests(commands: &mut Commands, view: &QuestLogView, selected: usize) {
    let panel = render_header(commands, "QUEST LOG", view.character.display_name());
    if view.quests.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "No quests have been recorded yet.");
    } else {
        for (index, quest) in view.quests.iter().enumerate() {
            let marker = if selected == index { "▶ " } else { "" };
            bevy_presentation::spawn_choice_button(
                commands,
                panel,
                index,
                format!("{marker}[{}] {}", quest.status, quest.title),
            );
        }
    }
    let back_index = view.quests.len();
    let marker = if selected == back_index { "▶ " } else { "" };
    bevy_presentation::spawn_choice_button(commands, panel, back_index, format!("{marker}Back"));
}

fn render_quest_detail(commands: &mut Commands, view: Option<&QuestView>) {
    let panel = render_header(
        commands,
        "QUEST · DETAILS",
        "Objective progress and reward.",
    );
    let Some(view) = view else {
        bevy_presentation::spawn_muted_label(commands, panel, "That quest is no longer available.");
        bevy_presentation::spawn_choice_button(commands, panel, 0, "Back");
        return;
    };
    bevy_presentation::spawn_label(commands, panel, &view.title);
    bevy_presentation::spawn_muted_label(commands, panel, format!("Status: {}", view.status));
    if !view.description.trim().is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, &view.description);
    }
    bevy_presentation::spawn_label(commands, panel, "Objectives");
    if view.objectives.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "No objectives recorded.");
    } else {
        for objective in &view.objectives {
            let marker = if objective.completed { "x" } else { " " };
            bevy_presentation::spawn_muted_label(
                commands,
                panel,
                format!(
                    "[{marker}] {} ({}/{})",
                    objective.label, objective.progress, objective.required
                ),
            );
        }
    }
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        format!(
            "Reward: {}",
            view.reward_item_name.as_deref().unwrap_or("—")
        ),
    );
    bevy_presentation::spawn_choice_button(commands, panel, 0, "Back to quest log");
}

fn render_meditation(
    commands: &mut Commands,
    view: &crate::presentation::MeditationView,
    selected: usize,
    message: Option<&str>,
) {
    let panel = render_header(commands, "MEDITATION", view.current_time.clone());
    if let Some(message) = message.or(view.unavailable_message.as_deref()) {
        bevy_presentation::spawn_muted_label(commands, panel, message);
    }
    if !view.safe_to_meditate {
        bevy_presentation::spawn_choice_button(commands, panel, 0, "Back");
        return;
    }
    bevy_presentation::spawn_muted_label(commands, panel, "Choose when to end your meditation.");
    for (index, target) in view.targets.iter().enumerate() {
        let marker = if selected == index { "▶ " } else { "" };
        bevy_presentation::spawn_choice_button(
            commands,
            panel,
            index,
            format!("{marker}{}", target.label),
        );
    }
    let back = view.targets.len();
    let marker = if selected == back { "▶ " } else { "" };
    bevy_presentation::spawn_choice_button(commands, panel, back, format!("{marker}Cancel"));
}

fn render_meditation_result(
    commands: &mut Commands,
    result: Option<&crate::presentation::MeditationResultView>,
) {
    let panel = render_header(
        commands,
        "MEDITATION · COMPLETE",
        "The result of your rest.",
    );
    let Some(result) = result else {
        bevy_presentation::spawn_muted_label(commands, panel, "No meditation result is available.");
        bevy_presentation::spawn_choice_button(commands, panel, 0, "Back");
        return;
    };
    bevy_presentation::spawn_label(commands, panel, &result.ending_time);
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        format!("Time meditated: {} portion(s)", result.portions),
    );
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        format!("HP recovered: {}", result.hp_recovered),
    );
    if result.exhausted_removed {
        bevy_presentation::spawn_muted_label(commands, panel, "Exhausted is removed.");
    }
    if result.well_rested_applied {
        bevy_presentation::spawn_muted_label(commands, panel, "Well-rested is applied.");
    }
    bevy_presentation::spawn_choice_button(commands, panel, 0, "Back to gameplay");
}

fn render_history(commands: &mut Commands, view: &HistoryView, selected: usize) {
    let panel = render_header(
        commands,
        "WORLD HISTORY",
        format!("{} · {}", view.world_name, view.character.display_name()),
    );
    bevy_presentation::spawn_muted_label(commands, panel, &view.time);
    if view.entries.is_empty() {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "The world has not recorded any history yet.",
        );
    } else {
        for (index, entry) in view.entries.iter().enumerate() {
            let marker = if selected == index { "▶ " } else { "" };
            bevy_presentation::spawn_choice_button(
                commands,
                panel,
                index,
                format!(
                    "{marker}Day {} {} {}",
                    entry.day,
                    entry_marker(entry),
                    entry.text
                ),
            );
        }
    }
    let back_index = view.entries.len();
    let marker = if selected == back_index { "▶ " } else { "" };
    bevy_presentation::spawn_choice_button(commands, panel, back_index, format!("{marker}Back"));
}

fn render_history_detail(commands: &mut Commands, entry: Option<&HistoryEntryView>) {
    let panel = render_header(commands, "HISTORY · DETAILS", "Chronicle entry.");
    let Some(entry) = entry else {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "That history entry is no longer available.",
        );
        bevy_presentation::spawn_choice_button(commands, panel, 0, "Back");
        return;
    };
    bevy_presentation::spawn_label(commands, panel, format!("Day {}", entry.day));
    bevy_presentation::spawn_muted_label(commands, panel, &entry.text);
    if let Some(event_id) = &entry.event_id {
        bevy_presentation::spawn_muted_label(commands, panel, format!("Event: {event_id}"));
    }
    if let Some(location) = &entry.location_name {
        bevy_presentation::spawn_muted_label(commands, panel, format!("Location: {location}"));
    }
    if let Some(outcome) = &entry.outcome {
        bevy_presentation::spawn_muted_label(commands, panel, format!("Outcome: {outcome}"));
    }
    bevy_presentation::spawn_choice_button(commands, panel, 0, "Back to history");
}

fn render_journal_entry(
    commands: &mut Commands,
    state: &crate::model::GameState,
    draft: &str,
    selected: usize,
    message: Option<&str>,
    parent: JournalParent,
) {
    let panel = render_header(
        commands,
        "JOURNAL ENTRY",
        match parent {
            JournalParent::Gameplay => state.character.display_name(),
            JournalParent::Character => "Character journal".to_string(),
        },
    );
    if let Some(message) = message {
        bevy_presentation::spawn_label(commands, panel, message);
        bevy_presentation::spawn_choice_button(commands, panel, 0, "Continue");
        return;
    }
    bevy_presentation::spawn_muted_label(commands, panel, "Type a note. Backspace edits it.");
    bevy_presentation::spawn_label(
        commands,
        panel,
        if draft.is_empty() {
            "_".to_string()
        } else {
            draft.to_string()
        },
    );
    for (index, label) in ["Record note", "Cancel"].into_iter().enumerate() {
        let marker = if selected == index { "▶ " } else { "" };
        bevy_presentation::spawn_choice_button(commands, panel, index, format!("{marker}{label}"));
    }
}

fn entry_marker(entry: &HistoryEntryView) -> &'static str {
    match entry.entry_type {
        crate::presentation::HistoryEntryViewType::Event => "[EVENT]",
        crate::presentation::HistoryEntryViewType::Narrative => "[NOTE]",
    }
}
