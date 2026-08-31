use super::{character, combat_screen, interactions, legacy, state_effects};
use crate::model::{EntityId, GameState, Item};
use crate::ui::{clear_combat_health, pause};

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}

struct CombatEncounter {
    enemy_name: String,
    enemy_hp: i32,
    enemy_max_hp: i32,
    enemy_power: i32,
    enemy_id: EntityId,
}

pub(crate) fn investigate_threat(state: &mut GameState) -> std::io::Result<()> {
    if !state.threat.active {
        println!("There is no active threat to face.");
        pause();
        return Ok(());
    }
    let location = match state.world.location_by_id(state.character.location_id) {
        Some(location) => location.clone(),
        None => {
            println!("The threat cannot be reached here.");
            pause();
            return Ok(());
        }
    };
    let (enemy_name, enemy_hp, enemy_power, trophy_name) = encounter_profile(state, &location.name);
    let enemy_max_hp = enemy_hp.max(1);
    let mut encounter = CombatEncounter {
        enemy_name,
        enemy_hp,
        enemy_max_hp,
        enemy_power,
        enemy_id: state.world.allocate_id(),
    };
    let character_name = state.character.display_name();
    let mut events = vec![format!(
        "{} engages {} at {}.",
        character_name, encounter.enemy_name, location.name
    )];

    loop {
        if !state.character.alive {
            break;
        }
        if encounter.enemy_hp <= 0 {
            let enemy_name = encounter.enemy_name.clone();
            let character_name = state.character.display_name();
            state.threat.clear();
            if let Some(loc) = state.world.location_by_id_mut(location.id) {
                loc.dangerous = false;
            }
            state.character.turn += 1;
            state.world.record_history(
                state.character.turn,
                format!(
                    "{} defeated {} at {}.",
                    character_name, enemy_name, location.name
                ),
            );
            let trophy = Item {
                id: encounter.enemy_id,
                name: trophy_name.clone(),
                description: format!(
                    "A proof that the {} was confronted and survived.",
                    location.name
                ),
            };
            state.character.inventory.push(trophy.clone());
            legacy::notify_item_gain(state, &trophy);
            interactions::update_faction_memory_for_location(
                state,
                location.id,
                format!("{} was cleared of danger.", location.name),
            );
            crate::game::quests::record_enemy_defeat(state, &enemy_name, location.id);
            character::gain_experience(state, 15);
            events.push("Victory".to_string());
            events.push(format!("Defeated: {}", enemy_name));
            events.push(format!("Loot: {}", trophy.name));
            events.push("The threat is broken. The place is quieter now.".to_string());
            trim_combat_events(&mut events);
            combat_screen::show_result(
                &character_name,
                state.character.hp,
                state.character.max_hp,
                active_condition(state),
                &encounter.enemy_name,
                encounter.enemy_hp,
                encounter.enemy_max_hp,
                encounter.enemy_power,
                &location.name,
                state.character.turn,
                &events,
                "Victory",
                "The threat is broken. The place is quieter now.",
            )?;
            combat_screen::wait_for_key()?;
            clear_combat_health();
            break;
        }

        let action = combat_screen::choose_action(
            &state.character.display_name(),
            state.character.hp,
            state.character.max_hp,
            active_condition(state),
            &encounter.enemy_name,
            encounter.enemy_hp,
            encounter.enemy_max_hp,
            encounter.enemy_power,
            &location.name,
            state.character.turn,
            &events,
        )?;

        match action {
            0 => {
                state_effects::advance_time(state, 1);
                state.character.turn += 1;
                let damage = (3 + state.character.effective_might()).max(1);
                encounter.enemy_hp = (encounter.enemy_hp - damage).max(0);
                events.push(format!(
                    "You strike {} for {} damage.",
                    encounter.enemy_name, damage
                ));
                let character_name = state.character.display_name();
                state.world.record_history(
                    state.character.turn,
                    format!(
                        "{} struck {} for {} damage.",
                        character_name, encounter.enemy_name, damage
                    ),
                );
                if encounter.enemy_hp > 0 {
                    let retaliation = encounter.enemy_power;
                    events.push(take_combat_damage(
                        state,
                        retaliation,
                        &encounter.enemy_name,
                        &location.name,
                    ));
                }
            }
            1 => {
                state_effects::advance_time(state, 1);
                state.character.turn += 1;
                let retaliation =
                    (encounter.enemy_power - 1 - state.character.attributes.endurance / 2).max(0);
                let character_name = state.character.display_name();
                state.world.record_history(
                    state.character.turn,
                    format!(
                        "{} guarded against {}.",
                        character_name, encounter.enemy_name
                    ),
                );
                events.push(format!(
                    "You guard. Incoming damage is reduced to {}.",
                    retaliation
                ));
                if retaliation > 0 {
                    events.push(take_combat_damage(
                        state,
                        retaliation,
                        &encounter.enemy_name,
                        &location.name,
                    ));
                } else {
                    events.push("The blow glances off harmlessly.".to_string());
                }
            }
            2 => {
                state_effects::advance_time(state, 1);
                state.character.turn += 1;
                let character_name = state.character.display_name();
                state.world.record_history(
                    state.character.turn,
                    format!(
                        "{} fled from {} at {}.",
                        character_name, encounter.enemy_name, location.name
                    ),
                );
                events.push(format!(
                    "You flee. {} remains in {}.",
                    encounter.enemy_name, location.name
                ));
                trim_combat_events(&mut events);
                combat_screen::show_result(
                    &character_name,
                    state.character.hp,
                    state.character.max_hp,
                    active_condition(state),
                    &encounter.enemy_name,
                    encounter.enemy_hp,
                    encounter.enemy_max_hp,
                    encounter.enemy_power,
                    &location.name,
                    state.character.turn,
                    &events,
                    "Fled",
                    "The threat remains.",
                )?;
                combat_screen::wait_for_key()?;
                clear_combat_health();
                break;
            }
            _ => unreachable!(),
        }

        if state.character.hp <= 0 {
            let location_name = location.name.clone();
            legacy::mark_character_dead(
                state,
                format!("{} overcame them", encounter.enemy_name),
                &location_name,
            );
            events.push("Defeat".to_string());
            events.push("You were overwhelmed.".to_string());
            trim_combat_events(&mut events);
            combat_screen::show_result(
                &state.character.display_name(),
                state.character.hp,
                state.character.max_hp,
                active_condition(state),
                &encounter.enemy_name,
                encounter.enemy_hp,
                encounter.enemy_max_hp,
                encounter.enemy_power,
                &location.name,
                state.character.turn,
                &events,
                "Defeat",
                "You were overwhelmed.",
            )?;
            combat_screen::wait_for_key()?;
            clear_combat_health();
            break;
        }

        trim_combat_events(&mut events);
    }
    clear_combat_health();
    Ok(())
}

fn active_condition(state: &GameState) -> Option<&str> {
    state
        .character
        .conditions
        .first()
        .map(|condition| condition.name.as_str())
}

fn trim_combat_events(events: &mut Vec<String>) {
    const MAX_EVENTS: usize = 12;
    if events.len() > MAX_EVENTS {
        let excess = events.len() - MAX_EVENTS;
        events.drain(0..excess);
    }
}

fn encounter_profile(state: &GameState, location_name: &str) -> (String, i32, i32, String) {
    if let Some(profile) = state
        .campaign_content
        .as_ref()
        .and_then(|content| content.encounter_for(location_name))
    {
        (
            profile.enemy_name.clone(),
            profile.enemy_hp,
            profile.enemy_power,
            profile.trophy_item_name.clone(),
        )
    } else {
        (
            "Ash-Crazed Marauder".to_string(),
            7,
            2,
            "Marauder's Token".to_string(),
        )
    }
}

fn take_combat_damage(
    state: &mut GameState,
    damage: i32,
    enemy_name: &str,
    location_name: &str,
) -> String {
    if damage <= 0 {
        return "The blow glances off harmlessly.".to_string();
    }
    state.character.hp -= damage;
    let character_name = state.character.display_name();
    state.world.record_history(
        state.character.turn,
        format!(
            "{} took {} damage from {} at {}.",
            character_name, damage, enemy_name, location_name
        ),
    );
    format!("You take {} damage from {}.", damage, enemy_name)
}
