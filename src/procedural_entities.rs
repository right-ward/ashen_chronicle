use std::collections::HashSet;

use crate::content::CampaignContent;
use crate::model::{EntityId, Faction, GameState, Npc, World};
use crate::procedural_characteristics::{
    generate_world_characteristics, Climate, LocationCharacteristics, LocationKind, RegionCharacteristics,
    RegionTheme,
};

const GIVEN_NAMES: &[&str] = &[
    "Aren", "Bryn", "Cala", "Dain", "Eira", "Fenn", "Garr", "Hale", "Iven", "Jora", "Kest", "Lysa",
    "Maren", "Neris", "Orin", "Pella", "Quin", "Rhea", "Soren", "Talia", "Ulric", "Vera", "Wren", "Ysolde",
];

const FAMILY_NAMES: &[&str] = &[
    "Vale", "Ash", "Morrow", "Fen", "Stone", "Rook", "Vane", "Harrow", "Thorne", "Wick", "Grim", "Hearth",
    "Briar", "Cairn", "Dusk", "Flint", "Gale", "Marsh", "Pike", "Reeve", "Sable", "Ward", "Wren", "Yew",
];

const SETTLEMENT_TERMS: &[&str] = &["Haven", "Watch", "Crossing", "Hold", "Rest", "Market"];
const RUIN_TERMS: &[&str] = &["Ruins", "Remains", "Hollow", "Fall", "Vestige"];
const WILDERNESS_TERMS: &[&str] = &["Wilds", "Grove", "Reach", "Thicket", "Expanse"];
const MINE_TERMS: &[&str] = &["Mine", "Deep", "Pit", "Quarry", "Works"];
const SHRINE_TERMS: &[&str] = &["Shrine", "Sanctum", "Chapel", "Wayside"];
const CROSSROADS_TERMS: &[&str] = &["Fork", "Crossing", "Junction", "Ways", "Roadmeet"];

/// Populate a generated world with deterministic, context-driven locations,
/// factions, and NPCs. Authored entities are left untouched and names are
/// selected to avoid colliding with authored content.
pub fn populate_generated_entities(
    state: &mut GameState,
    content: &CampaignContent,
) -> (usize, usize) {
    if state.world.generation.is_none() {
        return (0, 0);
    }

    let characteristics = generate_world_characteristics(&state.world);
    let generated_location_names = rename_generated_locations(&mut state.world, &characteristics, content);

    let faction_start = state.factions.len();
    let generated_factions = generate_factions(&mut state.world, &mut state.factions, &characteristics, content);
    let generated_npcs = generate_npcs(
        &mut state.world,
        &state.factions,
        &mut state.npcs,
        &characteristics,
        &generated_location_names,
        content,
    );

    assert_eq!(state.factions.len() - faction_start, generated_factions);
    (generated_factions, generated_npcs)
}

fn rename_generated_locations(
    world: &mut World,
    characteristics: &crate::procedural_characteristics::WorldCharacteristics,
    content: &CampaignContent,
) -> HashSet<EntityId> {
    let authored_names = content
        .world
        .locations
        .iter()
        .map(|location| location.name.as_str())
        .collect::<HashSet<_>>();
    let used_names = world
        .locations
        .iter()
        .map(|location| location.name.clone())
        .collect::<HashSet<_>>();
    let seed = world
        .generation
        .as_ref()
        .map(|generation| generation.seed)
        .unwrap_or(world.id);

    let mut generated_ids = HashSet::new();
    let mut reserved = used_names;
    for (index, location) in world.locations.iter_mut().enumerate() {
        if !location.name.starts_with("Generated Site ")
            || authored_names.contains(location.name.as_str())
        {
            continue;
        }
        let Some(location_characteristics) = characteristics
            .locations
            .iter()
            .find(|entry| entry.location_id == location.id)
        else {
            continue;
        };
        let Some(region) = characteristics.region_characteristics(location_characteristics.region_id) else {
            continue;
        };
        let base = location_base_name(location_characteristics.kind, region.theme, region.climate);
        let term = term_for(
            location_characteristics.kind,
            (seed as usize + index) % term_pool_len(location_characteristics.kind),
        );
        let mut candidate = format!("{base} {term}");
        let mut suffix = 2;
        while reserved.contains(&candidate) || authored_names.contains(candidate.as_str()) {
            candidate = format!("{base} {term} {suffix}");
            suffix += 1;
        }
        location.name = candidate.clone();
        if location.description == "An unexplored place shaped by the world seed." {
            location.description =
                generated_location_description(location_characteristics, region, &candidate);
        }
        reserved.insert(candidate);
        generated_ids.insert(location.id);
    }
    generated_ids
}

fn generate_factions(
    world: &mut World,
    factions: &mut Vec<Faction>,
    characteristics: &crate::procedural_characteristics::WorldCharacteristics,
    content: &CampaignContent,
) -> usize {
    let existing = factions
        .iter()
        .map(|faction| faction.name.clone())
        .chain(content.factions.iter().map(|faction| faction.name.clone()))
        .collect::<HashSet<_>>();
    let mut reserved = existing;
    let mut added = 0;

    for region in &characteristics.regions {
        let should_generate = region.population >= 200 || region.prosperity >= 45;
        if !should_generate {
            continue;
        }
        let Some(world_region) = world.regions.iter().find(|candidate| candidate.id == region.region_id) else {
            continue;
        };
        let base = faction_base_name(region.theme, region.climate, region.resources.first().map(String::as_str));
        let mut name = format!("{} of {}", base, world_region.name);
        let mut suffix = 2;
        while reserved.contains(&name) {
            name = format!("{} of {} {}", base, world_region.name, suffix);
            suffix += 1;
        }
        let id = world.allocate_id();
        let mut faction = Faction::new(id, name.clone());
        faction.memory.push(format!(
            "A generated faction shaped by a {} {} region with {} prosperity and {} danger.",
            climate_label(region.climate),
            theme_label(region.theme),
            region.prosperity,
            region.danger
        ));
        factions.push(faction);
        reserved.insert(name);
        added += 1;
    }
    added
}

fn generate_npcs(
    world: &mut World,
    factions: &[Faction],
    npcs: &mut Vec<Npc>,
    characteristics: &crate::procedural_characteristics::WorldCharacteristics,
    generated_location_ids: &HashSet<EntityId>,
    content: &CampaignContent,
) -> usize {
    let authored_names = content
        .npcs
        .iter()
        .map(|npc| npc.name.clone())
        .collect::<HashSet<_>>();
    let mut reserved = npcs
        .iter()
        .map(|npc| npc.name.clone())
        .chain(authored_names.iter().cloned())
        .collect::<HashSet<_>>();
    let seed = world
        .generation
        .as_ref()
        .map(|generation| generation.seed)
        .unwrap_or(world.id);
    let mut generated = 0;

    for location in &characteristics.locations {
        if !generated_location_ids.contains(&location.location_id) || location.population == 0 {
            continue;
        }
        let npc_count = npc_count_for(location);
        let Some(faction_id) = factions
            .iter()
            .find(|faction| faction_location_context(faction, world, location.region_id))
            .map(|faction| faction.id)
        else {
            continue;
        };
        for slot in 0..npc_count {
            let index = seed
                .wrapping_add(location.location_id)
                .wrapping_add((slot as u64).wrapping_mul(31)) as usize;
            let given = GIVEN_NAMES[index % GIVEN_NAMES.len()];
            let family = FAMILY_NAMES[(index / GIVEN_NAMES.len()) % FAMILY_NAMES.len()];
            let mut name = format!("{given} {family}");
            let mut suffix = 2;
            while reserved.contains(&name) {
                name = format!("{given} {family} {suffix}");
                suffix += 1;
            }
            let title = npc_title(location.kind, slot);
            let id = world.allocate_id();
            let mut npc = Npc::new(id, name.clone(), title, location.location_id, Some(faction_id));
            npc.memory.push(format!(
                "Lives in a {} shaped by {} conditions and local resources.",
                location_kind_label(location.kind),
                location_tags(location)
            ));
            npcs.push(npc);
            reserved.insert(name);
            generated += 1;
        }
    }
    generated
}

fn faction_location_context(
    faction: &Faction,
    world: &World,
    region_id: EntityId,
) -> bool {
    world
        .regions
        .iter()
        .find(|region| region.id == region_id)
        .map(|region| faction.name.contains(&region.name))
        .unwrap_or(false)
}

fn npc_count_for(location: &LocationCharacteristics) -> usize {
    match location.kind {
        LocationKind::Settlement => {
            if location.population >= 120 { 3 } else { 2 }
        }
        LocationKind::Crossroads | LocationKind::Mine => 1,
        LocationKind::Shrine => usize::from(location.population >= 5),
        LocationKind::Ruin | LocationKind::Wilderness => usize::from(location.population >= 5),
    }
}

fn npc_title(kind: LocationKind, slot: usize) -> String {
    match kind {
        LocationKind::Settlement => ["Steward", "Trader", "Guard"][slot % 3].to_string(),
        LocationKind::Crossroads => "Waykeeper".to_string(),
        LocationKind::Mine => "Prospector".to_string(),
        LocationKind::Shrine => "Caretaker".to_string(),
        LocationKind::Ruin => "Scavenger".to_string(),
        LocationKind::Wilderness => "Wanderer".to_string(),
    }
}

fn location_base_name(kind: LocationKind, theme: RegionTheme, climate: Climate) -> String {
    match kind {
        LocationKind::Settlement => settlement_prefix(theme, climate).to_string(),
        LocationKind::Ruin => ruin_prefix(theme).to_string(),
        LocationKind::Wilderness => wilderness_prefix(theme).to_string(),
        LocationKind::Mine => mine_prefix(theme).to_string(),
        LocationKind::Shrine => shrine_prefix(theme).to_string(),
        LocationKind::Crossroads => crossroads_prefix(theme).to_string(),
    }
}

fn settlement_prefix(theme: RegionTheme, climate: Climate) -> &'static str {
    match (theme, climate) {
        (RegionTheme::Coast, _) => "Salt",
        (RegionTheme::Marsh, _) => "Fen",
        (RegionTheme::Highlands, _) => "Stone",
        (RegionTheme::Woodland, _) => "Green",
        (RegionTheme::Wastes, _) => "Ash",
        (RegionTheme::Frontier, Climate::Cold) => "Frost",
        _ => "Ember",
    }
}

fn ruin_prefix(theme: RegionTheme) -> &'static str {
    match theme {
        RegionTheme::Coast => "Tide",
        RegionTheme::Marsh => "Drowned",
        RegionTheme::Highlands => "Broken",
        RegionTheme::Woodland => "Overgrown",
        RegionTheme::Wastes => "Ashen",
        RegionTheme::Frontier => "Fallen",
    }
}

fn wilderness_prefix(theme: RegionTheme) -> &'static str {
    match theme {
        RegionTheme::Coast => "Salt",
        RegionTheme::Marsh => "Sinking",
        RegionTheme::Highlands => "High",
        RegionTheme::Woodland => "Blackroot",
        RegionTheme::Wastes => "Grey",
        RegionTheme::Frontier => "Lonely",
    }
}

fn mine_prefix(theme: RegionTheme) -> &'static str {
    match theme {
        RegionTheme::Highlands => "Iron",
        RegionTheme::Wastes => "Cinder",
        _ => "Old",
    }
}

fn shrine_prefix(theme: RegionTheme) -> &'static str {
    match theme {
        RegionTheme::Woodland => "Green",
        RegionTheme::Marsh => "Fen",
        RegionTheme::Coast => "Tide",
        _ => "Ash",
    }
}

fn crossroads_prefix(theme: RegionTheme) -> &'static str {
    match theme {
        RegionTheme::Coast => "Saltroad",
        RegionTheme::Highlands => "Highroad",
        RegionTheme::Woodland => "Greenway",
        RegionTheme::Marsh => "Fenway",
        RegionTheme::Wastes => "Cinderway",
        RegionTheme::Frontier => "Oldroad",
    }
}

fn term_pool_len(kind: LocationKind) -> usize {
    match kind {
        LocationKind::Settlement => SETTLEMENT_TERMS.len(),
        LocationKind::Ruin => RUIN_TERMS.len(),
        LocationKind::Wilderness => WILDERNESS_TERMS.len(),
        LocationKind::Mine => MINE_TERMS.len(),
        LocationKind::Shrine => SHRINE_TERMS.len(),
        LocationKind::Crossroads => CROSSROADS_TERMS.len(),
    }
}

fn term_for(kind: LocationKind, index: usize) -> &'static str {
    let terms = match kind {
        LocationKind::Settlement => SETTLEMENT_TERMS,
        LocationKind::Ruin => RUIN_TERMS,
        LocationKind::Wilderness => WILDERNESS_TERMS,
        LocationKind::Mine => MINE_TERMS,
        LocationKind::Shrine => SHRINE_TERMS,
        LocationKind::Crossroads => CROSSROADS_TERMS,
    };
    terms[index % terms.len()]
}

fn faction_base_name(theme: RegionTheme, climate: Climate, resource: Option<&str>) -> &'static str {
    match (theme, climate, resource) {
        (RegionTheme::Coast, _, Some("fish")) => "Fisherfolk League",
        (RegionTheme::Highlands, _, Some("ore")) => "Stonebound Company",
        (RegionTheme::Woodland, _, Some("timber")) => "Greenwood Circle",
        (RegionTheme::Marsh, _, _) => "Fen Covenant",
        (RegionTheme::Wastes, _, _) => "Ashen Remnant",
        (RegionTheme::Frontier, Climate::Cold, _) => "Frost Marchers",
        (RegionTheme::Frontier, _, _) => "March Wardens",
        _ => "Roadward Compact",
    }
}

fn generated_location_description(
    location: &LocationCharacteristics,
    region: &RegionCharacteristics,
    name: &str,
) -> String {
    format!(
        "{name} lies in the {} {}. It is a {} place with {} local population and access to {}.",
        climate_label(region.climate),
        theme_label(region.theme),
        location_kind_label(location.kind),
        location.population,
        location.resources.join(", ")
    )
}

fn location_tags(location: &LocationCharacteristics) -> String {
    if location.tags.is_empty() {
        "quiet local character".to_string()
    } else {
        location.tags.join(", ")
    }
}

fn location_kind_label(kind: LocationKind) -> &'static str {
    match kind {
        LocationKind::Settlement => "settled",
        LocationKind::Ruin => "ruined",
        LocationKind::Wilderness => "wild",
        LocationKind::Mine => "industrial",
        LocationKind::Shrine => "sacred",
        LocationKind::Crossroads => "transitional",
    }
}

fn theme_label(theme: RegionTheme) -> &'static str {
    match theme {
        RegionTheme::Frontier => "frontier",
        RegionTheme::Woodland => "woodland",
        RegionTheme::Highlands => "highland",
        RegionTheme::Marsh => "marsh",
        RegionTheme::Wastes => "waste",
        RegionTheme::Coast => "coastal",
    }
}

fn climate_label(climate: Climate) -> &'static str {
    match climate {
        Climate::Cold => "cold",
        Climate::Temperate => "temperate",
        Climate::Arid => "arid",
        Climate::Wet => "wet",
    }
}

trait WorldCharacteristicsExt {
    fn region_characteristics(&self, region_id: EntityId) -> Option<&RegionCharacteristics>;
}

impl WorldCharacteristicsExt for crate::procedural_characteristics::WorldCharacteristics {
    fn region_characteristics(&self, region_id: EntityId) -> Option<&RegionCharacteristics> {
        self.regions
            .iter()
            .find(|region| region.region_id == region_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_campaign_content;
    use crate::model::{WorldGenerationMetadata, WorldMode};
    use crate::procedural::{generate_world, WorldGenerationConfig};

    fn generated_state() -> GameState {
        let content = load_campaign_content();
        let seed = 4242;
        let config = WorldGenerationConfig::default();
        let mut world = generate_world("Ashen", seed, config);
        world.mode = WorldMode::New;
        world.generation = Some(WorldGenerationMetadata {
            seed,
            region_count: config.region_count,
            location_count: config.location_count,
            extra_edges: config.extra_edges,
        });
        crate::procedural::place_authored_content(&mut world, &content);
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
    fn generated_entities_are_deterministic() {
        let content = load_campaign_content();
        let mut a = generated_state();
        let mut b = generated_state();
        let first = populate_generated_entities(&mut a, &content);
        let second = populate_generated_entities(&mut b, &content);
        assert_eq!(first, second);
        assert_eq!(
            a.world
                .locations
                .iter()
                .map(|l| &l.name)
                .collect::<Vec<_>>(),
            b.world
                .locations
                .iter()
                .map(|l| &l.name)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            a.factions.iter().map(|f| &f.name).collect::<Vec<_>>(),
            b.factions.iter().map(|f| &f.name).collect::<Vec<_>>()
        );
        assert_eq!(
            a.npcs
                .iter()
                .map(|npc| (&npc.name, npc.location_id, npc.faction_id))
                .collect::<Vec<_>>(),
            b.npcs
                .iter()
                .map(|npc| (&npc.name, npc.location_id, npc.faction_id))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn generated_entities_are_idempotent() {
        let content = load_campaign_content();
        let mut state = generated_state();
        let first = populate_generated_entities(&mut state, &content);
        let second = populate_generated_entities(&mut state, &content);
        assert_eq!(first, second);
        assert_eq!(state.factions.len(), content.factions.len() + first.0);
        assert_eq!(state.npcs.len(), content.npcs.len() + first.1);
    }

    #[test]
    fn generated_npcs_reference_generated_locations_and_factions() {
        let content = load_campaign_content();
        let mut state = generated_state();
        populate_generated_entities(&mut state, &content);
        for npc in &state.npcs {
            assert!(state.world.location_by_id(npc.location_id).is_some());
            if let Some(faction_id) = npc.faction_id {
                assert!(state.factions.iter().any(|faction| faction.id == faction_id));
            }
        }
    }
}
