use crate::model::{EntityId, Faction, GameState};
use crate::procedural_characteristics::{
    generate_world_characteristics, Climate, RegionCharacteristics, RegionTheme,
};

const RELATIONSHIP_MARKER: &str = "[generated relationship]";

/// Add deterministic relationships between generated factions using the
/// characteristics of the regions they inhabit. Relationships are stored in
/// faction memory so they persist with the existing runtime model and saves.
pub fn populate_generated_relationships(state: &mut GameState) -> usize {
    if state.world.generation.is_none() {
        return 0;
    }

    let characteristics = generate_world_characteristics(&state.world);
    let mut generated = state
        .factions
        .iter()
        .filter(|faction| is_generated_faction(faction))
        .filter_map(|faction| {
            faction_region(faction, &state.world.regions, &characteristics)
                .map(|region| (faction.id, region))
        })
        .collect::<Vec<_>>();
    generated.sort_by_key(|(faction_id, _)| *faction_id);

    let mut added = 0;
    for left_index in 0..generated.len() {
        for right_index in left_index + 1..generated.len() {
            let (left_id, left_region) = &generated[left_index];
            let (right_id, right_region) = &generated[right_index];
            let Some((kind, strength, reason)) = relationship_for(left_region, right_region)
            else {
                continue;
            };

            let left_name = state
                .factions
                .iter()
                .find(|faction| faction.id == *left_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_default();
            let right_name = state
                .factions
                .iter()
                .find(|faction| faction.id == *right_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_default();

            let left_memory = format!(
                "{RELATIONSHIP_MARKER} {kind} with {right_name} (strength {strength}): {reason}"
            );
            let right_memory = format!(
                "{RELATIONSHIP_MARKER} {kind} with {left_name} (strength {strength}): {reason}"
            );
            let already_present = state
                .factions
                .iter()
                .find(|faction| faction.id == *left_id)
                .map(|faction| faction.memory.iter().any(|entry| entry == &left_memory))
                .unwrap_or(true);
            if already_present {
                continue;
            }

            if let Some(faction) = state.factions.iter_mut().find(|faction| faction.id == *left_id) {
                faction.memory.push(left_memory);
            }
            if let Some(faction) = state.factions.iter_mut().find(|faction| faction.id == *right_id) {
                faction.memory.push(right_memory);
            }
            added += 1;
        }
    }

    added
}

fn is_generated_faction(faction: &Faction) -> bool {
    faction
        .memory
        .iter()
        .any(|entry| entry.starts_with("A generated faction shaped by"))
}

fn faction_region<'a>(
    faction: &Faction,
    world_regions: &'a [crate::model::Region],
    characteristics: &'a crate::procedural_characteristics::WorldCharacteristics,
) -> Option<&'a RegionCharacteristics> {
    let world_region = world_regions
        .iter()
        .find(|region| faction.name.ends_with(&format!("of {}", region.name)))?;
    characteristics
        .regions
        .iter()
        .find(|region| region.region_id == world_region.id)
}

fn relationship_for(
    left: &RegionCharacteristics,
    right: &RegionCharacteristics,
) -> Option<(&'static str, u8, String)> {
    let shared_resource = left
        .resources
        .iter()
        .find(|resource| right.resources.contains(resource));
    if let Some(resource) = shared_resource {
        let strength = 45 + left.danger.max(right.danger) / 2;
        return Some((
            "rivalry",
            strength,
            format!(
                "both regions depend on {resource}, and the surrounding danger increases competition"
            ),
        ));
    }

    if left.danger >= 60 && right.danger >= 60 {
        return Some((
            "alliance",
            70,
            "both regions face severe danger, creating pressure to cooperate for mutual protection"
                .to_string(),
        ));
    }

    if complementary_resources(left, right) {
        return Some((
            "trade",
            55,
            format!(
                "{} resources complement {} resources across the two regions",
                left.resources.join(", "),
                right.resources.join(", ")
            ),
        ));
    }

    let prosperity_gap = left.prosperity.abs_diff(right.prosperity);
    if prosperity_gap >= 30 {
        return Some((
            "influence",
            40 + prosperity_gap.min(50),
            "the more prosperous region has greater leverage over its less prosperous neighbour"
                .to_string(),
        ));
    }

    if left.climate != right.climate && left.theme != right.theme {
        return Some((
            "dependency",
            35,
            "different environments create a practical need for resources and services from the other region"
                .to_string(),
        ));
    }

    None
}

fn complementary_resources(
    left: &RegionCharacteristics,
    right: &RegionCharacteristics,
) -> bool {
    left.resources.iter().any(|left_resource| {
        right.resources.iter().any(|right_resource| {
            matches!(
                (left_resource.as_str(), right_resource.as_str()),
                ("fish", "timber")
                    | ("timber", "fish")
                    | ("fish", "ore")
                    | ("ore", "fish")
                    | ("timber", "ore")
                    | ("ore", "timber")
                    | ("herbs", "ore")
                    | ("ore", "herbs")
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_campaign_content;
    use crate::model::{GameState, WorldGenerationMetadata, WorldMode};
    use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};
    use crate::procedural_entities::populate_generated_entities;

    fn generated_state() -> GameState {
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
        GameState {
            world,
            character,
            threat: Default::default(),
            corpses: Vec::new(),
            factions: Vec::new(),
            npcs: Vec::new(),
            quests: Vec::new(),
            last_announced_location_id: None,
            campaign_content: Some(content),
        }
    }

    #[test]
    fn generated_relationships_are_deterministic_and_idempotent() {
        let content = load_campaign_content();
        let mut first_state = generated_state();
        populate_generated_entities(&mut first_state, &content);
        let first_added = populate_generated_relationships(&mut first_state);
        let first_memories = relationship_memories(&first_state);

        let mut second_state = generated_state();
        populate_generated_entities(&mut second_state, &content);
        let second_added = populate_generated_relationships(&mut second_state);
        assert_eq!(first_added, second_added);
        assert_eq!(first_memories, relationship_memories(&second_state));

        let repeated = populate_generated_relationships(&mut first_state);
        assert_eq!(repeated, 0);
        assert_eq!(first_memories, relationship_memories(&first_state));
    }

    #[test]
    fn generated_relationships_reference_existing_factions() {
        let content = load_campaign_content();
        let mut state = generated_state();
        populate_generated_entities(&mut state, &content);
        let added = populate_generated_relationships(&mut state);
        assert!(added > 0);

        for faction in &state.factions {
            for memory in faction
                .memory
                .iter()
                .filter(|entry| entry.starts_with(RELATIONSHIP_MARKER))
            {
                let Some(target) = memory.split(" with ").nth(1) else {
                    panic!("relationship memory should contain a target");
                };
                let target = target.split(" (strength").next().unwrap_or_default();
                assert!(state.factions.iter().any(|candidate| candidate.name == target));
            }
        }
    }

    fn relationship_memories(state: &GameState) -> Vec<(EntityId, Vec<String>)> {
        state
            .factions
            .iter()
            .filter(|faction| is_generated_faction(faction))
            .map(|faction| {
                (
                    faction.id,
                    faction
                        .memory
                        .iter()
                        .filter(|entry| entry.starts_with(RELATIONSHIP_MARKER))
                        .cloned()
                        .collect(),
                )
            })
            .collect()
    }
}
