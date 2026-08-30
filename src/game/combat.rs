use super::{character, interactions, legacy, state_effects};
use crate::model::{EntityId, GameState, Item};
use crate::ui::{
    choose_from_list, clear_combat_health, clear_log, pause, set_dashboard, Dashboard,
};

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
    let mut events = vec![format!("{} engages {} at {}.", character_name, encounter.enemy_name, location.name)];
    render_combat_screen(state, &encounter, &events, &location.name);

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
            render_combat_screen(state, &encounter, &events, &location.name);
            pause();
            clear_combat_health();
            break;
        }

        render_combat_screen(state, &encounter, &events, &location.name);
        let choices = vec![
            "Attack".to_string(),
            "Guard".to_string(),
            "Flee".to_string(),
        ];
        match choose_from_list("Combat actions", &choices, None)? {
            Some(0) => {
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
            Some(1) => {
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
            Some(2) => {
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
                render_combat_screen(state, &encounter, &events, &location.name);
                pause();
                clear_combat_health();
                break;
            }
            _ => {}
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
            render_combat_screen(state, &encounter, &events, &location.name);
            pause();
            clear_combat_health();
            break;
        }
        trim_combat_events(&mut events);
        render_combat_screen(state, &encounter, &events, &location.name);
    }
    clear_combat_health();
    Ok(())
}

fn render_combat_screen(
    state: &GameState,
    encounter: &CombatEncounter,
    events: &[String],
    location_name: &str,
) {
    let character = &state.character;
    let dashboard = Dashboard {
        world_name: "COMBAT ENCOUNTER".to_string(),
        hp: character.hp,
        max_hp: character.max_hp,
        enemy_name: Some(encounter.enemy_name.clone()),
        enemy_hp: Some(encounter.enemy_hp.max(0)),
        enemy_max_hp: Some(encounter.enemy_max_hp),
        time_display: format!("Turn {}", character.turn),
        condition_line: Some(format!("You: {}", character.display_name())),
        location_name: Some(format!("{}  vs  {}", character.display_name(), encounter.enemy_name)),
        location_description: Some(format!("Encounter location: {}", location_name)),
        danger_line: Some(format!("Enemy power: {}", encounter.enemy_power)),
        threat_line: None,
        action_hint: Some("Choose an action. Arrows / Enter".to_string()),
    };
    set_dashboard(dashboard);
    clear_log();
    for event in events.iter().rev().take(8).rev() {
        crate::ui::line(event);
    }
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
