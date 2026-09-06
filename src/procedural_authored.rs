use std::collections::HashSet;

use crate::content::CampaignContent;
use crate::model::{GameState, Region};
use crate::procedural_characteristics::{RegionCharacteristics, WorldCharacteristics};

const AUTHORED_ANCHOR_MARKER: &str = "[authored anchor]";

/// Integrate authored campaign entities with the generated world without
/// replacing their canonical names, locations, faction assignments, or quest
/// references. Anchor information is stored in existing faction/NPC memory
/// so it persists with the normal save data.
pub fn integrate_authored_content(state: &mut GameState, content: &CampaignContent) -> usize {
    if state.world.generation.is_none() {
        return 0;
    }

    let authored_npc_names = content
        .npcs
        .iter()
        .map(|npc| npc.name.as_str())
        .collect::<HashSet<_>>();
    let mut added = 0;

    for npc_content in &content.npcs {
        let Some(npc_index) = state
            .npcs
            .iter()
            .position(|npc| npc.name == npc_content.name)
        else {
            continue;
        };
        let npc = &state.npcs[npc_index];
        let Some(location) = state.world.location_by_id(npc.location_id) else {
            continue;
        };
        let Some(region) = state
            .world
            .regions
            .iter()
            .find(|region| region.id == location.region_id)
        else {
            continue;
        };
        let marker = format!(
            "{AUTHORED_ANCHOR_MARKER} {} operates at {} in {}.",
            npc.name, location.name, region.name
        );
        if !state.npcs[npc_index]
            .memory
            .iter()
            .any(|entry| entry == &marker)
        {
            state.npcs[npc_index].memory.push(marker);
            added += 1;
        }
    }

    let authored_faction_regions = content
        .factions
        .iter()
        .filter_map(|faction| {
            content
                .npcs
                .iter()
                .filter(|npc| npc.faction_name.as_deref() == Some(faction.name.as_str()))
                .filter_map(|npc| state.npcs.iter().find(|candidate| candidate.name == npc.name))
                .filter_map(|npc| state.world.location_by_id(npc.location_id))
                .filter_map(|location| {
                    state
                        .world
                        .regions
                        .iter()
                        .find(|region| region.id == location.region_id)
                })
                .min_by_key(|region| region.id)
                .map(|region| (faction.name.as_str(), region.name.as_str()))
        })
        .collect::<Vec<_>>();

    for (faction_name, region_name) in authored_faction_regions {
        let marker = format!(
            "{AUTHORED_ANCHOR_MARKER} {faction_name} is anchored in {region_name}."
        );
        let Some(faction) = state
            .factions
            .iter_mut()
            .find(|faction| faction.name == faction_name)
        else {
            continue;
        };
        if !faction.memory.iter().any(|entry| entry == &marker) {
            faction.memory.push(marker);
            added += 1;
        }
    }

    debug_assert!(content.npcs.iter().all(|npc| {
        !authored_npc_names.contains(npc.name.as_str())
            || state.npcs.iter().any(|runtime| runtime.name == npc.name)
    }));

    added
}

pub(crate) fn authored_anchor_region(
    faction_name: &str,
    faction_memory: &[String],
    world_regions: &[Region],
    characteristics: &WorldCharacteristics,
) -> Option<RegionCharacteristics> {
    let prefix = format!("{AUTHORED_ANCHOR_MARKER} {faction_name} is anchored in ");
    let region_name = faction_memory
        .iter()
        .find_map(|entry| entry.strip_prefix(&prefix))
        .map(|value| value.trim_end_matches('.'))?;
    let region = world_regions.iter().find(|region| region.name == region_name)?;
    characteristics
        .regions
        .iter()
        .find(|characteristics| characteristics.region_id == region.id)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_campaign_content;
    use crate::model::{WorldGenerationMetadata, WorldMode};
    use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};

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
    fn authored_content_is_anchored_deterministically() {
        let mut first = generated_state();
        let mut second = generated_state();
        let first_content = first.campaign_content.clone().expect("content is loaded");
        let second_content = second.campaign_content.clone().expect("content is loaded");

        crate::game::world::bootstrap_campaign_content(&mut first);
        crate::game::world::bootstrap_campaign_content(&mut second);

        let first_markers = first
            .npcs
            .iter()
            .filter_map(|npc| {
                npc.memory
                    .iter()
                    .find(|entry| entry.starts_with(AUTHORED_ANCHOR_MARKER))
            })
            .cloned()
            .collect::<Vec<_>>();
        let second_markers = second
            .npcs
            .iter()
            .filter_map(|npc| {
                npc.memory
                    .iter()
                    .find(|entry| entry.starts_with(AUTHORED_ANCHOR_MARKER))
            })
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(first_markers, second_markers);

        for faction in &first.factions {
            if first_content
                .factions
                .iter()
                .any(|authored| authored.name == faction.name)
            {
                assert!(faction
                    .memory
                    .iter()
                    .any(|entry| entry.starts_with(AUTHORED_ANCHOR_MARKER)));
            }
        }
        assert_eq!(first_content.factions.len(), second_content.factions.len());
    }

    #[test]
    fn authored_quest_references_remain_valid_after_generation() {
        let mut state = generated_state();
        let content = state.campaign_content.clone().expect("content is loaded");
        crate::game::world::bootstrap_campaign_content(&mut state);

        for quest in &content.quests {
            let runtime_quest = state
                .quests
                .iter()
                .find(|candidate| candidate.content_id == quest.id)
                .expect("authored quest should be present");
            assert_eq!(
                state
                    .world
                    .location_by_id(runtime_quest.target_location_id)
                    .map(|location| location.name.as_str()),
                Some(quest.location_name.as_str())
            );
            assert!(state.factions.iter().any(|faction| {
                faction.id == runtime_quest.faction_id && faction.name == quest.faction_name
            }));
            assert!(state.npcs.iter().any(|npc| {
                npc.id == runtime_quest.giver_npc_id && npc.name == quest.giver_npc_name
            }));
        }
    }
}
