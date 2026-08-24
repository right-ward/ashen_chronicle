use crate::game::{character, legacy, state_effects};
use crate::model::{EntityId, Faction, GameState, Item, Quest};
use crate::ui::{choose_from_list, pause};

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}

pub(crate) fn npc_ids_at_location(state: &GameState, location_id: EntityId) -> Vec<EntityId> {
    state
        .npcs
        .iter()
        .filter(|npc| npc.location_id == location_id)
        .map(|npc| npc.id)
        .collect()
}

pub(crate) fn npc_index_by_id(state: &GameState, npc_id: EntityId) -> Option<usize> {
    state.npcs.iter().position(|npc| npc.id == npc_id)
}

fn quest_key(quest: &Quest) -> String {
    if quest.content_id.is_empty() {
        format!("legacy.quest.{}", quest.id)
    } else {
        quest.content_id.clone()
    }
}

pub(crate) fn faction_id_by_name(state: &GameState, faction_name: &str) -> Option<EntityId> {
    state
        .factions
        .iter()
        .find(|faction| faction.name == faction_name)
        .map(|faction| faction.id)
}

fn faction_by_id_mut(state: &mut GameState, faction_id: EntityId) -> Option<&mut Faction> {
    state
        .factions
        .iter_mut()
        .find(|faction| faction.id == faction_id)
}

pub(crate) fn talk(state: &mut GameState) -> std::io::Result<()> {
    let location_id = state.character.location_id;
    let npc_ids = npc_ids_at_location(state, location_id);
    if npc_ids.is_empty() {
        println!("There is no one here to talk to.");
        pause();
        return Ok(());
    }
    let options: Vec<String> = npc_ids
        .iter()
        .filter_map(|id| npc_index_by_id(state, *id).map(|index| state.npcs[index].display_name()))
        .collect();
    if let Some(choice) = choose_from_list("Talk to whom?", &options, Some("Back"))? {
        talk_to_npc(state, npc_ids[choice])?;
    }
    Ok(())
}

fn talk_to_npc(state: &mut GameState, npc_id: EntityId) -> std::io::Result<()> {
    let Some(npc_index) = npc_index_by_id(state, npc_id) else {
        return Ok(());
    };
    let npc_name = state.npcs[npc_index].display_name();
    if !npc_is_available_now(state.world.time_points) {
        println!(
            "{}",
            npc_unavailable_message(&npc_name, state.world.time_points)
        );
        pause();
        return Ok(());
    }
    if let Some(memory) = state.npcs[npc_index].memory.last() {
        println!("{} remembers: {}", npc_name, memory);
    }
    if let Some(portrait) = state
        .campaign_content
        .as_ref()
        .and_then(|content| content.portrait_for(&state.npcs[npc_index].name))
    {
        println!("");
        println!("{}", portrait);
    }
    let quest_indices: Vec<usize> = state
        .quests
        .iter()
        .enumerate()
        .filter(|(_, quest)| quest.giver_npc_id == npc_id)
        .map(|(index, _)| index)
        .collect();
    let mut options = vec![
        "Ask if they need help".to_string(),
        "Tell them it's done".to_string(),
    ];
    let can_probe_memory =
        state.character.effective_insight() >= 2 && !state.npcs[npc_index].memory.is_empty();
    if can_probe_memory {
        options.push("Ask what they remember".to_string());
    }
    if quest_indices.is_empty() && !can_probe_memory {
        println!("{} has little to say.", npc_name);
        pause();
        return Ok(());
    }
    if let Some(choice) =
        choose_from_list(&format!("Talk to {}", npc_name), &options, Some("Back"))?
    {
        match choice {
            0 => {
                let mut found_offer = false;
                for quest_index in quest_indices {
                    let (quest_key, title, description, faction_id, offered, completed) = {
                        let quest = &state.quests[quest_index];
                        (
                            quest_key(quest),
                            quest.title.clone(),
                            quest.description.clone(),
                            quest.faction_id,
                            quest.offered,
                            quest.completed,
                        )
                    };
                    if state
                        .world
                        .completed_quest_ids
                        .iter()
                        .any(|known| known == &quest_key)
                        || completed
                    {
                        continue;
                    }
                    found_offer = true;
                    if offered {
                        println!(
                            "{} says: 'You already agreed to help with {}.'",
                            npc_name, title
                        );
                    } else {
                        if let Some(quest) = state.quests.get_mut(quest_index) {
                            quest.offered = true;
                        }
                        println!("{} says: '{}'", npc_name, description);
                        remember_npc(state, npc_id, format!("offered the quest {}", title));
                        remember_faction(
                            state,
                            faction_id,
                            format!("{} offered the quest {}.", npc_name, title),
                        );
                    }
                }
                if !found_offer {
                    println!(
                        "{} has no work for you. Whatever was asked here has already been done.",
                        npc_name
                    );
                }
                pause();
            }
            1 => {
                let mut handled = false;
                for quest_index in quest_indices {
                    let (quest_key, _title, offered, completed, required_item_name) = {
                        let quest = &state.quests[quest_index];
                        (
                            quest_key(quest),
                            quest.title.clone(),
                            quest.offered,
                            quest.completed,
                            quest.required_item_name.clone(),
                        )
                    };
                    if state
                        .world
                        .completed_quest_ids
                        .iter()
                        .any(|known| known == &quest_key)
                        || completed
                    {
                        continue;
                    }
                    if !offered {
                        println!("{} does not know what you are talking about. You have not accepted any work from them.", npc_name);
                        handled = true;
                        continue;
                    }
                    handled = true;
                    if state
                        .character
                        .inventory
                        .iter()
                        .any(|item| item.name == required_item_name)
                    {
                        complete_quest(state, quest_index);
                    } else {
                        println!(
                            "{} looks at you expectantly. You have not brought the required proof.",
                            npc_name
                        );
                    }
                }
                if !handled {
                    println!("{} has no unfinished deed to hear about.", npc_name);
                }
                pause();
            }
            2 if can_probe_memory => {
                if let Some(memory) = state.npcs[npc_index].memory.last() {
                    println!("{} searches your face, then recalls: {}", npc_name, memory);
                }
                pause();
            }
            _ => {}
        }
    }
    state_effects::advance_time(state, 1);
    Ok(())
}

fn complete_quest(state: &mut GameState, quest_index: usize) -> bool {
    let (quest_key, title, required_item_name, faction_id) = {
        let quest = &state.quests[quest_index];
        (
            quest_key(quest),
            quest.title.clone(),
            quest.required_item_name.clone(),
            quest.faction_id,
        )
    };
    if state
        .world
        .completed_quest_ids
        .iter()
        .any(|known| known == &quest_key)
    {
        return false;
    }
    let Some(item_index) = state
        .character
        .inventory
        .iter()
        .position(|item| item.name == required_item_name)
    else {
        return false;
    };
    state.character.inventory.remove(item_index);
    let current_character_name = state.character.display_name();
    if let Some(quest) = state.quests.get_mut(quest_index) {
        quest.completed = true;
        quest.reward_claimed = true;
        quest.completed_by = Some(current_character_name.clone());
    }
    state.world.completed_quest_ids.push(quest_key.clone());
    adjust_faction_reputation(
        state,
        faction_id,
        5,
        &format!("{} completed {}.", current_character_name, title),
    );
    let reward_name = state
        .quests
        .get(quest_index)
        .map(|quest| quest.reward_item_name.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Unnamed Reward".to_string());
    let reward = Item {
        id: state.world.allocate_id(),
        name: reward_name,
        description: format!("A token earned by completing {}.", title),
    };
    state.character.inventory.push(reward.clone());
    legacy::notify_item_gain(state, &reward);
    grant_reward_reputation(state, &reward);
    state.world.record_history(
        state.character.turn,
        format!("{} completed {}.", current_character_name, title),
    );
    println!("\nQuest complete: {}", title);
    println!("  Quest item consumed: {}", required_item_name);
    println!("  Reward: {}", reward.name);
    character::gain_experience(state, 25);
    println!("  Reputation: +5 for completing the deed, +5 while carrying the reward");
    true
}

pub(crate) fn grant_reward_reputation(state: &mut GameState, item: &Item) {
    let Some(faction_name) = (match item.name.as_str() {
        "Wardens' Seal" => Some("Cinder Wardens"),
        "Rootworker's Token" => Some("Hollow Market Kin"),
        "Bell Covenant Charm" => Some("Drowned Bell Covenant"),
        _ => None,
    }) else {
        return;
    };
    let Some(faction_id) = faction_id_by_name(state, faction_name) else {
        return;
    };
    adjust_faction_reputation(
        state,
        faction_id,
        5,
        &format!("Carrying {} marks affiliation with the faction.", item.name),
    );
}

fn remember_npc(state: &mut GameState, npc_id: EntityId, memory: String) {
    if let Some(index) = npc_index_by_id(state, npc_id) {
        let npc = &mut state.npcs[index];
        npc.memory.push(memory);
        if npc.memory.len() > 5 {
            let remove_count = npc.memory.len() - 5;
            npc.memory.drain(0..remove_count);
        }
    }
}

fn remember_faction(state: &mut GameState, faction_id: EntityId, memory: String) {
    if let Some(faction) = faction_by_id_mut(state, faction_id) {
        faction.memory.push(memory);
        if faction.memory.len() > 5 {
            let remove_count = faction.memory.len() - 5;
            faction.memory.drain(0..remove_count);
        }
    }
}

fn adjust_faction_reputation(
    state: &mut GameState,
    faction_id: EntityId,
    delta: i32,
    reason: &str,
) {
    if let Some(faction) = faction_by_id_mut(state, faction_id) {
        faction.reputation += delta;
        faction.memory.push(reason.to_string());
        if faction.memory.len() > 5 {
            let remove_count = faction.memory.len() - 5;
            faction.memory.drain(0..remove_count);
        }
    }
}

fn npc_unavailable_message(npc_name: &str, points: u32) -> String {
    let slot = points % 12;
    let (reason, hint) = match slot {
        0 | 1 => ("It is too late in the night.", "Try again after dawn."),
        10 | 11 => ("It is too late tonight.", "Try again in the morning."),
        2..=5 => (
            "It is still too early in the day.",
            "Try again later today.",
        ),
        6..=9 => ("It is too late in the day.", "Try again tomorrow morning."),
        _ => ("They are unavailable right now.", "Try again later."),
    };
    format!(
        "{} is not available right now. {} {}",
        npc_name, reason, hint
    )
}

fn npc_is_available_now(points: u32) -> bool {
    matches!(points % 12, 2..=9)
}

pub(crate) fn update_faction_memory_for_location(
    state: &mut GameState,
    location_id: EntityId,
    memory: String,
) {
    let npc_ids = npc_ids_at_location(state, location_id);
    let mut faction_ids = Vec::new();
    for npc_id in npc_ids {
        if let Some(index) = npc_index_by_id(state, npc_id) {
            let npc = &state.npcs[index];
            if let Some(faction_id) = npc.faction_id {
                faction_ids.push(faction_id);
                remember_npc(state, npc_id, memory.clone());
            }
        }
    }
    faction_ids.sort_unstable();
    faction_ids.dedup();
    for faction_id in faction_ids {
        remember_faction(state, faction_id, memory.clone());
    }
}
