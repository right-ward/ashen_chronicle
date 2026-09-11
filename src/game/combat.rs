use super::{character, interactions, legacy, state_effects};
use crate::model::{EntityId, GameState, Item};
use crate::presentation::{CharacterView, CombatResultView, CombatView, CombatantView};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CombatEncounter {
    pub(crate) enemy_name: String,
    pub(crate) enemy_hp: i32,
    pub(crate) enemy_max_hp: i32,
    pub(crate) enemy_power: i32,
    pub(crate) enemy_id: EntityId,
    pub(crate) location_id: EntityId,
    pub(crate) location_name: String,
    trophy_name: String,
    pub(crate) events: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CombatStartError {
    NoActiveThreat,
    MissingLocation,
}

impl fmt::Display for CombatStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveThreat => formatter.write_str("There is no active threat to face."),
            Self::MissingLocation => formatter.write_str("The threat cannot be reached here."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CombatResult {
    Victory,
    Fled,
    Defeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CombatStep {
    Continue,
    Result {
        view: Box<CombatResultView>,
        outcome: CombatResult,
    },
}

pub(crate) fn start_encounter(state: &mut GameState) -> Result<CombatEncounter, CombatStartError> {
    if !state.threat.active {
        return Err(CombatStartError::NoActiveThreat);
    }
    let location = state
        .world
        .location_by_id(state.character.location_id)
        .cloned()
        .ok_or(CombatStartError::MissingLocation)?;
    let (enemy_name, enemy_hp, enemy_power, trophy_name) = encounter_profile(state, &location.name);
    let enemy_max_hp = enemy_hp.max(1);
    let character_name = state.character.display_name();
    Ok(CombatEncounter {
        enemy_name,
        enemy_hp,
        enemy_max_hp,
        enemy_power,
        enemy_id: state.world.allocate_id(),
        location_id: location.id,
        location_name: location.name,
        trophy_name,
        events: vec![format!(
            "{} engages the threat at {}.",
            character_name,
            location_name_for_message(state)
        )],
    })
}

pub(crate) fn resolve_action(
    state: &mut GameState,
    encounter: &mut CombatEncounter,
    action: usize,
) -> CombatStep {
    match action {
        0 => {
            state_effects::advance_time(state, 1);
            state.character.turn += 1;
            let damage = (3 + state.character.effective_might()).max(1);
            encounter.enemy_hp = (encounter.enemy_hp - damage).max(0);
            encounter.events.push(format!(
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
            if encounter.enemy_hp <= 0 {
                return finish_victory(state, encounter);
            }
            let retaliation = encounter.enemy_power;
            encounter.events.push(take_combat_damage(
                state,
                retaliation,
                &encounter.enemy_name,
                &encounter.location_name,
            ));
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
            encounter.events.push(format!(
                "You guard. Incoming damage is reduced to {}.",
                retaliation
            ));
            if retaliation > 0 {
                encounter.events.push(take_combat_damage(
                    state,
                    retaliation,
                    &encounter.enemy_name,
                    &encounter.location_name,
                ));
            } else {
                encounter
                    .events
                    .push("The blow glances off harmlessly.".to_string());
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
                    character_name, encounter.enemy_name, encounter.location_name
                ),
            );
            encounter.events.push(format!(
                "You flee. {} remains in {}.",
                encounter.enemy_name, encounter.location_name
            ));
            trim_combat_events(&mut encounter.events);
            return CombatStep::Result {
                view: Box::new(build_result_view(
                    state,
                    encounter,
                    &encounter.location_name,
                    "Fled",
                    "The threat remains.",
                )),
                outcome: CombatResult::Fled,
            };
        }
        _ => return CombatStep::Continue,
    }

    if state.character.hp <= 0 {
        return finish_defeat(state, encounter);
    }

    trim_combat_events(&mut encounter.events);
    CombatStep::Continue
}

pub(crate) fn build_combat_view(state: &GameState, encounter: &CombatEncounter) -> CombatView {
    CombatView {
        character: character_view(state),
        player_condition: active_condition(state).map(str::to_string),
        enemy: CombatantView {
            name: encounter.enemy_name.clone(),
            current_hp: encounter.enemy_hp,
            max_hp: encounter.enemy_max_hp,
        },
        enemy_power: encounter.enemy_power,
        location_name: encounter.location_name.clone(),
        turn: state.character.turn,
        events: encounter.events.clone(),
        actions: vec![
            "Attack".to_string(),
            "Guard".to_string(),
            "Flee".to_string(),
        ],
    }
}

fn finish_victory(state: &mut GameState, encounter: &mut CombatEncounter) -> CombatStep {
    let enemy_name = encounter.enemy_name.clone();
    let character_name = state.character.display_name();
    state.threat.clear();
    if let Some(location) = state.world.location_by_id_mut(encounter.location_id) {
        location.dangerous = false;
    }
    state.character.turn += 1;
    state.world.record_history(
        state.character.turn,
        format!(
            "{} defeated {} at {}.",
            character_name, enemy_name, encounter.location_name
        ),
    );
    let trophy = Item {
        id: encounter.enemy_id,
        name: encounter.trophy_name.clone(),
        description: format!(
            "A proof that the {} was confronted and survived.",
            encounter.location_name
        ),
    };
    state.character.inventory.push(trophy.clone());
    interactions::grant_reward_reputation(state, &trophy);
    interactions::update_faction_memory_for_location(
        state,
        encounter.location_id,
        format!("{} was cleared of danger.", encounter.location_name),
    );
    crate::game::quests::record_enemy_defeat(state, &enemy_name, encounter.location_id);
    character::gain_experience(state, 15);
    let result_note = format!(
        "Loot: {}\n{}\n\nThe threat is broken. The place is quieter now.",
        trophy.name, trophy.description
    );
    encounter.events.push("Victory".to_string());
    encounter.events.push(format!("Defeated: {}", enemy_name));
    encounter.events.push(format!("Loot: {}", trophy.name));
    encounter
        .events
        .push(format!("Description: {}", trophy.description));
    encounter
        .events
        .push("The threat is broken. The place is quieter now.".to_string());
    trim_combat_events(&mut encounter.events);
    CombatStep::Result {
        view: Box::new(build_result_view(
            state,
            encounter,
            &encounter.location_name,
            "Victory",
            &result_note,
        )),
        outcome: CombatResult::Victory,
    }
}

fn finish_defeat(state: &mut GameState, encounter: &mut CombatEncounter) -> CombatStep {
    let location_name = encounter.location_name.clone();
    legacy::mark_character_dead(
        state,
        format!("{} overcame them", encounter.enemy_name),
        &location_name,
    );
    encounter.events.push("Defeat".to_string());
    encounter.events.push("You were overwhelmed.".to_string());
    trim_combat_events(&mut encounter.events);
    CombatStep::Result {
        view: Box::new(build_result_view(
            state,
            encounter,
            &location_name,
            "Defeat",
            "You were overwhelmed.",
        )),
        outcome: CombatResult::Defeat,
    }
}

fn build_result_view(
    state: &GameState,
    encounter: &CombatEncounter,
    location_name: &str,
    result_title: &str,
    result_note: &str,
) -> CombatResultView {
    let _ = location_name;
    CombatResultView {
        combat: build_combat_view(state, encounter),
        result_title: result_title.to_string(),
        result_note: result_note.to_string(),
    }
}

fn character_view(state: &GameState) -> CharacterView {
    CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    }
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

fn location_name_for_message(state: &GameState) -> String {
    state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| location.name.clone())
        .unwrap_or_else(|| "unknown location".to_string())
}
