use crate::game::character;
use crate::game::state_effects::{self, add_or_refresh_condition};
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
            state_effects::advance_time(state, 2);
            if state_effects::is_night(state.world.time_points) {
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
                state_effects::is_night(state.world.time_points),
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
    state_effects::advance_time(state, portions);
    state.character.turn += 1;
    state.character.heal(healing);
    state_effects::remove_condition(&mut state.character.conditions, "Exhausted");
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
        state_effects::advance_time(state, 1);
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
