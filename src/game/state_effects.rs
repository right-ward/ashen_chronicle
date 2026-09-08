use crate::model::{Condition, GameState};

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
    if amount > 0 {
        crate::procedural_opportunities::evolve_generated_world(state);
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

pub(crate) fn remove_condition(conditions: &mut Vec<Condition>, name: &str) {
    conditions.retain(|condition| condition.name != name);
}

pub(crate) fn is_night(points: u32) -> bool {
    matches!(points % 12, 0 | 1 | 10 | 11)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_campaign_content;
    use crate::model::{GameState, WorldGenerationMetadata, WorldMode};
    use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};
    use crate::procedural_entities::populate_generated_entities;
    use crate::procedural_opportunities::populate_generated_opportunities;
    use crate::procedural_relationships::populate_generated_relationships;

    #[test]
    fn advancing_time_applies_generated_world_evolution() {
        let content = load_campaign_content();
        let seed = 9090;
        let config = WorldGenerationConfig::default();
        let mut world = generate_world("Ashen", seed, config);
        world.mode = WorldMode::New;
        world.generation = Some(WorldGenerationMetadata {
            seed,
            region_count: config.region_count,
            location_count: config.location_count,
            extra_edges: config.extra_edges,
        });
        place_authored_content(&mut world, &content);
        let character = world.spawn_character("Test".into(), "Warden".into());
        let mut state = GameState {
            world,
            character,
            threat: Default::default(),
            corpses: Vec::new(),
            factions: Vec::new(),
            npcs: Vec::new(),
            quests: Vec::new(),
            last_announced_location_id: None,
            rng_state: 0,
            campaign_content: Some(content.clone()),
        };
        populate_generated_entities(&mut state, &content);
        crate::procedural_authored::integrate_authored_content(&mut state, &content);
        populate_generated_relationships(&mut state);
        populate_generated_opportunities(&mut state);

        let quest_index = state
            .quests
            .iter()
            .position(|quest| quest.content_id.starts_with("generated.quest."))
            .expect("generated quest should exist");
        let target_id = state.quests[quest_index].target_location_id;
        state.quests[quest_index].completed = true;
        state.quests[quest_index].reward_claimed = true;
        advance_time(&mut state, 1);
        assert!(!state.world.location_by_id(target_id).unwrap().dangerous);
    }
}
