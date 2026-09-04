use crate::content::CampaignContent;
use crate::model::{Location, Region, World, WorldMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationConfig {
    pub region_count: usize,
    pub location_count: usize,
    pub extra_edges: usize,
}

impl Default for WorldGenerationConfig {
    fn default() -> Self {
        Self {
            region_count: 3,
            location_count: 12,
            extra_edges: 8,
        }
    }
}

impl WorldGenerationConfig {
    fn normalized(self) -> Self {
        let location_count = self.location_count.max(1);
        Self {
            region_count: self.region_count.max(1).min(location_count),
            location_count,
            extra_edges: self.extra_edges,
        }
    }
}

/// Generate a deterministic, connected world graph from `seed`.
///
/// The generator only establishes structural world data. Authored campaign
/// content can be anchored into the generated structure with
/// [`place_authored_content`] without changing the generation rules.
pub fn generate_world(
    world_name: impl Into<String>,
    seed: u64,
    config: WorldGenerationConfig,
) -> World {
    let config = config.normalized();
    let mut rng = DeterministicRng::new(seed);
    let mut world = World::new(&world_name.into(), WorldMode::New);

    let mut region_ids = Vec::with_capacity(config.region_count);
    for index in 0..config.region_count {
        let id = world.allocate_id();
        world.regions.push(Region {
            id,
            name: format!("Region {:02}", index + 1),
            description: format!("A generated region shaped by seed {seed:016x}."),
            location_ids: Vec::new(),
        });
        region_ids.push(id);
    }

    for index in 0..config.location_count {
        let region_index = (index * config.region_count) / config.location_count;
        let region_id = region_ids[region_index];
        let id = world.allocate_id();
        world.locations.push(Location {
            id,
            name: format!("Generated Site {:02}", index + 1),
            description: "An unexplored place shaped by the world seed.".to_string(),
            region_id,
            dangerous: rng.next_bool(),
            corpse_ids: Vec::new(),
            exits: Vec::new(),
        });
    }

    for index in 1..world.locations.len() {
        add_bidirectional_exit(&mut world, index - 1, index);
    }

    let max_extra_edges = possible_extra_edges(world.locations.len());
    let extra_edges = config.extra_edges.min(max_extra_edges);
    let mut added_edges = 0;
    let mut attempts = 0;
    let max_attempts = extra_edges.saturating_mul(8).saturating_add(16);

    while added_edges < extra_edges && attempts < max_attempts {
        attempts += 1;
        let a = rng.gen_range(world.locations.len());
        let b = rng.gen_range(world.locations.len());
        if a == b || has_exit(&world.locations[a], world.locations[b].id) {
            continue;
        }
        add_bidirectional_exit(&mut world, a, b);
        added_edges += 1;
    }

    rebuild_region_location_ids(&mut world);
    world
}

/// Place authored campaign locations into deterministic generated slots.
///
/// The first generated region is the anchor for the authored campaign region.
/// Authored locations are ordered by stable content ID and mapped onto the
/// first generated location slots. Their names, descriptions, danger state,
/// and authored exit relationships are preserved. Existing generated exits
/// remain, so the procedural graph continues to provide connectivity and
/// additional routes around authored locations.
pub fn place_authored_content(world: &mut World, content: &CampaignContent) -> usize {
    if world.regions.is_empty() || content.world.locations.is_empty() {
        return 0;
    }

    while world.locations.len() < content.world.locations.len() {
        add_generated_location(world);
    }

    let anchor_region_id = world.regions[0].id;
    world.regions[0].name = content.world.region.name.clone();
    world.regions[0].description = content.world.region.description.clone();

    let mut authored_locations = content.world.locations.iter().collect::<Vec<_>>();
    authored_locations.sort_by(|left, right| left.id.cmp(&right.id));

    let authored_runtime_ids = authored_locations
        .iter()
        .enumerate()
        .map(|(index, location)| (location.id.as_str(), world.locations[index].id))
        .collect::<std::collections::HashMap<_, _>>();

    for (index, authored) in authored_locations.iter().enumerate() {
        let generated_id = world.locations[index].id;
        let generated_exits = world.locations[index].exits.clone();
        let authored_exits = authored
            .exits
            .iter()
            .filter_map(|exit_id| authored_runtime_ids.get(exit_id.as_str()).copied())
            .collect::<Vec<_>>();

        let mut exits = generated_exits;
        for exit_id in authored_exits {
            if exit_id != generated_id && !exits.contains(&exit_id) {
                exits.push(exit_id);
            }
        }

        world.locations[index].name = authored.name.clone();
        world.locations[index].description = authored.description.clone();
        world.locations[index].dangerous = authored.dangerous;
        world.locations[index].region_id = anchor_region_id;
        world.locations[index].exits = exits;
    }

    rebuild_region_location_ids(world);
    authored_locations.len()
}

fn add_generated_location(world: &mut World) {
    let id = world.allocate_id();
    let region_id = world
        .regions
        .last()
        .map(|region| region.id)
        .or_else(|| world.regions.first().map(|region| region.id))
        .expect("generated world has at least one region");
    let index = world.locations.len() + 1;
    world.locations.push(Location {
        id,
        name: format!("Generated Site {index:02}"),
        description: "An unexplored place shaped by the world seed.".to_string(),
        region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: Vec::new(),
    });
    if index > 1 {
        add_bidirectional_exit(world, index - 2, index - 1);
    }
}

fn rebuild_region_location_ids(world: &mut World) {
    for region in &mut world.regions {
        region.location_ids.clear();
    }
    for location in &world.locations {
        if let Some(region) = world
            .regions
            .iter_mut()
            .find(|region| region.id == location.region_id)
        {
            region.location_ids.push(location.id);
        }
    }
}

fn possible_extra_edges(location_count: usize) -> usize {
    location_count
        .saturating_mul(location_count.saturating_sub(1))
        .saturating_div(2)
        .saturating_sub(location_count.saturating_sub(1))
}

fn add_bidirectional_exit(world: &mut World, a_index: usize, b_index: usize) {
    let a_id = world.locations[a_index].id;
    let b_id = world.locations[b_index].id;
    if !has_exit(&world.locations[a_index], b_id) {
        world.locations[a_index].exits.push(b_id);
    }
    if !has_exit(&world.locations[b_index], a_id) {
        world.locations[b_index].exits.push(a_id);
    }
}

fn has_exit(location: &Location, target_id: u64) -> bool {
    location.exits.contains(&target_id)
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    fn gen_range(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        (self.next_u64() as usize) % upper
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_campaign_content;
    use std::collections::HashSet;

    #[test]
    fn same_seed_and_config_produce_identical_world_structure() {
        let config = WorldGenerationConfig {
            region_count: 4,
            location_count: 20,
            extra_edges: 10,
        };
        let a = generate_world("Ashen", 12345, config);
        let b = generate_world("Ashen", 12345, config);

        assert_eq!(a.regions.len(), b.regions.len());
        assert_eq!(a.locations.len(), b.locations.len());
        assert_eq!(
            a.regions
                .iter()
                .map(|region| (&region.name, &region.location_ids))
                .collect::<Vec<_>>(),
            b.regions
                .iter()
                .map(|region| (&region.name, &region.location_ids))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            a.locations
                .iter()
                .map(|location| (
                    &location.name,
                    location.region_id,
                    location.dangerous,
                    &location.exits
                ))
                .collect::<Vec<_>>(),
            b.locations
                .iter()
                .map(|location| (
                    &location.name,
                    location.region_id,
                    location.dangerous,
                    &location.exits
                ))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn generated_world_is_connected_from_the_starting_location() {
        let world = generate_world("Ashen", 7, WorldGenerationConfig::default());
        let start = world
            .locations
            .first()
            .expect("generator creates a location")
            .id;
        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }
            if let Some(location) = world.location_by_id(id) {
                stack.extend(location.exits.iter().copied());
            }
        }

        assert_eq!(visited.len(), world.locations.len());
    }

    #[test]
    fn generated_world_has_valid_regions_and_symmetric_exits() {
        let world = generate_world(
            "Ashen",
            99,
            WorldGenerationConfig {
                region_count: 5,
                location_count: 17,
                extra_edges: 12,
            },
        );
        let region_ids = world
            .regions
            .iter()
            .map(|region| region.id)
            .collect::<HashSet<_>>();
        let location_ids = world
            .locations
            .iter()
            .map(|location| location.id)
            .collect::<HashSet<_>>();

        assert!(!world.regions.is_empty());
        assert!(!world.locations.is_empty());
        assert!(world
            .locations
            .iter()
            .all(|location| region_ids.contains(&location.region_id)));
        assert!(world.regions.iter().all(|region| region
            .location_ids
            .iter()
            .all(|id| location_ids.contains(id))));
        assert!(world.locations.iter().all(|location| {
            location.exits.iter().all(|target_id| {
                world
                    .location_by_id(*target_id)
                    .map(|target| target.exits.contains(&location.id))
                    .unwrap_or(false)
            })
        }));
    }

    #[test]
    fn zero_sized_configuration_is_normalized_to_a_minimal_world() {
        let world = generate_world(
            "Ashen",
            1,
            WorldGenerationConfig {
                region_count: 0,
                location_count: 0,
                extra_edges: 0,
            },
        );

        assert_eq!(world.regions.len(), 1);
        assert_eq!(world.locations.len(), 1);
        assert!(world.locations[0].exits.is_empty());
    }

    #[test]
    fn authored_content_is_placed_deterministically_and_preserves_authored_relationships() {
        let content = load_campaign_content();
        let config = WorldGenerationConfig {
            region_count: 3,
            location_count: 12,
            extra_edges: 6,
        };
        let mut a = generate_world("Ashen", 2026, config);
        let mut b = generate_world("Ashen", 2026, config);

        let placed_a = place_authored_content(&mut a, &content);
        let placed_b = place_authored_content(&mut b, &content);

        assert_eq!(placed_a, content.world.locations.len());
        assert_eq!(placed_a, placed_b);
        assert_eq!(
            a.locations
                .iter()
                .map(|location| (
                    &location.name,
                    &location.description,
                    location.region_id,
                    location.dangerous,
                    &location.exits
                ))
                .collect::<Vec<_>>(),
            b.locations
                .iter()
                .map(|location| (
                    &location.name,
                    &location.description,
                    location.region_id,
                    location.dangerous,
                    &location.exits
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            a.regions
                .iter()
                .map(|region| (&region.name, &region.description, &region.location_ids))
                .collect::<Vec<_>>(),
            b.regions
                .iter()
                .map(|region| (&region.name, &region.description, &region.location_ids))
                .collect::<Vec<_>>()
        );

        assert_eq!(a.regions[0].name, content.world.region.name);

        let runtime_by_name = a
            .locations
            .iter()
            .map(|location| (location.name.as_str(), location.id))
            .collect::<std::collections::HashMap<_, _>>();
        for authored in &content.world.locations {
            let runtime_id = runtime_by_name[authored.name.as_str()];
            let runtime = a
                .location_by_id(runtime_id)
                .expect("authored location exists");
            for authored_exit in &authored.exits {
                let target_runtime_id = content
                    .world
                    .locations
                    .iter()
                    .find(|location| location.id == *authored_exit)
                    .and_then(|location| runtime_by_name.get(location.name.as_str()))
                    .copied()
                    .expect("authored exit resolves to an authored location");
                assert!(runtime.exits.contains(&target_runtime_id));
            }
        }

        let authored_names = content
            .world
            .locations
            .iter()
            .map(|location| location.name.as_str())
            .collect::<HashSet<_>>();
        assert!(a
            .locations
            .iter()
            .take(content.world.locations.len())
            .all(|location| authored_names.contains(location.name.as_str())));
    }
}
