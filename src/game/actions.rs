use crate::game::{character, interactions, legacy};
use crate::model::{Condition, GameState};
use crate::persistence::save_game;
use crate::ui::{choose_from_list, narrate, pause, prompt};
use std::path::Path;

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}

#[derive(Clone, Copy)]
pub(crate) enum GameAction {
    Travel,
    InvestigateThreat,
    SearchRemains,
    Talk,
    Meditate,
    QuestLog,
    Inventory,
    Journal,
    TestDeath,
    Quit,
    CharacterSheet,
}

pub(crate) struct MenuEntry {
    pub(crate) label: String,
    pub(crate) action: GameAction,
}

pub(crate) fn build_main_menu(state: &GameState) -> Vec<MenuEntry> {
    let mut menu = vec![
        MenuEntry {
            label: "Travel".to_string(),
            action: GameAction::Travel,
        },
        MenuEntry {
            label: "Meditate".to_string(),
            action: GameAction::Meditate,
        },
        MenuEntry {
            label: "Character sheet".to_string(),
            action: GameAction::CharacterSheet,
        },
        MenuEntry {
            label: "View inventory".to_string(),
            action: GameAction::Inventory,
        },
        MenuEntry {
            label: "Quest log".to_string(),
            action: GameAction::QuestLog,
        },
        MenuEntry {
            label: "Write journal note".to_string(),
            action: GameAction::Journal,
        },
        MenuEntry {
            label: "Talk".to_string(),
            action: GameAction::Talk,
        },
        MenuEntry {
            label: "Quit".to_string(),
            action: GameAction::Quit,
        },
        MenuEntry {
            label: "Test the death flow".to_string(),
            action: GameAction::TestDeath,
        },
    ];
    if state.threat.active {
        menu.insert(
            6,
            MenuEntry {
                label: "Investigate".to_string(),
                action: GameAction::InvestigateThreat,
            },
        );
    }
    if has_unscavenged_remains_at_location(state) {
        let insert_at = if state.threat.active { 7 } else { 6 };
        menu.insert(
            insert_at,
            MenuEntry {
                label: "Search remains".to_string(),
                action: GameAction::SearchRemains,
            },
        );
    }
    menu
}

fn has_unscavenged_remains_at_location(state: &GameState) -> bool {
    let location_id = state.character.location_id;
    state
        .corpses
        .iter()
        .any(|corpse| corpse.location_id == location_id && !corpse.inventory.is_empty())
}

pub(crate) fn advance_time(state: &mut GameState, amount: u32) {
    let total = state.world.time_points + amount;
    state.world.day += total / 12;
    state.world.time_points = total % 12;
    for condition in &mut state.character.conditions {
        condition.remaining = condition.remaining.saturating_sub(amount);
    }
    state
        .character
        .conditions
        .retain(|condition| condition.remaining > 0);
    if amount > 0 && state.character.hp <= state.character.max_hp / 3 && state.character.alive {
        add_or_refresh_condition(
            &mut state.character.conditions,
            Condition::new("Wounded", 3, -1),
        );
    }
}

pub(crate) fn add_or_refresh_condition(conditions: &mut Vec<Condition>, condition: Condition) {
    if let Some(existing) = conditions
        .iter_mut()
        .find(|current| current.name == condition.name)
    {
        existing.remaining = existing.remaining.max(condition.remaining);
        existing.penalty = condition.penalty;
        existing.bonus = condition.bonus;
    } else {
        conditions.push(condition);
    }
}

fn remove_condition(conditions: &mut Vec<Condition>, name: &str) {
    conditions.retain(|condition| condition.name != name);
}

fn is_night(points: u32) -> bool {
    matches!(points % 12, 0 | 1 | 10 | 11)
}

pub(crate) fn gain_experience(state: &mut GameState, amount: u32) {
    character::gain_experience(state, amount);
}

pub(crate) fn character_sheet(state: &GameState) {
    character::character_sheet(state);
}

pub(crate) fn travel(state: &mut GameState) -> std::io::Result<()> {
    let current_location = match state.world.location_by_id(state.character.location_id) {
        Some(location) => location.clone(),
        None => {
            println!("You are lost in a location that no longer exists.");
            pause();
            return Ok(());
        }
    };
    let options: Vec<String> = current_location
        .exits
        .iter()
        .filter_map(|id| state.world.location_by_id(*id).map(|loc| loc.name.clone()))
        .collect();
    if options.is_empty() {
        println!("There is nowhere to travel.");
        pause();
        return Ok(());
    }
    if let Some(choice) = choose_from_list("Travel where?", &options, Some("Back"))? {
        if let Some(target_id) = current_location.exits.get(choice).copied() {
            advance_time(state, 2);
            if is_night(state.world.time_points) {
                add_or_refresh_condition(
                    &mut state.character.conditions,
                    Condition::new("Exhausted", 2, -1),
                );
            }
            state.character.turn += 1;
            state.character.location_id = target_id;
            state.threat.clear();
            state.last_announced_location_id = None;
            let location = state.world.location_by_id(target_id).cloned();
            let location_name = location
                .as_ref()
                .map(|loc| loc.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let character_name = state.character.display_name();
            state.world.record_history(
                state.character.turn,
                format!("{} traveled to {}.", character_name, location_name),
            );
            println!("You travel to {}.", location_name);
            let dangerous = location.as_ref().map(|loc| loc.dangerous).unwrap_or(false);
            let context = crate::events::EventContext::for_travel_arrival(
                &location_name,
                dangerous,
                is_night(state.world.time_points),
            );
            crate::events::trigger_event(state, &context);
            if let Some(location) = location {
                if location.dangerous {
                    state.threat.activate(
                        location.id,
                        format!("{} stirs", location.name),
                        "The air is tense. Something here is still awake.".to_string(),
                    );
                    narrate("This place is dangerous.");
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn meditate_and_save(state: &mut GameState, save_path: &Path) -> std::io::Result<()> {
    let location_is_dangerous = state
        .world
        .location_is_dangerous(state.character.location_id);
    if state.threat.active || location_is_dangerous {
        println!("Not safe enough to meditate here.");
        pause();
        return Ok(());
    }
    let input = prompt("How long will you meditate? [1-4 time portions] ")?;
    let portions = input
        .parse::<u32>()
        .ok()
        .filter(|value| (1..=4).contains(value))
        .unwrap_or(1);
    let healing = portions as i32 + state.character.effective_endurance();
    advance_time(state, portions);
    state.character.turn += 1;
    state.character.heal(healing);
    remove_condition(&mut state.character.conditions, "Exhausted");
    let mut rested = Condition::new("Well-rested", 3, 0);
    rested.bonus = 1;
    add_or_refresh_condition(&mut state.character.conditions, rested);
    let character_name = state.character.display_name();
    state.world.record_history(
        state.character.turn,
        format!(
            "{} meditated for {} time portions and recovered.",
            character_name, portions
        ),
    );
    save_game(save_path, state)?;
    narrate(&format!(
        "You meditate until your breathing steadies. You look at the sky...\n{}\nYou recover {} HP and save the game.",
        crate::game::time::time_display(state.world.time_points, state.world.day),
        healing
    ));
    Ok(())
}

pub(crate) fn search_remains(state: &mut GameState) -> std::io::Result<()> {
    legacy::search_remains(state)
}

pub(crate) fn show_inventory(state: &GameState) {
    println!("\nInventory for {}", state.character.display_name());
    if state.character.inventory.is_empty() {
        println!("  Nothing.");
    } else {
        for item in &state.character.inventory {
            println!("  - {}: {}", item.name, item.description);
        }
    }
    pause();
}

pub(crate) fn review_quests(state: &GameState) {
    println!();
    println!("Quest log for {}", state.character.display_name());
    let visible_quests: Vec<_> = state
        .quests
        .iter()
        .filter(|quest| quest.offered || quest.completed)
        .collect();
    if visible_quests.is_empty() {
        println!("  Nothing yet.");
        pause();
        return;
    }
    for quest in visible_quests {
        let status = if quest.completed {
            if quest.reward_claimed {
                "completed"
            } else {
                "completed, reward pending"
            }
        } else {
            "active"
        };
        println!("  - {} [{}]", quest.title, status);
        println!("    {}", quest.description);
    }
    pause();
}

pub(crate) fn write_note(state: &mut GameState) -> std::io::Result<()> {
    let note = prompt("Write a journal note: ")?;
    if !note.is_empty() {
        state.character.notes.push(note.clone());
        advance_time(state, 1);
        state.character.turn += 1;
        let character_name = state.character.display_name();
        state.world.record_history(
            state.character.turn,
            format!("{} noted: {}", character_name, note),
        );
        narrate("The journal entry is recorded.");
    }
    Ok(())
}

pub(crate) fn force_death(state: &mut GameState) {
    legacy::force_death(state);
}

pub(crate) fn mark_character_dead(state: &mut GameState, cause: String, location_name: &str) {
    legacy::mark_character_dead(state, cause, location_name);
}
