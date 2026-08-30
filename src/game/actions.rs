use crate::game::state_effects::{self, add_or_refresh_condition};
use crate::model::{Condition, GameState};
use crate::persistence::save_game;
use crate::ui::{choose_from_list, narrate};
use std::path::Path;

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}

const MEDITATION_TARGETS: [(u32, &str); 8] = [
    (2, "Dawn"),
    (3, "Morning"),
    (5, "High Sun"),
    (6, "Afternoon"),
    (8, "Dusk"),
    (9, "Evening"),
    (11, "Midnight"),
    (0, "Deep Night"),
];

pub(crate) fn travel_to(state: &mut GameState, target_id: crate::model::EntityId) -> std::io::Result<()> {
    let current_location = match state.world.location_by_id(state.character.location_id) {
        Some(location) => location.clone(),
        None => {
            println!("You are lost in a location that no longer exists.");
            crate::ui::pause();
            return Ok(());
        }
    };

    if !current_location.exits.contains(&target_id) {
        println!("That route is not available from here.");
        crate::ui::pause();
        return Ok(());
    }

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
    crate::game::quests::sync_active_quests(state);
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
    Ok(())
}

pub(crate) fn meditate_and_save(state: &mut GameState, save_path: &Path) -> std::io::Result<()> {
    let location_is_dangerous = state
        .world
        .location_is_dangerous(state.character.location_id);
    if state.threat.active || location_is_dangerous {
        println!("Not safe enough to meditate here.");
        crate::ui::pause();
        return Ok(());
    }

    let options: Vec<String> = MEDITATION_TARGETS
        .iter()
        .map(|(_, label)| (*label).to_string())
        .collect();
    let Some(selection) = choose_from_list("Stop meditation at", &options, Some("Cancel"))? else {
        return Ok(());
    };
    let target_slot = MEDITATION_TARGETS[selection].0;
    let current_slot = state.world.time_points % 12;
    let portions = ((target_slot + 12 - current_slot) % 12).max(1);

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
            "{} meditated until {} and recovered.",
            character_name,
            MEDITATION_TARGETS[selection].1
        ),
    );
    save_game(save_path, state)?;
    narrate(&format!(
        "You meditate until your breathing steadies. You look at the sky...\nStopped at {}.\nYou recover {} HP and save the game.",
        crate::game::time::time_display(state.world.time_points, state.world.day),
        healing
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::MEDITATION_TARGETS;

    #[test]
    fn meditation_targets_advance_to_the_next_occurrence() {
        let current = 4;
        let morning = MEDITATION_TARGETS
            .iter()
            .find(|(slot, _)| *slot == 3)
            .map(|(slot, _)| *slot)
            .unwrap();
        let dusk = MEDITATION_TARGETS
            .iter()
            .find(|(slot, _)| *slot == 8)
            .map(|(slot, _)| *slot)
            .unwrap();
        assert_eq!((morning + 12 - current) % 12, 11);
        assert_eq!((dusk + 12 - current) % 12, 4);
    }

    #[test]
    fn meditation_does_not_allow_a_zero_time_advance() {
        let current = 8;
        let dusk = MEDITATION_TARGETS
            .iter()
            .find(|(slot, _)| *slot == 8)
            .map(|(slot, _)| *slot)
            .unwrap();
        assert_eq!(((dusk + 12 - current) % 12).max(1), 1);
    }
}
