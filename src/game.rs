mod actions;
mod character;
mod combat;
mod combat_screen;
#[allow(dead_code)]
mod console;
mod dispatcher;
#[path = "game/interactions.rs"]
mod interactions_core;
mod legacy;
mod menu;
mod navigation;
mod presentation;
mod quests;
mod records;
mod runtime;
mod screens;
mod state_effects;
mod time;
mod world;

mod interactions {
    pub(crate) use super::interactions_core::*;

    use crate::model::{GameState, Item};

    pub(crate) fn grant_reward_reputation(state: &mut GameState, item: &Item) {
        let Some(faction_name) = (match item.name.as_str() {
            "Wardens' Seal" => Some("Cinder Wardens"),
            "Rootworker's Token" => Some("Hollow Market Kin"),
            "Bell Covenant Charm" => Some("Drowned Bell Covenant"),
            _ => None,
        }) else {
            return;
        };
        let Some(faction_id) = super::interactions_core::faction_id_by_name(state, faction_name)
        else {
            return;
        };
        if let Some(faction) = state
            .factions
            .iter_mut()
            .find(|faction| faction.id == faction_id)
        {
            faction.reputation += 5;
            faction.memory.push(format!(
                "Carrying {} marks affiliation with the faction.",
                item.name
            ));
            if faction.memory.len() > 5 {
                let remove_count = faction.memory.len() - 5;
                faction.memory.drain(0..remove_count);
            }
        }
    }
}

pub fn run() -> std::io::Result<()> {
    let _ui = crate::ui::init()?;
    let Some((mut state, mut save_path)) = screens::start_screen()? else {
        return Ok(());
    };
    world::bootstrap_campaign_content(&mut state);
    runtime::main_loop(&mut state, &mut save_path)
}

pub(crate) use world::validate_loaded_state;
