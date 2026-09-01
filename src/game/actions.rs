use crate::game::state_effects::{self, add_or_refresh_condition};
use crate::model::{Condition, GameState};
use crate::persistence::save_game;
use crate::presentation::{
    CharacterView, MeditationResultView, MeditationTargetView, MeditationView,
};
use crate::ui::{choose_from_list, narrate, set_menu_screen};
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

pub(crate) fn travel_to(
    state: &mut GameState,
    target_id: crate::model::EntityId,
) -> std::io::Result<()> {
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

fn character_view(state: &GameState) -> CharacterView {
    CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    }
}

fn build_meditation_view(state: &GameState) -> MeditationView {
    let safe_to_meditate = !state.threat.active
        && !state
            .world
            .location_is_dangerous(state.character.location_id);
    MeditationView {
        character: character_view(state),
        current_time: crate::game::time::time_display(state.world.time_points, state.world.day),
        safe_to_meditate,
        unavailable_message: (!safe_to_meditate)
            .then(|| "The place is not safe enough to meditate here.".to_string()),
        targets: MEDITATION_TARGETS
            .iter()
            .map(|(_, label)| MeditationTargetView {
                label: (*label).to_string(),
            })
            .collect(),
    }
}

pub(crate) fn meditate_and_save(state: &mut GameState, save_path: &Path) -> std::io::Result<()> {
    let view = build_meditation_view(state);
    if !view.safe_to_meditate {
        set_menu_screen("Meditation", view.unavailable_message.clone(), None);
        let _ = choose_from_list("Meditation", &["Back".to_string()], None)?;
        return Ok(());
    }

    set_menu_screen(
        "Meditation",
        Some(format!(
            "You settle into stillness.\nCurrent time:\n{}\n\nChoose when to end your meditation.",
            view.current_time
        )),
        None,
    );

    let options: Vec<String> = view
        .targets
        .iter()
        .map(|target| target.label.clone())
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
    let target_label = view.targets[selection].label.clone();
    state.world.record_history(
        state.character.turn,
        format!(
            "{} meditated until {} and recovered.",
            character_name, target_label
        ),
    );
    save_game(save_path, state)?;

    let result = MeditationResultView {
        ending_time: crate::game::time::time_display(state.world.time_points, state.world.day),
        portions,
        hp_recovered: healing,
        exhausted_removed: true,
        well_rested_applied: true,
    };
    let mut result_lines = vec![
        "Your breathing steadies as you meditate.".to_string(),
        String::new(),
        result.ending_time.clone(),
        format!("Time meditated: {} portion(s)", result.portions),
        format!("HP recovered: {}", result.hp_recovered),
    ];
    if result.exhausted_removed {
        result_lines.extend([String::new(), "Exhausted is removed.".to_string()]);
    }
    if result.well_rested_applied {
        result_lines.push("Well-rested is applied.".to_string());
    }
    set_menu_screen("Meditation — Complete", Some(result_lines.join("\n")), None);
    let _ = choose_from_list("Meditation result", &["Back".to_string()], None)?;
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
