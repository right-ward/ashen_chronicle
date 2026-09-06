use std::collections::HashMap;

use crate::content::CampaignContent;
use crate::model::GameState;

const AUTHORED_ANCHOR_MARKER: &str = "[authored anchor]";

/// Integrate authored campaign entities with the generated world without
/// replacing their canonical names, locations, faction assignments, or quest
/// references. Anchor information is stored in the existing faction/NPC
/// memory fields so it persists with the normal save data.
pub fn integrate_authored_content(state: &mut GameState, content: &CampaignContent) -> usize {
    if state.world.generation.is_none() {
        return 0;
    }

    let authored_npc_names = content
        .npcs
        .iter()
        .map(|npc| npc.name.as_str())
        .collect::<HashMap<_, _>>();
    let mut added = 0;

    for npc_content in &content.npcs {
        let Some(npc) = state.npcs.iter().find(|npc| npc.name == npc_content.name) else {
            continue;
        };
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
        if let Some(runtime_npc) = state.npcs.iter_mut().find(|candidate| candidate.id == npc.id) {
            if !runtime_npc.memory.iter().any(|entry| entry == &marker) {
                runtime_npc.memory.push(marker);
                added += 1;
            }
        }
    }

    let authored_faction_regions = content
        .factions
        .iter()
        .filter_map(|faction| {
            let region = content
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
                .map(|region| (faction.name.as_str(), region.name.as_str()));
            region
        })
        .collect::<HashMap<_, _>>();

    for faction_content in &content.factions {
        let Some(region_name) = authored_faction_regions.get(faction_content.name.as_str()) else {
            continue;
        };
        let marker = format!(
            "{AUTHORED_ANCHOR_MARKER} {} is anchored in {}.",
            faction_content.name, region_name
        );
        let Some(faction) = state
            .factions
            .iter_mut()
            .find(|faction| faction.name == faction_content.name)
        else {
            continue;
        };
        if !faction.memory.iter().any(|entry| entry == &marker) {
            faction.memory.push(marker);
            added += 1;
        }
    }

    debug_assert!(content.npcs.iter().all(|npc| {
        !authored_npc_names.contains_key(npc.name.as_str())
            || state.npcs.iter().any(|runtime| runtime.name == npc.name)
    }));

    added
}

pub(crate) fn authored_anchor_region(
    faction_name: &str,
    faction_memory: &[String],
    world_regions: &[crate::model::Region],
) -> Option<crate::procedural_characteristics::RegionCharacteristics> {
    let prefix = format!("{AUTHORED_ANCHOR_MARKER} {faction_name} is anchored in ");
    let region_name = faction_memory
        .iter()
        .find_map(|entry| entry.strip_prefix(&prefix))
        .map(str::trim_end_matches)?
        .strip_suffix('.')?;
    let region = world_regions.iter().find(|region| region.name == region_name)?;
    crate::procedural_characteristics::generate_world_characteristics_from_regions(world_regions)
        .regions
        .into_iter()
        .find(|characteristics| characteristics.region_id == region.id)
}
